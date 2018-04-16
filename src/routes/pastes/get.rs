use database::DbConn;
use models::id::PasteId;
use models::paste::{Paste, Metadata};
use models::paste::output::{Output, OutputFile};
use models::status::{Status, ErrorKind};
use routes::{RouteResult, OptionalUser};

use rocket::http::Status as HttpStatus;

#[derive(Debug, Default, FromForm)]
struct Query {
  full: Option<bool>,
}

// routes separated because of https://github.com/SergioBenitez/Rocket/issues/376

#[get("/<id>")]
fn get(id: PasteId, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  _get(id, None, user, conn)
}

#[get("/<id>?<query>")]
fn get_query(id: PasteId, query: Query, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  _get(id, Some(query), user, conn)
}

fn _get(id: PasteId, query: Option<Query>, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  let paste = match id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Status::show_error(status, kind));
  }
  let query = query.unwrap_or_default();

  let full = query.full == Some(true);
  let files: Vec<OutputFile> = id.files(&conn)?
    .iter()
    .map(|x| x.as_output_file(full))
    .collect::<Result<_, _>>()?;

  let output = Output {
    id: (*id).into(),
    paste: Paste {
      metadata: Metadata {
        name: paste.name().clone(),
        visibility: paste.visibility(),
      },
      files: Vec::new(),
    },
    deletion_key: None,
    files,
  };

  Ok(Status::show_success(HttpStatus::Ok, output))
}
