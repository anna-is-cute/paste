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

use r2d2_redis::redis::Commands;

use reqwest::blocking::Client;

use rocket::{
  http::{
    ContentType, Header, Status,
    hyper::header::{CacheControl, CacheDirective},
  },
  response::Response,
  State,
};

use std::io::{Cursor, Read};

#[derive(Responder)]
pub enum Avatar<'r> {
  Avatar(Response<'r>),
  Status(Status),
  #[response(status = 304)]
  NotModified(()),
}

#[get("/account/avatar/<id>")]
pub fn get<'r>(id: UserId, config: State<Config>, client: State<Client>, if_mod: IfMod, conn: DbConn, mut redis: Redis) -> Result<Avatar<'r>> {
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

  let user: Option<User> = users::table.find(id).first(&*conn).optional()?;
  let user = match user {
    Some(u) => u,
    None => return Ok(Avatar::Status(Status::NotFound)),
  };

  let domain = user.avatar_provider().domain();
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

  let url = format!("https://{}/avatar/{}?s=256&d=identicon", domain, hash);
  let mut req = client.get(&url);
  if let IfMod(Some(s)) = if_mod {
    req = req.header("If-Modified-Since", s);
  }
  let mut resp = req.send()?;

  if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
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
