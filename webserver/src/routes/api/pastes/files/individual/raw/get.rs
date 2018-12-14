use crate::{
  config::Config,
  database::DbConn,
  errors::*,
  models::{
    id::{PasteId, FileId},
    status::{Status, ErrorKind},
  },
  routes::OptionalUser,
};

use rocket::{
  http::Status as HttpStatus,
  request::State,
  response::{
    NamedFile,
    status::Custom,
  },
};

use rocket_contrib::json::Json;

#[get("/<paste_id>/files/<file_id>/raw")]
pub fn get(paste_id: PasteId, file_id: FileId, user: OptionalUser, conn: DbConn, config: State<Config>,) -> Result<FileOrError> {
  let paste = match paste_id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(FileOrError::Error(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste))),
  };

  if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(FileOrError::Error(Status::show_error(status, kind)));
  }

  let path = paste.files_directory(&*config).join(file_id.to_simple().to_string());

  // TODO: specials headers?
  Ok(FileOrError::File(NamedFile::open(path)?))
}

#[derive(Responder)]
pub enum FileOrError {
  File(NamedFile),
  Error(Custom<Json<Status<()>>>),
}
