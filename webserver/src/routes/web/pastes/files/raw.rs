use crate::{
  database::{
    DbConn,
    models::{pastes::Paste as DbPaste, users::User},
    schema::users,
  },
  errors::*,
  models::id::{PasteId, FileId},
  routes::{AddHeaders, web::OptionalWebUser},
};

use diesel::prelude::*;

use rocket::{
  http::{Status as HttpStatus},
  request::Request,
  response::{Responder, Response},
};

use std::{fs::File, result};

pub enum As {
  Add(AddHeaders<File>),
  Status(HttpStatus),
}

impl Responder<'r> for As {
  fn respond_to(self, request: &Request) -> result::Result<Response<'r>, HttpStatus> {
    match self {
      As::Add(r) => r.respond_to(request),
      As::Status(r) => Err(r),
    }
  }
}


#[get("/p/<username>/<paste_id>/files/<file_id>/raw")]
pub fn get(username: String, paste_id: PasteId, file_id: FileId, user: OptionalWebUser, conn: DbConn) -> Result<As> {
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
