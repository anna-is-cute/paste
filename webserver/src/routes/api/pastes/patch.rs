use crate::{
  database::DbConn,
  models::{
    id::PasteId,
    paste::update::MetadataUpdate,
    status::{Status, ErrorKind},
  },
  routes::{RouteResult, RequiredUser},
};

use rocket::{http::Status as HttpStatus, State};

use rocket_contrib::json::{Json, JsonError};

use sidekiq::Client as SidekiqClient;

type UpdateResult<'a> = ::std::result::Result<Json<MetadataUpdate>, JsonError<'a>>;

#[patch("/<paste_id>", format = "application/json", data = "<info>")]
pub fn patch(paste_id: PasteId, info: UpdateResult<'a>, user: RequiredUser, conn: DbConn, sidekiq: State<SidekiqClient>) -> RouteResult<()> {
  // TODO: can this be a request guard?
  let info = match info {
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
  let mut paste = match paste_id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };
  if let Some((status, kind)) = paste.check_access(Some(user.id())) {
    return Ok(Status::show_error(status, kind));
  }

  // update paste and database if necessary
  paste.update(&conn, &*sidekiq, &info)?;

  // return status (204?)
  Ok(Status::show_success(HttpStatus::NoContent, ()))
}
