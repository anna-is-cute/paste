use crate::{
  config::Config,
  database::{
    DbConn,
    models::{
      deletion_keys::{DeletionKey, SecretDeletionKey},
      pastes::Paste,
      users::User,
    },
  },
  errors::*,
  models::{
    id::{PasteId, UserId},
    paste::Visibility,
    status::{Status, ErrorKind},
  },
  routes::{RouteResult, RequiredUser, DeletionAuth},
};

use diesel::prelude::*;

use rocket::{
  request::State,
  http::Status as HttpStatus,
};

use rocket_contrib::json::Json;

#[delete("/<id>", rank = 1)]
pub fn delete(id: PasteId, auth: DeletionAuth, conn: DbConn, config: State<Config>) -> RouteResult<()> {
  let paste = match id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };
  if let Some((status, kind)) = check_deletion(&conn, &paste, auth)? {
    return Ok(Status::show_error(status, kind));
  }
  // should be validated beyond this point

  paste.delete(&*config, &conn)?;

  // FIXME:
  // Error: Failed to write response: Custom { kind: WriteZero, error: StringError("failed to write
  // whole buffer") }.
  Ok(Status::show_success(HttpStatus::NoContent, ()))
}

fn check_deletion(conn: &DbConn, paste: &Paste, auth: DeletionAuth) -> Result<Option<(HttpStatus, ErrorKind)>> {
  let author_id = paste.author_id();
  if_chain! {
    if let DeletionAuth::Key(ref key) = auth;
    if author_id.is_none();
    then {
      return check_deletion_key(conn, paste, key);
    }
  }
  if_chain! {
    if let DeletionAuth::User(ref user) = auth;
    if let Some(id) = author_id;
    then {
      return Ok(check_deletion_user(paste, user, id));
    }
  }

  Ok(None)
}

fn check_deletion_user(paste: &Paste, user: &User, author_id: UserId) -> Option<(HttpStatus, ErrorKind)> {
  if user.id() == author_id {
    return None;
  }
  if paste.visibility() == Visibility::Private {
    return Some((HttpStatus::NotFound, ErrorKind::MissingPaste));
  }
  Some((HttpStatus::Forbidden, ErrorKind::NotAllowed))
}

fn check_deletion_key(conn: &DbConn, paste: &Paste, key: &SecretDeletionKey) -> Result<Option<(HttpStatus, ErrorKind)>> {
  use crate::database::schema::deletion_keys;

  let real_key: DeletionKey = match deletion_keys::table.find(paste.id()).first(&**conn).optional()? {
    Some(key) => key,
    None => return Ok(Some((HttpStatus::Forbidden, ErrorKind::NotAllowed))),
  };
  if real_key.check_key(&key.uuid().to_simple().to_string()) {
    return Ok(None);
  }
  Ok(Some((HttpStatus::Forbidden, ErrorKind::NotAllowed)))
}

#[delete("/ids", format = "application/json", data = "<info>", rank = 2)]
pub fn ids(info: Json<Vec<PasteId>>, user: RequiredUser, conn: DbConn, config: State<Config>) -> RouteResult<()> {
  let ids = info.into_inner();

  if ids.len() > 15 {
    return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadParameters(Some("up to 15 pastes can be deleted at a time".into()))));
  }

  let mut pastes = Vec::with_capacity(ids.len());
  for id in ids {
    let paste = match id.get(&conn)? {
      Some(p) => p,
      None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
    };
    if let Some((status, kind)) = paste.check_access(Some(user.id())) {
      return Ok(Status::show_error(status, kind));
    }
    if paste.author_id() != Some(user.id()) {
      return Ok(Status::show_error(HttpStatus::Forbidden, ErrorKind::NotAllowed));
    }
    pastes.push(paste);
  }

  for paste in &pastes {
    paste.delete(&*config, &conn)?;
  }

  // FIXME:
  // Error: Failed to write response: Custom { kind: WriteZero, error: StringError("failed to write
  // whole buffer") }.
  Ok(Status::show_success(HttpStatus::NoContent, ()))
}

// #[derive(Debug, Deserialize)]
// struct MultiDelete {
//   ids: Vec<PasteId>,
// }
