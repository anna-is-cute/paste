use database::DbConn;
use database::models::files::File as DbFile;
use database::models::pastes::Paste as DbPaste;
use database::schema::{pastes, files};
use models::paste::{PasteId, Visibility, Content};
use models::paste::output::OutputFile;
use models::status::{Status, ErrorKind};
use routes::{RouteResult, OptionalUser};

use diesel::prelude::*;

use rocket_contrib::UUID;

use rocket::http::Status as HttpStatus;

use std::fs::File;
use std::io::Read;

#[get("/<paste_id>/files/<file_id>")]
fn get_file_id(paste_id: PasteId, file_id: UUID, user: OptionalUser, conn: DbConn) -> RouteResult<OutputFile> {
  let paste: Option<DbPaste> = pastes::table.filter(pastes::id.eq(*paste_id)).first(&*conn).optional()?;
  let paste = match paste {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if paste.visibility() == Visibility::Private && user.as_ref().map(|x| x.id()) != *paste.author_id() {
    return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste));
  }

  let db_file: Option<DbFile> = DbFile::belonging_to(&paste)
    .filter(files::id.eq(*file_id))
    .first(&*conn)
    .optional()?;
  let db_file = match db_file {
    Some(f) => f,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingFile)),
  };

  let files_dir = paste_id.files_directory();

  let file_path = files_dir.join(db_file.id().simple().to_string());
  let mut file = File::open(file_path)?;
  let mut data = Vec::new();
  file.read_to_end(&mut data)?;

  // TODO: store if the file is text or binary instead of attempting to parse
  let content = String::from_utf8(data.clone())
    .map(Content::Text)
    .unwrap_or_else(|_| Content::Base64(data));

  let pf = OutputFile::new(&db_file.id(), Some(db_file.name().clone()), Some(content));

  Ok(Status::show_success(HttpStatus::Ok, pf))
}
