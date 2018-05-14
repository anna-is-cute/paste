use database::{DbConn, schema};
use database::models::deletion_keys::NewDeletionKey;
use database::models::files::File as DbFile;
use database::models::pastes::{Paste as DbPaste, NewPaste};
use models::paste::Paste;
use models::paste::output::{Output, OutputFile, OutputAuthor};
use models::status::{Status, ErrorKind};
use routes::{RouteResult, OptionalUser};
use store::Store;

use diesel;
use diesel::prelude::*;

use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

type InfoResult = ::std::result::Result<Json<Paste>, ::rocket_contrib::SerdeError>;

#[post("/", format = "application/json", data = "<info>")]
fn post(info: InfoResult, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  // TODO: can this be a request guard?
  let info = match info {
    Ok(x) => x.into_inner(),
    Err(e) => {
      let message = format!("could not parse json: {}", e);
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some(message))));
    },
  };

  // check that files are valid
  // move validate_files to Paste?
  if let Err(e) = Store::validate_files(&info.files) {
    return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::InvalidFile(Some(e))));
  }
  // move this to PasteId::create?
  // rocket has already verified the paste info is valid, so create a paste
  let id = Store::new_paste(user.as_ref().map(|x| x.id()))?;

  // TODO: refactor
  let np = NewPaste::new(
    id,
    info.metadata.name.as_ref().map(|x| x.to_string()),
    info.metadata.description.as_ref().map(|x| x.to_string()),
    info.metadata.visibility,
    user.as_ref().map(|x| x.id()),
    None,
  );
  let paste: DbPaste = diesel::insert_into(schema::pastes::table)
    .values(&np)
    .get_result(&*conn)?;

  // TODO: refactor
  let deletion_key = if user.is_none() {
    let key = NewDeletionKey::generate(id);
    diesel::insert_into(schema::deletion_keys::table)
      .values(&key)
      .execute(&*conn)?;
    Some(key.key())
  } else {
    None
  };

  let files: Vec<DbFile> = info.files
    .into_iter()
    .map(|x| paste.create_file(&conn, x.name.map(|x| x.to_string()), x.content))
    .collect::<Result<_, _>>()?;

  match *user {
    Some(ref u) => paste.commit(u.name(), u.email(), "create paste")?,
    None => paste.commit("Anonymous", "none", "create paste")?,
  }

  let files: Vec<OutputFile> = files
    .into_iter()
    .map(|x| x.as_output_file(false, &paste))
    .collect::<Result<_, _>>()?;

  let author = match *user {
    Some(ref user) => Some(OutputAuthor::new(user.id(), user.username())),
    None => None,
  };

  let output = Output::new(
    id,
    author,
    info.metadata.name.as_ref().map(ToString::to_string),
    info.metadata.description.as_ref().map(ToString::to_string),
    info.metadata.visibility,
    deletion_key,
    files,
  );

  Ok(Status::show_success(HttpStatus::Created, output))
}
