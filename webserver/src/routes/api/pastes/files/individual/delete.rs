use crate::{
  config::Config,
  database::DbConn,
  models::{
    id::{PasteId, FileId},
    status::{Status, ErrorKind},
  },
  routes::{RouteResult, RequiredUser},
};

use rocket::{http::Status as HttpStatus, State};

#[delete("/<paste_id>/files/<file_id>")]
pub fn delete(paste_id: PasteId, file_id: FileId, user: RequiredUser, conn: DbConn, config: State<Config>) -> RouteResult<()> {
  let paste = match paste_id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if let Some((status, kind)) = paste.check_access(user.id()) {
    return Ok(Status::show_error(status, kind));
  }

  paste.delete_file(&*config, &conn, file_id)?;

  paste.commit_if_dirty(&*config, user.name(), user.email(), "delete file")?;

  Ok(Status::show_success(HttpStatus::NoContent, ()))
}
