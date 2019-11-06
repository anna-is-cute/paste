use crate::{
  config::Config,
  database::{
    DbConn,
    schema::users,
    models::users::User,
  },
  errors::*,
  models::id::UserId,
  redis_store::Redis,
  utils::webp,
};

use diesel::prelude::*;

use redis::Commands;

use reqwest::{Client, RedirectPolicy, StatusCode};

use rocket::{
  Outcome,
  State,
  http::{
    ContentType, Header, Status,
    hyper::header::{CacheControl, CacheDirective},
  },
  request::{self, FromRequest, Request},
  response::Response,
};

use url::Url;

use std::io::{Cursor, Read};

#[derive(Responder)]
pub enum Avatar<'r> {
  Avatar(Response<'r>),
  Status(Status),
  #[response(status = 304)]
  NotModified(()),
}

#[get("/account/avatar/<id>")]
pub fn get<'r>(id: UserId, config: State<Config>, if_mod: IfMod, conn: DbConn, mut redis: Redis) -> Result<Avatar<'r>> {
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
  const EMPTY: &[u8] = &[];
  const CACHE_TIME: usize = 600;
  const WEBP_QUALITY: f32 = 90.0;

  fn webp_response<'r>(bytes: Vec<u8>) -> Response<'r> {
    Response::build()
      .header(ContentType::WEBP)
      .header(CacheControl(vec![
        CacheDirective::Public,
        CacheDirective::MaxAge(CACHE_TIME as u32),
      ]))
      .sized_body(Cursor::new(bytes))
      .status(Status::Ok)
      .finalize()
  }

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
  let redis_key = format!("avatar:{}:{}", domain, hash);

  let attempt_convert = if config.read().general.convert_avatars {
    let bytes: Option<Vec<u8>> = redis.get(&redis_key)?;

    match bytes {
      Some(bytes) if bytes.is_empty() => false,
      None => true,
      Some(bytes) => return Ok(Avatar::Avatar(webp_response(bytes))),
    }
  } else {
    false
  };

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
  let mut resp = req.send()?;

  // if not modified, return not modified
  if resp.status() == StatusCode::NOT_MODIFIED {
    return Ok(Avatar::NotModified(()));
  }

  if attempt_convert {
    // allocate a buffer for the image
    let len = resp.content_length()
      .map(|l| std::cmp::min(l, 250 * 1_000) as usize)
      .unwrap_or(125 * 1_000);
    let mut bytes = Vec::with_capacity(len);
    // read in the response
    resp.read_to_end(&mut bytes)?;
    // attempt to parse the image and convert it
    let image = image::load_from_memory(&bytes)
      .ok()
      .and_then(|i| webp::convert(&i, WEBP_QUALITY));
    match image {
      // if successful and the result is smaller than the original image, cache it and return it
      Some(webp_bytes) if webp_bytes.len() < bytes.len() => {
        redis.set_ex(&redis_key, &*webp_bytes, CACHE_TIME)?;
        return Ok(Avatar::Avatar(webp_response(webp_bytes)));
      },
      // otherwise, mark the avatar as "do not convert" for the cache time
      _ => redis.set_ex(&redis_key, EMPTY, CACHE_TIME)?,
    }
  }

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
