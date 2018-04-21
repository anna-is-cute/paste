use config::Config;
use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::models::users::User;
use database::schema::{pastes, users};
use errors::*;
use models::id::PasteId;
use models::paste::output::{Output, OutputFile, OutputAuthor};
use routes::web::OptionalWebUser;

use diesel::prelude::*;

use rocket::State;
use rocket::http::Status as HttpStatus;
use rocket::request::Request;
use rocket::response::{Redirect, Responder, Response};

use rocket_contrib::Template;

use std::result;

enum Rst {
  Redirect(Redirect),
  Status(HttpStatus),
  Template(Template),
}

impl<'r> Responder<'r> for Rst {
  fn respond_to(self, request: &Request) -> result::Result<Response<'r>, HttpStatus> {
    match self {
      Rst::Redirect(r) => r.respond_to(request),
      Rst::Status(s) => Err(s),
      Rst::Template(t) => t.respond_to(request),
    }
  }
}

#[get("/<id>")]
fn id<'r>(id: PasteId, user: OptionalWebUser, conn: DbConn) -> Result<Rst> {
  let result: Option<(Option<String>, DbPaste)> = users::table
    .inner_join(pastes::table)
    .select((users::username.nullable(), pastes::all_columns))
    .filter(pastes::id.eq(*id))
    .first(&*conn)
    .optional()?;

  let (owner, paste) = match result {
    Some(x) => x,
    None => return Ok(Rst::Status(HttpStatus::NotFound)),
  };

  if let Some((status, _)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Rst::Status(status));
  }

  let owner = owner.unwrap_or_else(|| "anonymous".into());
  Ok(Rst::Redirect(Redirect::to(&format!("/{}/{}", owner, id))))
}

#[get("/<username>/<id>")]
fn username_id(username: String, id: PasteId, config: State<Config>, user: OptionalWebUser, conn: DbConn) -> Result<Rst> {
  let paste: DbPaste = match id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Rst::Status(HttpStatus::NotFound)),
  };

  let (expected_username, author): (String, Option<OutputAuthor>) = match paste.author_id() {
    Some(author) => {
      let user: User = users::table.find(author).first(&*conn)?;
      (user.username().clone(), Some(OutputAuthor::new(&author, user.username().clone())))
    },
    None => ("anonymous".into(), None),
  };

  if username != expected_username {
    return Ok(Rst::Status(HttpStatus::NotFound));
  }

  if let Some((status, _)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Rst::Status(status));
  }

  let files: Vec<OutputFile> = id.files(&conn)?
    .iter()
    .map(|x| x.as_output_file(true))
    .collect::<result::Result<_, _>>()?;

  let output = Output::new(
    *id,
    author,
    paste.name().clone(),
    paste.description().clone(),
    paste.visibility(),
    None,
    files,
  );

  let ctx = json!({
    "paste": output,
    "config": &*config,
  });

  Ok(Rst::Template(Template::render("paste/index", ctx)))
}
