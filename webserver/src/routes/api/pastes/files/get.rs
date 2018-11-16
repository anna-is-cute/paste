use crate::{
  config::Config,
  database::DbConn,
  models::{
    id::PasteId,
    paste::output::OutputFile,
    status::{Status, ErrorKind},
  },
  routes::{RouteResult, OptionalUser},
};

use rocket::{http::Status as HttpStatus, State};

#[get("/<paste_id>/files")]
pub fn get(paste_id: PasteId, user: OptionalUser, conn: DbConn, config: State<Config>) -> RouteResult<Vec<OutputFile>> {
  let paste = match paste_id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Status::show_error(status, kind));
  }

  let files: Vec<OutputFile> = paste_id.output_files(&*config, &conn, &paste, true)?;

  Ok(Status::show_success(HttpStatus::Ok, files))
}
