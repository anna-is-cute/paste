use crate::{
  database::{
    DbConn,
    models::{
      deletion_keys::DeletionKey,
      pastes::Paste,
      users::User,
    },
  },
  models::{
    id::{PasteId, UserId},
    paste::Visibility,
    status::{Status, ErrorKind},
  },
  routes::{RouteResult, RequiredUser, DeletionAuth},
};

use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

#[delete("/<id>", rank = 1)]
fn delete(id: PasteId, auth: DeletionAuth, conn: DbConn) -> RouteResult<()> {
  let paste = match id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };
  if let Some((status, kind)) = check_deletion(&paste, auth) {
    return Ok(Status::show_error(status, kind));
  }
  // should be validated beyond this point

  paste.delete(&conn)?;

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
      return check_deletion_user(paste, user, id);
    }
  }

  None
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

fn check_deletion_key(paste: &Paste, key: &DeletionKey) -> Option<(HttpStatus, ErrorKind)> {
  if paste.id() == key.paste_id() {
    return None;
  }
  Some((HttpStatus::Forbidden, ErrorKind::NotAllowed))
}

#[delete("/ids", format = "application/json", data = "<info>", rank = 2)]
fn ids(info: Json<Vec<PasteId>>, user: RequiredUser, conn: DbConn) -> RouteResult<()> {
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
    paste.delete(&conn)?;
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
