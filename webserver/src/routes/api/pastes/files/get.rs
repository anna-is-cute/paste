use crate::{
  database::DbConn,
  models::{
    id::PasteId,
    paste::output::OutputFile,
    status::{Status, ErrorKind},
  },
  routes::{RouteResult, OptionalUser},
};

use rocket::http::Status as HttpStatus;

#[get("/<paste_id>/files")]
fn get(paste_id: PasteId, user: OptionalUser, conn: DbConn) -> RouteResult<Vec<OutputFile>> {
  let paste = match paste_id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Status::show_error(status, kind));
  }

  let mut files: Vec<OutputFile> = paste_id.files(&conn)?
    .into_iter()
    .map(|f| f.as_output_file(false, &paste))
    .collect::<Result<_, _>>()?;

  files.sort_unstable_by(|a, b| a.name.cmp(&b.name));

  Ok(Status::show_success(HttpStatus::Ok, files))
}
