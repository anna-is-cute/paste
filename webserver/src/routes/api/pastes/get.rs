use crate::{
  database::{
    DbConn,
    models::{
      pastes::Paste as DbPaste,
      users::User,
    },
  },
  database::schema::{pastes, users},
  models::{
    id::PasteId,
    paste::{
      Metadata, Visibility,
      output::{Output, OutputFile, OutputAuthor},
    },
    status::{Status, ErrorKind},
  },
  routes::{RouteResult, OptionalUser},
};

use diesel::prelude::*;

use rocket::{http::Status as HttpStatus, request::Form};

use std::cmp::min;

#[derive(Debug, Serialize)]
pub struct AllPaste {
  id: PasteId,
  #[serde(flatten)]
  metadata: Metadata,
}

#[derive(Debug, FromForm)]
pub struct AllQuery {
  limit: Option<u8>,
}

#[get("/?<query..>")]
pub fn get_all_query(query: Form<AllQuery>, conn: DbConn) -> RouteResult<Vec<AllPaste>> {
  _get_all(Some(query.into_inner()), conn)
}

#[get("/")]
pub fn get_all(conn: DbConn) -> RouteResult<Vec<AllPaste>> {
  _get_all(None, conn)
}

fn _get_all(query: Option<AllQuery>, conn: DbConn) -> RouteResult<Vec<AllPaste>> {
  let limit = min(100, query.and_then(|x| x.limit).unwrap_or(5));

  let pastes: Vec<DbPaste> = pastes::table
    .filter(pastes::visibility.eq(Visibility::Public))
    .order(pastes::created_at.desc())
    .limit(i64::from(limit))
    .load(&*conn)?;

  let output = pastes
    .into_iter()
    .map(|x| AllPaste {
      id: x.id(),
      metadata: Metadata {
        name: x.name().map(Into::into),
        description: x.description().map(Into::into),
        visibility: x.visibility(),
        expires: x.expires(),
        created_at: Some(x.created_at()),
        updated_at: x.updated_at().ok(),
      },
    })
    .collect();

  Ok(Status::show_success(HttpStatus::Ok, output))
}

#[derive(Debug, Default, FromForm)]
pub struct Full {
  full: Option<bool>,
}

// routes separated because of https://github.com/SergioBenitez/Rocket/issues/376

#[get("/<id>")]
pub fn get(id: PasteId, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  _get(id, None, user, conn)
}

#[get("/<id>?<query..>")]
pub fn get_query(id: PasteId, query: Form<Full>, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  _get(id, Some(query.into_inner()), user, conn)
}

fn _get(id: PasteId, query: Option<Full>, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  let paste = match id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Status::show_error(status, kind));
  }
  let query = query.unwrap_or_default();

  let full = query.full == Some(true);
  let files: Vec<OutputFile> = id.output_files(&conn, &paste, full)?;

  let author = match paste.author_id() {
    Some(author) => {
      let user: User = users::table.find(author).first(&*conn)?;
      Some(OutputAuthor::new(author, user.username(), user.name()))
    },
    None => None
  };

  let output = Output::new(
    id,
    author,
    paste.name(),
    paste.description(),
    paste.visibility(),
    paste.created_at(),
    paste.updated_at().ok(), // FIXME
    paste.expires(),
    None,
    files,
  );

  Ok(Status::show_success(HttpStatus::Ok, output))
}
