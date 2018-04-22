use database::DbConn;
use models::id::{PasteId, FileId};
use models::status::{Status, ErrorKind};
use routes::{RouteResult, RequiredUser};

use rocket::http::Status as HttpStatus;

#[delete("/<paste_id>/files/<file_id>")]
fn delete(paste_id: PasteId, file_id: FileId, user: RequiredUser, conn: DbConn) -> RouteResult<()> {
  let paste = match paste_id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if let Some((status, kind)) = paste.check_access(user.id()) {
    return Ok(Status::show_error(status, kind));
  }

  paste_id.delete_file(&conn, *file_id)?;

  paste_id.commit_if_dirty(user.name(), user.email(), "delete file")?;

  Ok(Status::show_success(HttpStatus::NoContent, ()))
}
