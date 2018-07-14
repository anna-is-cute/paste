use database::DbConn;
use models::id::PasteId;
use models::paste::update::MetadataUpdate;
use models::status::{Status, ErrorKind};
use routes::{RouteResult, RequiredUser};

use rocket::http::Status as HttpStatus;
use rocket::State;

use rocket_contrib::Json;

use sidekiq::Client as SidekiqClient;

type UpdateResult = ::std::result::Result<Json<MetadataUpdate>, ::rocket_contrib::SerdeError>;

#[patch("/<paste_id>", format = "application/json", data = "<info>")]
pub fn patch(paste_id: PasteId, info: UpdateResult, user: RequiredUser, conn: DbConn, sidekiq: State<SidekiqClient>) -> RouteResult<()> {
  // TODO: can this be a request guard?
  let info = match info {
    Ok(x) => x.into_inner(),
    Err(e) => {
      let message = format!("could not parse json: {}", e);
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some(message))));
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
