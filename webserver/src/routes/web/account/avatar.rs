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

use reqwest::{Client, RedirectPolicy, StatusCode};

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
pub fn get<'r>(id: UserId, if_mod: IfMod, conn: DbConn) -> Result<Avatar<'r>> {
  lazy_static! {
    pub static ref CLIENT: Client = Client::builder()
      // do not allow redirects
      .redirect(RedirectPolicy::none())
      .build()
      .expect("could not build client");
  }

  // headers to forward
  const HEADERS: &[&str] = &[
    "Content-Type", "Content-Length", "Cache-Control", "Expires", "Last-Modified",
  ];

  // get the user referenced by the given id
  let user: Option<User> = users::table.find(id).first(&*conn).optional()?;
  let user = match user {
    Some(u) => u,
    None => return Ok(Avatar::Status(Status::NotFound)),
  };

  // find the domain and port to be used to get the avatar
  let (domain, port) = user.avatar_provider().domain(user.email());
  // hash the user's email with the service's hash algo
  let hash = user.avatar_provider().hash(user.email());

  // create a url from the given host, port, and hash (256px and default to identicons)
  let mut url = Url::parse("https://example.com/avatar/")?.join(&hash)?;
  url.set_host(Some(&domain))?;
  url.set_port(Some(port)).expect("cannot fail to set port");
  url.query_pairs_mut()
    .append_pair("s", "256")
    .append_pair("d", "identicon");

  // use the custom no-redirect client to request the url
  let mut req = CLIENT.get(url.as_str());
  // include If-Modified-Since if specified
  if let IfMod(Some(s)) = if_mod {
    req = req.header("If-Modified-Since", s);
  }
  // send the request
  let resp = req.send()?;

  // if not modified, return not modified
  if resp.status() == StatusCode::NOT_MODIFIED {
    return Ok(Avatar::NotModified(()));
  }

  // create our image response
  let mut builder = Response::build();

  // forward the allowed headers
  for &name in HEADERS {
    // get the header or skip it if not present
    let h = match resp.headers().get(name) {
      Some(h) => h,
      None => continue,
    };

    // convert the value to a string
    let value = h.to_str()?;
    // only forward Content-Type if it's an image
    if name == "Content-Type" {
      if !value.starts_with("image/") {
        // if it's not an image Content-Type, set it to application/octet-stream
        builder.header(Header::new("Content-Type", "application/octet-stream"));
        continue;
      }
    }
    // build a new header with the value
    let h = Header::new(name, value.to_string());
    // add it to the response
    builder.header(h);
  }

  // add Content-Disposition: attachment to force download
  builder.header(Header::new("Content-Disposition", "attachment"));

  // get the status of our request
  let resp_status = resp.status();

  builder
    // stream the response
    .streamed_body(resp)
    // set the status to what we received
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
