use models::paste::PasteId;
use models::paste::output::OutputFile;
use models::status::{Status, ErrorKind};
use routes::RouteResult;

use rocket::http::Status as HttpStatus;

#[get("/<paste_id>/files")]
fn get_files(paste_id: PasteId) -> RouteResult<Vec<OutputFile>> {
  if !paste_id.exists() {
    return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste));
  }

  let metadata = paste_id.metadata()?; // FIXME: check if private
  let internal = paste_id.internal()?;

  let files: Vec<OutputFile> = internal.names
    .iter()
    .map(|(uuid, name)| OutputFile::new(uuid, Some(name.clone()), None))
    .collect();

  Ok(Status::show_success(HttpStatus::Ok, files))
}
