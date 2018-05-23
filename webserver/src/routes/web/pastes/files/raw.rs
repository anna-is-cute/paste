use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::models::users::User;
use database::schema::users;
use errors::*;
use models::id::{PasteId, FileId};
use routes::web::OptionalWebUser;
use routes::AddHeaders;

use diesel::prelude::*;

use rocket::http::{Status as HttpStatus};
use rocket::request::Request;
use rocket::response::{Responder, Response};

use std::fs::File;
use std::result;

enum As {
  Add(AddHeaders<File>),
  Status(HttpStatus),
}

impl<'r> Responder<'r> for As {
  fn respond_to(self, request: &Request) -> result::Result<Response<'r>, HttpStatus> {
    match self {
      As::Add(r) => r.respond_to(request),
      As::Status(r) => Err(r),
    }
  }
}


#[get("/pastes/<username>/<paste_id>/files/<file_id>/raw")]
fn get(username: String, paste_id: PasteId, file_id: FileId, user: OptionalWebUser, conn: DbConn) -> Result<As> {
  let paste: DbPaste = match paste_id.get(&conn)? {
    Some(p) => p,
    None => return Ok(As::Status(HttpStatus::NotFound)),
  };

  let expected_username: String = match paste.author_id() {
    Some(author) => {
      let user: User = users::table.find(author).first(&*conn)?;
      user.username().to_string()
    },
    None => "anonymous".into(),
  };

  if username != expected_username {
    return Ok(As::Status(HttpStatus::NotFound));
  }

  if let Some((status, _)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(As::Status(status));
  }

  let file = match paste_id.file(&conn, file_id)? {
    Some(f) => f,
    None => return Ok(As::Status(HttpStatus::NotFound)),
  };

  let h = if file.is_binary() == Some(true) {
    ("Content-Disposition".into(), "attachment".into())
  } else {
    ("Content-Type".into(), "text/plain; charset=utf-8".into())
  };

  Ok(As::Add(AddHeaders::new(
    File::open(file.path(&paste))?,
    vec![h],
  )))
}
