use database::DbConn;
use database::models::files::File as DbFile;
use database::schema::files;
use models::id::PasteId;
use models::paste::output::OutputFile;
use models::status::{Status, ErrorKind};
use routes::{RouteResult, OptionalUser};

use diesel::prelude::*;

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

  // TODO: refactor into PasteId::get_file
  let db_file: Option<DbFile> = DbFile::belonging_to(&paste)
    .filter(files::id.eq(*file_id))
    .first(&*conn)
    .optional()?;
  let db_file = match db_file {
    Some(f) => f,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingFile)),
  };

  let pf = db_file.as_output_file(true)?;

  Ok(Status::show_success(HttpStatus::Ok, pf))
}
