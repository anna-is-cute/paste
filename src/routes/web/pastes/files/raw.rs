use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::models::users::User;
use database::schema::users;
use errors::*;
use models::id::{PasteId, FileId};
use routes::web::{Rst, OptionalWebUser};

use diesel::prelude::*;

use rocket::http::Status as HttpStatus;
use rocket::request::Request;
use rocket::response::{Responder, Response};

use std::fs::File;
use std::result;

enum Rstf {
  Rst(Rst),
  File(File),
}

impl<'r> Responder<'r> for Rstf {
  fn respond_to(self, request: &Request) -> result::Result<Response<'r>, HttpStatus> {
    match self {
      Rstf::Rst(r) => r.respond_to(request),
      Rstf::File(f) => f.respond_to(request),
    }
  }
}


#[get("/pastes/<username>/<paste_id>/files/<file_id>/raw")]
fn get(username: String, paste_id: PasteId, file_id: FileId, user: OptionalWebUser, conn: DbConn) -> Result<Rstf> {
  let paste: DbPaste = match paste_id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Rstf::Rst(Rst::Status(HttpStatus::NotFound))),
  };

  let expected_username: String = match paste.author_id() {
    Some(author) => {
      let user: User = users::table.find(author).first(&*conn)?;
      user.username().to_string()
    },
    None => "anonymous".into(),
  };

  if username != expected_username {
    return Ok(Rstf::Rst(Rst::Status(HttpStatus::NotFound)));
  }

  if let Some((status, _)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Rstf::Rst(Rst::Status(status)));
  }

  let file = match paste_id.file(&conn, file_id)? {
    Some(f) => f,
    None => return Ok(Rstf::Rst(Rst::Status(HttpStatus::NotFound))),
  };

  Ok(Rstf::File(File::open(file.path())?))
}
