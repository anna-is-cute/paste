use config::Config;
use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::models::users::User;
use database::schema::{users, pastes};
use errors::*;
use models::paste::output::{Output, OutputAuthor};
use models::paste::{Visibility, Content};
use routes::web::{context, Rst, OptionalWebUser, Session};

use diesel::dsl::count;
use diesel::prelude::*;

use rocket::State;
use rocket::http::Status as HttpStatus;

use rocket_contrib::Template;

use std::fs::File;
use std::io::Read;

#[get("/users/<username>")]
fn get(username: String, config: State<Config>, user: OptionalWebUser, sess: Session, conn: DbConn) -> Result<Rst> {
  _get(1, username, config, user, sess, conn)
}

#[get("/users/<username>?<params>")]
fn get_page(username: String, params: PageParams, config: State<Config>, user: OptionalWebUser, sess: Session, conn: DbConn) -> Result<Rst> {
  _get(params.page, username, config, user, sess, conn)
}

#[derive(Debug, FromForm)]
struct PageParams {
  page: u32,
}

fn _get(page: u32, username: String, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
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

      let mut output_files = Vec::with_capacity(files.len());

      const LEN: usize = 385;
      let mut bytes = [0; LEN];

      for file in files {
        let mut f = file.as_output_file(false, &paste)?;

        // TODO: maybe store this in database or its own file?
        if !has_preview && file.is_binary() != Some(true) {
          let path = file.path(&paste);
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

        output_files.push(f);
      }

      outputs.push(Output::new(
        paste.id(),
        Some(author.clone()),
        paste.name(),
        paste.description(),
        paste.visibility(),
        paste.created_at(),
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
  Ok(Rst::Template(Template::render("user/index", ctx)))
}
