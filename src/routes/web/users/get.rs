use config::Config;
use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::models::users::User;
use database::schema::{users, pastes};
use errors::*;
use models::id::PasteId;
use models::paste::output::{Output, OutputAuthor, OutputFile};
use models::paste::Visibility;
use routes::web::{Rst, OptionalWebUser, Session};

use diesel::dsl::count;
use diesel::prelude::*;

use rocket::State;
use rocket::http::Status as HttpStatus;

use rocket_contrib::Template;

use std::result;

#[get("/users/<username>")]
fn get(username: String, config: State<Config>, user: OptionalWebUser, sess: Session, conn: DbConn) -> Result<Rst> {
  _get(1, username, config, user, sess, conn)
}

#[get("/users/<username>/<page>", rank = 1)]
fn get_page(username: String, page: u32, config: State<Config>, user: OptionalWebUser, sess: Session, conn: DbConn) -> Result<Rst> {
  _get(page, username, config, user, sess, conn)
}

fn _get(page: u32, username: String, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  if page == 0 {
    return Ok(Rst::Status(HttpStatus::NotFound));
  }
  let target: User = match users::table.filter(users::username.eq(&username)).first(&*conn).optional()? {
    Some(u) => u,
    None => return Ok(Rst::Status(HttpStatus::NotFound)),
  };
  // TODO: there must be a way to do this better
  let total_pastes: i64 = if Some(target.id()) == user.as_ref().map(|x| x.id()) {
    DbPaste::belonging_to(&target)
      .select(count(pastes::id))
      .get_result(&*conn)?
  } else {
    DbPaste::belonging_to(&target)
      .select(count(pastes::id))
      .filter(pastes::visibility.eq(Visibility::Public))
      .get_result(&*conn)?
  };
  if i64::from((page - 1) * 15) >= total_pastes {
    return Ok(Rst::Status(HttpStatus::NotFound));
  }
  let page = i64::from(page);
  let pastes: Vec<DbPaste> = if Some(target.id()) == user.as_ref().map(|x| x.id()) {
    DbPaste::belonging_to(&target)
      .order_by(pastes::created_at.desc())
      .offset((page - 1) * 15)
      .limit(15)
      .load(&*conn)?
  } else {
    DbPaste::belonging_to(&target)
      .filter(pastes::visibility.eq(Visibility::Public))
      .order_by(pastes::created_at.desc())
      .offset((page - 1) * 15)
      .limit(15)
      .load(&*conn)?
  };

  let author = OutputAuthor::new(&target.id(), target.username());

  let mut outputs = Vec::with_capacity(pastes.len());

  for paste in pastes {
    let id = PasteId(paste.id());

    let files: Vec<OutputFile> = id.files(&conn)?
      .iter()
      .map(|x| x.as_output_file(false))
      .collect::<result::Result<_, _>>()?;

    outputs.push(Output::new(
      paste.id(),
      Some(author.clone()),
      paste.name().clone(),
      paste.description().clone(),
      paste.visibility(),
      None,
      files,
    ));
  }

  let ctx = json!({
    "pastes": outputs,
    "config": &*config,
    "target": target,
    "user": &*user,
    "page": page,
    "total": total_pastes,
    "server_version": ::SERVER_VERSION,
    "resources_version": &*::RESOURCES_VERSION,
    "error": sess.data.remove("error"),
  });
  Ok(Rst::Template(Template::render("user/index", ctx)))
}
