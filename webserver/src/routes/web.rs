use crate::{
  config::Config,
  database::{
    PostgresPool,
    models::users::User,
    schema::users as users_db,
  },
  utils::AcceptLanguage,
};

use rocket::http::{Cookie, SameSite};

use data_encoding::BASE64;

use diesel::prelude::*;

use hashbrown::HashMap;

use rand::{
  Rng,
  distributions::{Alphanumeric, Distribution},
  seq::SliceRandom,
};

use rocket::{
  State, Outcome,
  http::{Header, Status as HttpStatus},
  request::{self, Request, FromRequest},
  response::{Responder, Response, Redirect},
};

use rocket_contrib::templates::Template;

use serde_json::{Value, json};

use sha2::{Digest, Sha384};

use uuid::Uuid;

use std::{ops::Deref, result};

macro_rules! links {
  ($links:expr, $($key:expr => $val:expr),+$(,)?) => {{
    let mut ls = $links;
    ls
      $(.add($key, $val))+;
    ls
  }};
  ($($key:expr => $val:expr),+$(,)?) => {{
    links!(crate::routes::web::Links::default(), $($key => $val),+)
  }};
}

pub mod about;
pub mod account;
pub mod admin;
pub mod auth;
pub mod credits;
pub mod fairings;
pub mod guards;
pub mod index;
pub mod pastes;
pub mod static_files;
pub mod users;

pub use self::fairings::*;
pub use self::guards::*;

#[derive(Serialize)]
pub struct Honeypot {
  class: String,
  css: String,
  integrity_hash: String,
}

impl Honeypot {
  pub fn new() -> Self {
    const ALPHA: [char; 52] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];

    let mut rng = rand::thread_rng();

    let length = rng.gen_range(15..20);

    let start = ALPHA.choose(&mut rng).unwrap();
    let end: String = Alphanumeric.sample_iter(&mut rng).map(char::from).take(length).collect();
    let class = format!("{}{}", start, end);

    let skip = rng.gen_range(1..4);

    let css = format!(
      "[class *= {}] {{ position: absolute; left: -100vw; width: 1px; height: 1px; }}",
      &class[..class.len() - skip],
    );

    let mut hasher = Sha384::new();
    hasher.update(&css);
    let integrity_hash = format!("sha384-{}", BASE64.encode(&hasher.finalize()[..]));

    Honeypot {
      class,
      css,
      integrity_hash,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AntiSpam<'a, 'r> where 'r: 'a {
  id: Uuid,
  #[serde(skip)]
  request: Option<&'a Request<'r>>,
  js: (String, String),
  no_js: (u8, u8, u8),
  script: String,
  integrity_hash: String,
}

impl<'a, 'r> AntiSpam<'a, 'r> {
  pub fn new(req: &'a Request<'r>) -> Self {
    let mut rng = rand::thread_rng();

    let length = rng.gen_range(7..10);

    let js_1: String = Alphanumeric.sample_iter(&mut rng).map(char::from).take(length).collect();
    let js_2: String = Alphanumeric.sample_iter(&mut rng).map(char::from).take(length).collect();

    let script = format!(
      "document.getElementById('js-check').value = '{}' + '{}';",
      js_1,
      js_2,
    );

    let mut hasher = Sha384::new();
    hasher.update(&script);
    let integrity_hash = format!("sha384-{}", BASE64.encode(&hasher.finalize()[..]));

    let x: u8 = rng.gen_range(1..10);
    let y: u8 = rng.gen_range(1..10);
    let sum = x + y;

    AntiSpam {
      id: Uuid::new_v4(),
      request: Some(req),
      js: (js_1, js_2),
      no_js: (x, y, sum),
      script,
      integrity_hash,
    }
  }
}


impl FromRequest<'a, 'r> for AntiSpam<'a, 'r> {
  type Error = String;

  fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    // get antispam (private cookie, so encrypted and authed)
    let antispam: Option<Self> = req
      .cookies()
      .get_private("antispam")
      .and_then(|x| serde_json::from_str(x.value()).ok());

    let antispam = match antispam {
      Some(mut s) => {
        s.request = Some(req);
        s
      },
      None => AntiSpam::new(req),
    };

    Outcome::Success(antispam)
  }
}

impl Drop for AntiSpam<'a, 'r> {
  fn drop(&mut self) {
    if let Some(req) = self.request {
      let current_cookie = req.cookies().get_private("antispam");
      let current_id = current_cookie
        .and_then(|x| serde_json::from_str(x.value()).ok())
        .map(|x: AntiSpam| x.id);

      if current_id != Some(self.id) {
        let json = match serde_json::to_string(self) {
          Ok(b) => b,
          Err(e) => {
            println!("could not serialize antispam: {}", e);
            return;
          },
        };

        let cookie = Cookie::build("antispam", json)
          .secure(true)
          .http_only(true)
          .same_site(SameSite::Lax)
          .finish();
        req.cookies().add_private(cookie);
      }
    }
  }
}


#[derive(Debug, Default)]
pub struct Links {
  links: HashMap<String, Value>,
}

impl serde::Serialize for Links {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer,
  {
    self.links.serialize(serializer)
  }
}

impl Links {
  pub fn add_value<K, V>(&mut self, key: K, value: V) -> &mut Self
    where K: Into<String>,
          V: serde::Serialize,
  {
    self.links.insert(key.into(), json!(value));

    self
  }

