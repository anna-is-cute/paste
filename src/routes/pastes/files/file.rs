use models::paste::{PasteId, Content};
use models::paste::output::OutputFile;
use models::status::{Status, ErrorKind};
use routes::RouteResult;

use rocket_contrib::UUID;

use rocket::http::Status as HttpStatus;

use std::fs::File;
use std::io::Read;

#[get("/<paste_id>/files/<file_id>")]
fn get_file_id(paste_id: PasteId, file_id: UUID) -> RouteResult<OutputFile> {
  if !paste_id.exists() {
    return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste));
  }
  let files_dir = paste_id.files_directory();

  let metadata = paste_id.metadata()?; // FIXME: check if private (request guard?)
  let internal = paste_id.internal()?;

  let name = match internal.names.iter().find(|(u, _)| u == &*file_id) {
    Some((_, name)) => name,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingFile)),
  };

  let file_path = files_dir.join(file_id.simple().to_string());
  let mut file = File::open(file_path)?;
  let mut data = Vec::new();
  file.read_to_end(&mut data)?;

  // TODO: store if the file is text or binary instead of attempting to parse
  let content = String::from_utf8(data.clone())
    .map(Content::Text)
    .unwrap_or_else(|_| Content::Base64(data));

  let pf = OutputFile::new(&file_id, Some(name.clone()), Some(content));

  Ok(Status::show_success(HttpStatus::Ok, pf))
}
