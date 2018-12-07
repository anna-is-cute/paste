use crate::{
  config::Config,
  database::{
    DbConn,
    models::{pastes::Paste as DbPaste, users::User},
    schema::{users, pastes},
  },
  errors::*,
  models::paste::{
    Visibility, Content,
    output::{Output, OutputAuthor},
  },
  routes::web::{context, Rst, Links, OptionalWebUser, Session},
};

use diesel::{dsl::count, prelude::*};

use rocket::{State, http::Status as HttpStatus};

use rocket_contrib::templates::Template;

use serde_json::json;

use std::{fs::File, io::Read};

#[get("/u/<username>?<page>")]
pub fn get(username: String, page: Option<u32>, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  let page = page.unwrap_or(1);
  // TODO: make PositiveNumber struct or similar (could make Positive<num::Integer> or something)
  if page == 0 {
    return Ok(Rst::Status(HttpStatus::NotFound));
  }
  let target: User = match users::table.filter(users::username.eq(&username)).first(&*conn).optional()? {
    Some(u) => u,
    None => return Ok(Rst::Status(HttpStatus::NotFound)),
  };

  let mut query = DbPaste::belonging_to(&target)
    .select(count(pastes::id))
    .into_boxed();
  if Some(target.id()) != user.as_ref().map(|x| x.id()) {
    query = query.filter(pastes::visibility.eq(Visibility::Public));
  }
  let total_pastes: i64 = query.get_result(&*conn)?;

  let outputs = if total_pastes == 0 && page == 1 {
    Vec::default()
  } else {
    let page = i64::from(page);
    let offset = (page - 1) * 15;
    if offset >= total_pastes {
      return Ok(Rst::Status(HttpStatus::NotFound));
    }
    let pastes: Vec<DbPaste> = if Some(target.id()) == user.as_ref().map(|x| x.id()) {
      DbPaste::belonging_to(&target)
        .order_by(pastes::created_at.desc())
        .offset(offset)
        .limit(15)
        .load(&*conn)?
    } else {
      DbPaste::belonging_to(&target)
        .filter(pastes::visibility.eq(Visibility::Public))
        .order_by(pastes::created_at.desc())
        .offset(offset)
        .limit(15)
        .load(&*conn)?
    };

    let author = OutputAuthor::new(target.id(), target.username(), target.name());

    let mut outputs = Vec::with_capacity(pastes.len());

    for paste in pastes {
      let id = paste.id();

      let files = id.files(&conn)?;

      let mut has_preview = false;

      let mut output_files = id.output_files(&*config, &conn, &paste, false)?;

      const LEN: usize = 385;
      let mut bytes = [0; LEN];

      for f in &mut output_files {
        let file = match files.iter().find(|x| x.id() == f.id) {
          Some(f) => f,
          None => continue,
        };
        // TODO: maybe store this in database or its own file?
        if !has_preview && file.is_binary() != Some(true) {
          let path = file.path(&*config, &paste);
          let read = File::open(path)?.read(&mut bytes)?;
          let full = read < LEN;
          let end = if read == LEN { read - 1 } else { read };

          let preview = match String::from_utf8(bytes[..end].to_vec()) {
            Ok(s) => Some((full, s)),
            Err(e) => {
              let valid = e.utf8_error().valid_up_to();
              if valid > 0 {
                let p = unsafe { String::from_utf8_unchecked(bytes[..valid].to_vec()) };
                Some((full, p))
              } else {
                None
              }
            },
          };

          if let Some((full, mut p)) = preview {
            if !full {
              if let Some((mut i, _)) = p.rmatch_indices(|x| x == '\r' || x == '\n').next() {
                if i != 0 && p.len() > i && &p[i - 1..i] == "\r" {
                  i -= 1;
                }
                p.truncate(i);
              }
            }

            let p = p
              .lines()
              .take(10)
              .collect::<Vec<_>>()
              .join("\n");

            f.content = Some(Content::Text(p));
            has_preview = true;
          }
        }
      }

      outputs.push(Output::new(
        paste.id(),
        Some(author.clone()),
        paste.name(),
        paste.description(),
        paste.visibility(),
        paste.created_at(),
        paste.updated_at(&*config).ok(), // FIXME
        paste.expires(),
        None,
        output_files,
      ));
    }

    outputs
  };

  let mut ctx = context(&*config, user.as_ref(), &mut sess);
  ctx["pastes"] = json!(outputs);
  ctx["target"] = json!(target);
  ctx["page"] = json!(page);
  ctx["total"] = json!(total_pastes);
  ctx["links"] = json!(user_links(user.as_ref(), &target, &outputs, page));
  Ok(Rst::Template(Template::render("user/index", ctx)))
}

fn user_links(user: Option<&User>, target: &User, pastes: &[Output], page: u32) -> Links {
  let mut links = links!(
    "target_avatar" => uri!(crate::routes::web::account::avatar::get: target.id()),
    "next_page" => uri!(crate::routes::web::users::get::get:
      target.username(),
      page + 1,
    ),
    "prev_page" => if page <= 2 {
      uri!(crate::routes::web::users::get::get: target.username(), _)
    } else {
      uri!(crate::routes::web::users::get::get:
        target.username(),
        page - 1,
      )
    },
  );

  if let Some(ref u) = user {
    links.add("delete_multiple", uri!(crate::routes::web::pastes::delete::ids: u.username()));
  }

  links.add_value(
    "pastes",
    pastes
      .iter()
      .fold(&mut Links::default(), |l, x| l.add(
        x.id.to_simple().to_string(),
        uri!(crate::routes::web::pastes::get::users_username_id: target.username(), x.id),
      )),
  );

  links
}
