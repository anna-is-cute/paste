use backend::errors::BackendError;
use backend::pastes::*;
use database::DbConn;
use models::paste::Paste;
use models::paste::output::{Output, OutputFile, OutputAuthor};
use models::status::{Status, ErrorKind};
use routes::{RouteResult, OptionalUser};

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

  // check that file names are not the empty string
  if info.files.iter().filter_map(|x| x.name.as_ref()).any(|x| x.is_empty()) {
    return Ok(Status::show_error(
      HttpStatus::BadRequest,
      ErrorKind::InvalidFile(Some("names cannot be empty (for no name, omit the name field)".into())),
    ));
  }

  let files = info.files
    .into_iter()
    .map(|f| FilePayload {
      name: f.name.map(|x| x.into_inner()),
      content: f.content,
    })
    .collect();

  let pp = PastePayload {
    name: info.metadata.name.map(|x| x.into_inner()),
    description: info.metadata.description.map(|x| x.into_inner()),
    visibility: info.metadata.visibility,
    author: user.as_ref(),
    files,
  };

  let CreateSuccess { paste, files, deletion_key } = match pp.create(&conn) {
    Ok(s) => s,
    Err(e) => {
      let msg = e.into_message()?;
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some(msg.into()))));
    },
  };

  match *user {
    Some(ref u) => paste.commit(u.name(), u.email(), "create paste")?,
    None => paste.commit("Anonymous", "none", "create paste")?,
  }

  // TODO: eventually replace this all with a GET /pastes/<id>?full=true backend call
  let files: Vec<OutputFile> = files
    .into_iter()
    .map(|x| OutputFile::new(x.id(), Some(x.name()), None))
    .collect();

  let author = match *user {
    Some(ref user) => Some(OutputAuthor::new(user.id(), user.username(), user.name())),
    None => None,
  };

  let output = Output::new(
    paste.id(),
    author,
    paste.name(),
    paste.description(),
    paste.visibility(),
    deletion_key.map(|x| x.key()),
    files,
  );

  Ok(Status::show_success(HttpStatus::Created, output))
}
