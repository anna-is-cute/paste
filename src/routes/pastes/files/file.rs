use database::DbConn;
use models::id::PasteId;
use models::paste::output::OutputFile;
use models::status::{Status, ErrorKind};
use routes::{RouteResult, OptionalUser};

use rocket_contrib::UUID;

use rocket::http::Status as HttpStatus;

#[get("/<paste_id>/files/<file_id>")]
fn get_file_id(paste_id: PasteId, file_id: UUID, user: OptionalUser, conn: DbConn) -> RouteResult<OutputFile> {
  let paste = match paste_id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Status::show_error(status, kind));
  }

  let db_file = match paste_id.file(&conn, *file_id)? {
    Some(f) => f,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingFile)),
  };

  let pf = db_file.as_output_file(true)?;

  Ok(Status::show_success(HttpStatus::Ok, pf))
}
