use database::DbConn;
use errors::*;
use models::id::{PasteId, FileId};
use models::status::{Status, ErrorKind};
use routes::OptionalUser;

use rocket::http::Status as HttpStatus;
use rocket::request::Request;
use rocket::response::{Responder, Response, NamedFile};
use rocket::response::status::Custom;

use rocket_contrib::Json;

use std::result;

#[get("/<paste_id>/files/<file_id>/raw")]
fn get(paste_id: PasteId, file_id: FileId, user: OptionalUser, conn: DbConn) -> Result<FileOrError> {
  let paste = match paste_id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(FileOrError::Error(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste))),
  };

  if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(FileOrError::Error(Status::show_error(status, kind)));
  }

  let path = paste.files_directory().join(file_id.simple().to_string());

  // TODO: specials headers?
  Ok(FileOrError::File(NamedFile::open(path)?))
}

enum FileOrError {
  File(NamedFile),
  Error(Custom<Json<Status<()>>>),
}

impl<'r> Responder<'r> for FileOrError {
  fn respond_to(self, request: &Request) -> result::Result<Response<'r>, HttpStatus> {
    match self {
      FileOrError::File(f) => f.respond_to(request),
      FileOrError::Error(e) => e.respond_to(request),
    }
  }
}
