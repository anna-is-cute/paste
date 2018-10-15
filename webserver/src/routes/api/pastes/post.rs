use crate::{
  backend::{
    errors::BackendError,
    pastes::*,
  },
  database::DbConn,
  models::{
    paste::{
      Paste,
      output::{Output, OutputFile, OutputAuthor}
    },
    status::{Status, ErrorKind},
  },
  routes::{RouteResult, OptionalUser},
};

use rocket::{State, http::Status as HttpStatus};

use rocket_contrib::Json;

use sidekiq::Client as SidekiqClient;

type InfoResult = ::std::result::Result<Json<Paste>, ::rocket_contrib::SerdeError>;

#[post("/", format = "application/json", data = "<info>")]
fn post(info: InfoResult, user: OptionalUser, conn: DbConn, sidekiq: State<SidekiqClient>) -> RouteResult<Output> {
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
      highlight_language: f.highlight_language,
      content: f.content,
    })
    .collect();

  let pp = PastePayload {
    name: info.metadata.name.map(|x| x.into_inner()),
    description: info.metadata.description.map(|x| x.into_inner()),
    visibility: info.metadata.visibility,
    expires: info.metadata.expires,
    author: user.as_ref(),
    files,
  };

  let CreateSuccess { paste, files, deletion_key } = match pp.create(&conn, &*sidekiq) {
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

  // TODO: eventually replace this all with a GET /p/<id>?full=true backend call
  let mut files: Vec<OutputFile> = files
    .into_iter()
    .map(|x| OutputFile::new(x.id(), Some(x.name()), x.highlight_language(), None))
    .collect();

  files.sort_unstable_by(|a, b| a.name.cmp(&b.name));

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
    paste.created_at(),
    paste.updated_at().ok(), // FIXME
    paste.expires(),
    deletion_key.map(|x| x.key()),
    files,
  );

  Ok(Status::show_success(HttpStatus::Created, output))
}
