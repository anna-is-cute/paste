use crate::{
  config::Config,
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

use rocket::{http::Status as HttpStatus, request::Form, State};

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
pub fn get_all(query: Option<Form<AllQuery>>, conn: DbConn, config: State<Config>) -> RouteResult<Vec<AllPaste>> {
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
        updated_at: x.updated_at(&*config).ok(),
      },
    })
    .collect();

  Ok(Status::show_success(HttpStatus::Ok, output))
}

#[derive(Debug, Default, FromForm)]
pub struct Full {
  full: Option<bool>,
}

#[get("/<id>?<query..>")]
pub fn get(id: PasteId, query: Option<Form<Full>>, user: OptionalUser, conn: DbConn, config: State<Config>) -> RouteResult<Output> {
  let paste = match id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Status::show_error(status, kind));
  }
  let query = query.map(|x| x.into_inner()).unwrap_or_default();

  let full = query.full == Some(true);
  let files: Vec<OutputFile> = id.output_files(&*config, &conn, &paste, full)?;

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
    paste.updated_at(&*config).ok(), // FIXME
    paste.expires(),
    None,
    files,
  );

  Ok(Status::show_success(HttpStatus::Ok, output))
}
