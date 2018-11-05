use crate::{
  config::Config,
  database::{
    PostgresPool,
    models::users::User,
    schema::users as users_db,
  },
  models::id::UserId,
};

use diesel::prelude::*;

use rocket::{
  State, Outcome,
  http::{Header, Status as HttpStatus},
  request::{self, Request, FromRequest},
  response::{Responder, Response, Redirect},
};

use rocket_contrib::Template;

use serde_json::{Value, json};

use uuid::Uuid;

use std::{ops::Deref, result};

pub mod about;
pub mod account;
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
    use rand::{Rng, seq::SliceRandom};
    use sha2::{Digest, Sha384};

    const ALPHA: [char; 6] = ['a', 'b', 'c', 'd', 'e', 'f'];
    const ALPHANUMERIC: [char; 16] = ['a', 'b', 'c', 'd', 'e', 'f', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

    let mut rng = rand::thread_rng();

    let start = ALPHA.choose(&mut rng).unwrap();
    let end: String = ALPHANUMERIC.choose_multiple(&mut rng, 15).collect();
    let class = format!("{}{}", start, end);

    let skip = rng.gen_range(1, 4);

    let css = format!(
      "[class *= {}] {{ position: absolute; left: -100vw; width: 1px; height: 1px; }}",
      &class[..class.len() - skip],
    );

    let mut hasher = Sha384::new();
    hasher.input(&css);
    let integrity_hash = format!("sha384-{}", base64::encode(&hasher.result()[..]));

    Honeypot {
      class,
      css,
      integrity_hash,
    }
  }
}

pub fn context(config: &Config, user: Option<&User>, session: &mut Session) -> Value {
  json!({
    "config": &config,
    "error": session.data.remove("error"),
    "info": session.data.remove("info"),
    "form": session.take_form(),
    "user": user,
    "session": session,
    "server_version": crate::SERVER_VERSION,
    "resources_version": &*crate::RESOURCES_VERSION,
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
    let id = request
      .cookies()
      .get_private("user_id")
      .and_then(|x| Uuid::parse_str(x.value()).ok());
    let id = match id {
      Some(id) => UserId(id),
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

pub enum Rst {
  Redirect(Redirect),
  Status(HttpStatus),
  Template(Template),
}

impl Responder<'r> for Rst {
  fn respond_to(self, request: &Request) -> result::Result<Response<'r>, HttpStatus> {
    match self {
      Rst::Redirect(r) => r.respond_to(request),
      Rst::Status(s) => Err(s),
      Rst::Template(t) => t.respond_to(request),
    }
  }
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