  pub fn add<K>(&mut self, key: K, uri: rocket::http::uri::Origin) -> &mut Self
    where K: Into<String>,
  {
    self.add_value(key, uri.to_string())
  }
}

lazy_static! {
  static ref STATIC_LINKS: Links = {
    let mut links = Links::default();
    links
      .add("index", uri!(crate::routes::web::index::get))
      .add("about", uri!(crate::routes::web::about::get))
      .add("login", uri!(crate::routes::web::auth::login::get))
      .add("logout", uri!(crate::routes::web::auth::logout::post))
      .add("register", uri!(crate::routes::web::auth::register::get))
      .add("settings", uri!(crate::routes::web::account::index::get))
      .add("admin", uri!(crate::routes::web::admin::index::get))
      .add("credits", uri!(crate::routes::web::credits::get));
    links
  };
}

pub fn context(config: &Config, user: Option<&User>, session: &mut Session, langs: AcceptLanguage) -> Value {
  json!({
    "config": &*config.read(),
    "langs": langs.into_strings(),
    "error": session.data.remove("error"),
    "error_safe": session.data.remove("error_safe"),
    "info": session.data.remove("info"),
    "form": session.take_form(),
    "user": user,
    "session": session,
    "server_version": crate::SERVER_VERSION,
    "resources_version": &*crate::RESOURCES_VERSION,
    "static_links": &*STATIC_LINKS,
    "user_page": user
      .as_ref()
      .map(|x| uri!(crate::routes::web::users::get::get: x.username(), _).to_string()),
    "user_avatar_link": user
      .as_ref()
      .map(|x| uri!(crate::routes::web::account::avatar::get: x.id()).to_string()),
  })
}

#[derive(Debug)]
pub struct OptionalWebUser(Option<User>);

impl OptionalWebUser {
  pub fn into_inner(self) -> Option<User> {
    self.0
  }
}

impl FromRequest<'a, 'r> for OptionalWebUser {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let session = match request.guard::<Session>() {
      Outcome::Success(s) => s,
      Outcome::Failure((status, _)) => return Outcome::Failure((status, ())),
      Outcome::Forward(()) => return Outcome::Forward(()),
    };
    let id = match session.user_id {
      Some(id) => id,
      None => return Outcome::Success(OptionalWebUser(None)),
    };

    let conn = match request.guard::<State<PostgresPool>>() {
      Outcome::Success(p) => match p.get() {
        Ok(c) => c,
        Err(_) => return Outcome::Failure((HttpStatus::ServiceUnavailable, ())),
      },
      Outcome::Failure((status, _)) => return Outcome::Failure((status, ())),
      Outcome::Forward(()) => return Outcome::Forward(()),
    };

    match users_db::table.find(id).first(&*conn) {
      Ok(u) => Outcome::Success(OptionalWebUser(Some(u))),
      Err(_) => Outcome::Success(OptionalWebUser(None)),
    }
  }
}

impl Deref for OptionalWebUser {
  type Target = Option<User>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Responder)]
#[allow(clippy::large_enum_variant)]
pub enum Rst {
  Redirect(Redirect),
  Status(HttpStatus),
  Template(Template),
}

pub struct AddCsp<T>(T, Vec<String>);

impl<T> AddCsp<T> {
  pub fn new<I, S>(inner: T, directives: I) -> Self
    where I: IntoIterator<Item = S>,
          S: AsRef<str>,
  {
    AddCsp(inner, directives.into_iter().map(|x| x.as_ref().to_string()).collect())
  }

  #[allow(unused)]
  pub fn none(inner: T) -> Self {
    AddCsp(inner, Default::default())
  }
}

impl<'r, T> Responder<'r> for AddCsp<T>
  where T: Responder<'r>,
{
  fn respond_to(self, request: &Request) -> result::Result<Response<'r>, HttpStatus> {
    let mut response = self.0.respond_to(request)?;
    response.set_header(Header::new("Content-Security-Policy", self.1.join("; ")));
    Ok(response)
  }
}
