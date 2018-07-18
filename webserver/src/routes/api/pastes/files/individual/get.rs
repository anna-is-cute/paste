use crate::{
  database::DbConn,
  models::{
    id::{PasteId, FileId},
    paste::output::OutputFile,
    status::{Status, ErrorKind},
  },
  routes::{RouteResult, OptionalUser},
};

use rocket::http::Status as HttpStatus;

#[get("/<paste_id>/files/<file_id>")]
fn get(paste_id: PasteId, file_id: FileId, user: OptionalUser, conn: DbConn) -> RouteResult<OutputFile> {
  let paste = match paste_id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Status::show_error(status, kind));
  }

  let db_file = match paste_id.file(&conn, file_id)? {
    Some(f) => f,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingFile)),
  };

  let pf = db_file.as_output_file(true, &paste)?;

  Ok(Status::show_success(HttpStatus::Ok, pf))
}
