use database::DbConn;
use database::models::pastes::Paste;
use database::schema::pastes;
use models::paste::{PasteId, Visibility};
use models::status::{Status, ErrorKind};
use routes::{RouteResult, DeletionAuth};

use diesel;
use diesel::prelude::*;

use rocket::http::Status as HttpStatus;

use std::fs;

#[delete("/<id>")]
fn delete(id: PasteId, auth: DeletionAuth, conn: DbConn) -> RouteResult<()> {
  let paste = match id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };
  match *paste.author_id() {
    Some(ref author_id) => {
      let is_private = paste.visibility() == Visibility::Private;
      let user = match auth {
        DeletionAuth::User(ref u) => u,
        _ if is_private => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
        _ => return Ok(Status::show_error(HttpStatus::Forbidden, ErrorKind::NotAllowed)),
      };
      if user.id() != *author_id {
        if is_private {
          return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste));
        }
        return Ok(Status::show_error(HttpStatus::Forbidden, ErrorKind::NotAllowed));
      }
    },
    None => {
      let deletion_key = match auth {
        DeletionAuth::Key(ref d) => d,
        _ => return Ok(Status::show_error(HttpStatus::Forbidden, ErrorKind::NotAllowed)),
      };
      if deletion_key.paste_id() != *id {
        return Ok(Status::show_error(HttpStatus::Forbidden, ErrorKind::NotAllowed));
      }
    },
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
