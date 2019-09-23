use crate::{
  database::{
    DbConn,
    schema::users,
    models::users::User,
  },
  errors::*,
  models::id::UserId,
};

use diesel::prelude::*;

use reqwest::{Client, StatusCode};

use rocket::{
  Outcome,
  State,
  http::{Header, Status},
  request::{self, FromRequest, Request},
  response::Response,
};

use url::Url;

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

  let (domain, port) = user.avatar_provider().domain(user.email());
  let hash = user.avatar_provider().hash(user.email());

  let mut url = Url::parse("https://example.com/avatar/")?.join(&hash)?;
  url.set_host(Some(&domain))?;
  url.set_port(Some(port)).expect("cannot fail to set port");
  url.query_pairs_mut()
    .append_pair("s", "256")
    .append_pair("d", "identicon");

  let mut req = client.get(url.as_str());
  if let IfMod(Some(s)) = if_mod {
    req = req.header("If-Modified-Since", s);
  }
  let resp = req.send()?;

  if resp.status() == StatusCode::NOT_MODIFIED {
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

impl FromRequest<'a, 'r> for IfMod {
  type Error = ();

  fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    match req.headers().get_one("If-Modified-Since") {
      Some(ref m) => Outcome::Success(IfMod(Some(m.to_string()))),
      None => Outcome::Success(IfMod(None)),
    }
  }
}
