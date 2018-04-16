use database::DbConn;
use database::models::deletion_keys::DeletionKey;
use database::models::pastes::Paste;
use database::models::users::User;
use database::schema::pastes;
use models::paste::{PasteId, Visibility};
use models::status::{Status, ErrorKind};
use routes::{RouteResult, DeletionAuth};

use diesel;
use diesel::prelude::*;

use rocket::http::Status as HttpStatus;

use uuid::Uuid;

use std::fs;

#[delete("/<id>")]
fn delete(id: PasteId, auth: DeletionAuth, conn: DbConn) -> RouteResult<()> {
  let paste = match id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };
  if let Some((status, kind)) = check_deletion(&paste, auth) {
    return Ok(Status::show_error(status, kind));
  }
  // should be validated beyond this point

  // remove files
  fs::remove_dir_all(id.directory())?;
  // remove database entry
  diesel::delete(&paste).execute(&*conn)?;

  // FIXME:
  // Error: Failed to write response: Custom { kind: WriteZero, error: StringError("failed to write
  // whole buffer") }.
  Ok(Status::show_success(HttpStatus::NoContent, ()))
}

fn check_deletion(paste: &Paste, auth: DeletionAuth) -> Option<(HttpStatus, ErrorKind)> {
  let author_id = paste.author_id();
  if_chain! {
    if let DeletionAuth::Key(ref key) = auth;
    if author_id.is_none();
    then {
      return check_deletion_key(paste, key);
    }
  }
  if_chain! {
    if let DeletionAuth::User(ref user) = auth;
    if let Some(id) = author_id;
    then {
      return check_deletion_user(paste, user, *id);
    }
  }

  None
}

fn check_deletion_user(paste: &Paste, user: &User, author_id: Uuid) -> Option<(HttpStatus, ErrorKind)> {
  if user.id() == author_id {
    return None;
  }
  if paste.visibility() == Visibility::Private {
    return Some((HttpStatus::NotFound, ErrorKind::MissingPaste));
  }
  Some((HttpStatus::Forbidden, ErrorKind::NotAllowed))
}

fn check_deletion_key(paste: &Paste, key: &DeletionKey) -> Option<(HttpStatus, ErrorKind)> {
  if paste.id() == key.paste_id() {
    return None;
  }
  Some((HttpStatus::Forbidden, ErrorKind::NotAllowed))
}
