use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::models::users::User;
use database::schema::{pastes, users};
use models::id::PasteId;
use models::paste::{Metadata, Visibility};
use models::paste::output::{Output, OutputFile, OutputAuthor};
use models::status::{Status, ErrorKind};
use routes::{RouteResult, OptionalUser};

use diesel::prelude::*;

use rocket::http::Status as HttpStatus;

use std::cmp::max;

#[derive(Debug, Serialize)]
struct AllPaste {
  id: PasteId,
  #[serde(flatten)]
  metadata: Metadata,
}

#[derive(Debug, FromForm)]
struct AllQuery {
  limit: Option<u8>,
}

#[get("/?<query>")]
fn get_all_query(query: AllQuery, conn: DbConn) -> RouteResult<Vec<AllPaste>> {
  _get_all(Some(query), conn)
}

#[get("/")]
fn get_all(conn: DbConn) -> RouteResult<Vec<AllPaste>> {
  _get_all(None, conn)
}

fn _get_all(query: Option<AllQuery>, conn: DbConn) -> RouteResult<Vec<AllPaste>> {
  let limit = max(100, query.and_then(|x| x.limit).unwrap_or(5));

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
        created_at: Some(x.created_at()),
      },
    })
    .collect();

  Ok(Status::show_success(HttpStatus::Ok, output))
}

#[derive(Debug, Default, FromForm)]
struct Query {
  full: Option<bool>,
}

// routes separated because of https://github.com/SergioBenitez/Rocket/issues/376

#[get("/<id>")]
fn get(id: PasteId, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  _get(id, None, user, conn)
}

#[get("/<id>?<query>")]
fn get_query(id: PasteId, query: Query, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  _get(id, Some(query), user, conn)
}

fn _get(id: PasteId, query: Option<Query>, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  let paste = match id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Status::show_error(status, kind));
  }
  let query = query.unwrap_or_default();

  let full = query.full == Some(true);
  let files: Vec<OutputFile> = id.files(&conn)?
    .iter()
    .map(|x| x.as_output_file(full, &paste))
    .collect::<Result<_, _>>()?;

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
    None,
    files,
  );

  Ok(Status::show_success(HttpStatus::Ok, output))
}
