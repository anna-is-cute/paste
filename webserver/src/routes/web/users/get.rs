use crate::{
  config::Config,
  database::{
    DbConn,
    models::{pastes::Paste as DbPaste, users::User},
    schema::{users, pastes},
  },
  errors::*,
  models::{
    id::PasteId,
    paste::{
      Visibility, Content,
      output::{Output, OutputAuthor},
    },
  },
  routes::web::{context, Rst, Links, OptionalWebUser, Session},
  utils::UrlDate,
};

use chrono::{DateTime, Utc};

use diesel::{dsl::count, prelude::*};

use rocket::{State, http::Status as HttpStatus};

use rocket_contrib::templates::Template;

use serde_json::json;

use std::{fs::File, io::Read};

#[get("/u/<username>", rank = 3)]
pub fn get(username: String, config: State<Config>, user: OptionalWebUser, sess: Session, conn: DbConn) -> Result<Rst> {
  _get(Relative::Before(Utc::now()), username, config, user, sess, conn)
}

#[get("/u/<username>?<after>", rank = 2)]
pub fn get_after(username: String, after: UrlDate, config: State<Config>, user: OptionalWebUser, sess: Session, conn: DbConn) -> Result<Rst> {
  println!("after: {}", *after);
  _get(Relative::After(after.into_inner()), username, config, user, sess, conn)
}

#[get("/u/<username>?<before>", rank = 1)]
pub fn get_before(username: String, before: UrlDate, config: State<Config>, user: OptionalWebUser, sess: Session, conn: DbConn) -> Result<Rst> {
  println!("before: {}", *before);
  _get(Relative::Before(before.into_inner()), username, config, user, sess, conn)
}

enum Relative {
  Before(DateTime<Utc>),
  After(DateTime<Utc>),
}

fn _get(rel: Relative, username: String, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
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

  let (mut can_prev, mut can_next) = (false, false);

  let outputs: Vec<Output> = if total_pastes == 0 {
    Vec::default()
  } else {
    let mut query_1 = DbPaste::belonging_to(&target)
      .limit(16)
      .into_boxed();
    let mut query_2 = DbPaste::belonging_to(&target)
      .select(pastes::id)
      .limit(1)
      .into_boxed();

    if Some(target.id()) != user.as_ref().map(|x| x.id()) {
      query_1 = query_1.filter(pastes::visibility.eq(Visibility::Public));
      query_2 = query_2.filter(pastes::visibility.eq(Visibility::Public));
    }

    match rel {
      Relative::Before(ref date) => {
        query_1 = query_1
          .order_by(pastes::created_at.desc())
          .filter(pastes::created_at.lt(date.naive_utc()));
        query_2 = query_2.order_by(pastes::created_at.desc());
      },
      Relative::After(ref date) => {
        query_1 = query_1
          .order_by(pastes::created_at.asc())
          .filter(pastes::created_at.gt(date.naive_utc()));
        query_2 = query_2.order_by(pastes::created_at.asc());
      },
    }

    let mut pastes: Vec<DbPaste> = query_1.load(&*conn)?;

    let edge_paste: Option<PasteId> = query_2.first(&*conn).optional()?;
    if let Some(ref p) = pastes.iter().next() {
      let can = Some(p.id()) != edge_paste;
      match rel {
        Relative::Before(_) => can_prev = can,
        Relative::After(_) => can_next = can,
      }
    }

    if pastes.len() == 16 {
      match rel {
        Relative::Before(_) => can_next = true,
        Relative::After(_) => can_prev = true,
      }
    }

    pastes.truncate(15);

    if let Relative::After(_) = rel {
      pastes.reverse();
    }

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
  ctx["total"] = json!(total_pastes);
  ctx["links"] = json!(user_links(
    user.as_ref(),
    &target,
    &outputs,
    if can_prev {
      outputs.iter().next().and_then(|x| x.paste.metadata.created_at)
    } else {
      None
    },
    if can_next {
      outputs.iter().last().and_then(|x| x.paste.metadata.created_at)
    } else {
      None
    },
  ));
  Ok(Rst::Template(Template::render("user/index", ctx)))
}

fn user_links(user: Option<&User>, target: &User, pastes: &[Output], first_date: Option<DateTime<Utc>>, last_date: Option<DateTime<Utc>>) -> Links {
  let mut links = Links::default();

  if let Some(first_date) = first_date {
    links.add(
      "prev_page",
      uri!(crate::routes::web::users::get::get_after:
        target.username(),
        UrlDate::from(first_date),
      ),
    );
  }

  if let Some(last_date) = last_date {
    links.add(
      "next_page",
      uri!(crate::routes::web::users::get::get_before:
        target.username(),
        UrlDate::from(last_date),
      ),
    );
  }

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
