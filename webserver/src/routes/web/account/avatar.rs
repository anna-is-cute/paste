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
  #[response(status = 304)]
  NotModified(()),
}

#[get("/account/avatar/<id>")]
pub fn get<'r>(id: UserId, client: State<Client>, if_mod: IfMod, conn: DbConn) -> Result<Avatar<'r>> {
  const HEADERS: &[&str] = &[
    "Content-Type", "Content-Length", "Cache-Control", "Expires", "Last-Modified",
  ];

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
  let mut req = client.get(&url);
  if let IfMod(Some(s)) = if_mod {
    req = req.header("If-Modified-Since", s);
  }
  let resp = req.send()?;

  if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
    return Ok(Avatar::NotModified(()));
  }

  let mut builder = Response::build();

  for &name in HEADERS {
    if let Some(h) = resp.headers().get(name) {
      let h = Header::new(name, h.to_str()?.to_string());
      builder.header(h);
    }
  }

  let resp_status = resp.status();

  builder
    .streamed_body(resp)
    .raw_status(resp_status.as_u16(), resp_status.canonical_reason().unwrap_or(""));

  Ok(Avatar::Avatar(builder.finalize()))
}

pub struct IfMod(Option<String>);

impl rocket::request::FromRequest<'a, 'r> for IfMod {
  type Error = ();

  fn from_request(req: &'a rocket::Request<'r>) -> rocket::request::Outcome<Self, Self::Error> {
    match req.headers().get_one("If-Modified-Since") {
      Some(ref m) => rocket::Outcome::Success(IfMod(Some(m.to_string()))),
      None => rocket::Outcome::Success(IfMod(None)),
    }
  }
}
