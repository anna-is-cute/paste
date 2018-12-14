use crate::{
  config::Config,
  database::DbConn,
  models::id::PasteId,
  models::paste::PasteFile,
  models::paste::output::OutputFile,
  models::status::{Status, ErrorKind},
  routes::{RouteResult, RequiredUser},
};

use rocket::{http::Status as HttpStatus, State};

use rocket_contrib::json::{Json, JsonError};

type UpdateResult<'a> = ::std::result::Result<Json<PasteFile>, JsonError<'a>>;

#[post("/<paste_id>/files", format = "application/json", data = "<file>")]
pub fn post(paste_id: PasteId, file: UpdateResult<'a>, user: RequiredUser, conn: DbConn, config: State<Config>) -> RouteResult<OutputFile> {
  // TODO: can this be a request guard?
  let file = match file {
    Ok(x) => x.into_inner(),
    Err(e) => {
      let message = match e {
        JsonError::Io(_) => None,
        JsonError::Parse(_, e) => Some(format!("could not parse json: {}", e)),
      };
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(message)));
    },
  };
  // verify auth
  let paste = match paste_id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };
  if let Some((status, kind)) = paste.check_access(Some(user.id())) {
    return Ok(Status::show_error(status, kind));
  }

  let created = paste.create_file(
    &*config,
    &conn,
    file.name.map(|x| x.to_string()),
    file.highlight_language,
    file.content
  )?;

  // commit
  // TODO: more descriptive commit message
  paste.commit(&*config, user.name(), user.email(), "update paste")?;

  let output = OutputFile::new(created.id(), Some(created.name().to_string()), created.highlight_language(), None);

  Ok(Status::show_success(HttpStatus::Created, output))
}
