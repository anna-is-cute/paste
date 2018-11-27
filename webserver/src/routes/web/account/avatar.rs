use crate::{
  database::{
    DbConn,
    schema::users,
    models::users::User,
  },
  errors::*,
  models::id::UserId,
};

use crypto::{digest::Digest, md5::Md5};

use diesel::prelude::*;

use reqwest::Client;

use rocket::{
  http::{Header, Status},
  response::Response,
  State,
};

use std::cell::RefCell;

thread_local! {
  static MD5: RefCell<Md5> = RefCell::new(Md5::new());
}

#[derive(Responder)]
pub enum Avatar<'r> {
  Avatar(Response<'r>),
  Status(Status),
}

#[get("/account/avatar/<id>")]
pub fn get<'r>(id: UserId, client: State<Client>, conn: DbConn) -> Result<Avatar<'r>> {
  let user: Option<User> = users::table.find(id).first(&*conn).optional()?;
  let user = match user {
    Some(u) => u,
    None => return Ok(Avatar::Status(Status::NotFound)),
  };

  let hash = MD5.with(|m| {
    let mut m = m.borrow_mut();
    m.input_str(user.email().to_lowercase().trim());
    let hash = m.result_str();
    m.reset();

    hash
  });

  let url = format!("https://gravatar.com/avatar/{}?s=128&d=identicon", hash);
  let resp = client.get(&url).send()?;

  let mut builder = Response::build();

  if let Some(h) = resp.headers().get("Content-Type") {
    let h = Header::new("Content-Type", h.to_str()?.to_string());
    builder.header(h);
  }

  if let Some(h) = resp.headers().get("Content-Length") {
    let h = Header::new("Content-Length", h.to_str()?.to_string());
    builder.header(h);
  }

  builder.streamed_body(resp);

  Ok(Avatar::Avatar(builder.finalize()))
}
