use database::DbConn;
use models::id::PasteId;
use models::paste::PasteFile;
use models::status::{Status, ErrorKind};
use routes::{RouteResult, RequiredUser};

use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

type UpdateResult = ::std::result::Result<Json<PasteFile>, ::rocket_contrib::SerdeError>;

#[post("/<paste_id>/files", format = "application/json", data = "<file>")]
pub fn post(paste_id: PasteId, file: UpdateResult, user: RequiredUser, conn: DbConn) -> RouteResult<()> {
  // TODO: can this be a request guard?
  let file = match file {
    Ok(x) => x.into_inner(),
    Err(e) => {
      let message = format!("could not parse json: {}", e);
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some(message))));
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

  paste_id.create_file(&conn, file.name, file.content)?;

  // commit
  // TODO: more descriptive commit message
  paste_id.commit(user.name(), user.email(), "update paste")?;

  Ok(Status::show_success(HttpStatus::NoContent, ()))
}
