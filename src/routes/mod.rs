#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value, print_literal))]

use config::Config;
use database::{PostgresPool, schema};
use database::models::deletion_keys::DeletionKey;
use database::models::users::User;
use errors::*;
use models::id::ApiKeyId;
use models::status::Status;
use routes::web::{context, OptionalWebUser, Session};

use diesel::prelude::*;

use rocket::{Request, State, Outcome};
use rocket::http::Status as HttpStatus;
use rocket::request::{self, FromRequest};
use rocket::response::{Responder, Response};
use rocket::response::status::Custom;

use rocket_contrib::{Json, Template};

use uuid::Uuid;

use std::ops::Deref;
use std::result;
use std::str::FromStr;

pub type RouteResult<T> = Result<Custom<Json<Status<T>>>>;

pub mod pastes;
pub mod web;

enum StringOrTemplate {
  String(String),
  Template(Template),
}

impl<'r> Responder<'r> for StringOrTemplate {
    fn respond_to(self, request: &Request) -> result::Result<Response<'r>, HttpStatus> {
      match self {
        StringOrTemplate::String(s) => s.respond_to(request),
        StringOrTemplate::Template(t) => t.respond_to(request),
      }
    }
}

fn error(req: &Request, kind: &str, template: &'static str) -> StringOrTemplate {
  if req.uri().path().starts_with("/api/") || req.uri().path() == "/api" {
    return StringOrTemplate::String(format!("{{\"status\":\"error\",\"error\":\"{}\"}}", kind));
  }
  let config: State<Config> = req.guard().unwrap();
  let user: OptionalWebUser = req.guard().unwrap();
  let mut session: Session = req.guard().unwrap();
  let ctx = context(&*config, user.as_ref(), &mut session);
  StringOrTemplate::Template(Template::render(template, ctx))
}

#[error(400)]
fn bad_request(req: &Request) -> StringOrTemplate {
  error(req, "bad_request", "error/400")
}

#[error(403)]
fn forbidden(req: &Request) -> StringOrTemplate {
  error(req, "forbidden", "error/403")
}

#[error(404)]
fn not_found(req: &Request) -> StringOrTemplate {
  error(req, "not_found", "error/404")
}

#[error(500)]
fn internal_server_error(req: &Request) -> StringOrTemplate {
  error(req, "internal_server_error", "error/500")
}

#[derive(Debug)]
pub enum ApiKeyError {
  NotPresent,
  Invalid,
  BadHeader,
  NotLinked,
  Internal,
}

#[derive(Debug)]
pub enum DeletionAuth {
  User(User),
  Key(DeletionKey),
}

#[derive(Debug)]
pub struct RequiredUser(User);

#[derive(Debug)]
pub struct OptionalUser(Option<User>);

impl<'a, 'r> FromRequest<'a, 'r> for DeletionAuth {
  type Error = ApiKeyError;

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let auth = match request.headers().get_one("Authorization") {
      Some(a) => a,
      None => return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::NotPresent)),
    };
    if !auth.to_lowercase().starts_with("key ") {
      return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::BadHeader));
    }
    let uuid = match Uuid::from_str(&auth[4..]) {
      Ok(u) => u,
      Err(_) => return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::Invalid)),
    };
    let conn = match request.guard::<State<PostgresPool>>() {
      Outcome::Success(p) => match p.get() {
        Ok(c) => c,
        Err(_) => return Outcome::Failure((HttpStatus::ServiceUnavailable, ApiKeyError::Internal)),
      },
      Outcome::Failure((status, _)) => return Outcome::Failure((status, ApiKeyError::Internal)),
      Outcome::Forward(()) => return Outcome::Forward(()),
    };
    let user = schema::users::table
      .inner_join(schema::api_keys::table)
      .filter(schema::api_keys::dsl::key.eq(uuid))
      .select(schema::users::all_columns)
      .first(&*conn)
      .optional();
    let auth = match user {
      Ok(Some(u)) => DeletionAuth::User(u),
      Ok(None) => {
        match schema::deletion_keys::table
          .filter(schema::deletion_keys::key.eq(uuid))
          .first(&*conn)
          .optional()
        {
          Ok(Some(d)) => DeletionAuth::Key(d),
          Ok(None) => return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::NotLinked)),
          Err(_) => return Outcome::Failure((HttpStatus::ServiceUnavailable, ApiKeyError::Internal)),
        }
      },
      Err(_) => return Outcome::Failure((HttpStatus::ServiceUnavailable, ApiKeyError::Internal)),
    };
    Outcome::Success(auth)
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for RequiredUser {
  type Error = ApiKeyError;

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let auth = match request.headers().get_one("Authorization") {
      Some(a) => a,
      None => return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::NotPresent)),
    };
    if !auth.to_lowercase().starts_with("key ") {
      return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::BadHeader));
    }
    let uuid = match Uuid::from_str(&auth[4..]) {
      Ok(u) => ApiKeyId(u),
      Err(_) => return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::Invalid)),
    };
    let conn = match request.guard::<State<PostgresPool>>() {
      Outcome::Success(p) => match p.get() {
        Ok(c) => c,
        Err(_) => return Outcome::Failure((HttpStatus::ServiceUnavailable, ApiKeyError::Internal)),
      },
      Outcome::Failure((status, _)) => return Outcome::Failure((status, ApiKeyError::Internal)),
      Outcome::Forward(()) => return Outcome::Forward(()),
    };
    let user = schema::users::table
      .inner_join(schema::api_keys::table)
      .filter(schema::api_keys::dsl::key.eq(uuid))
      .select(schema::users::all_columns)
      .first(&*conn)
      .optional();
    let user = match user {
      Ok(Some(u)) => u,
      Ok(None) => return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::NotLinked)),
      Err(_) => return Outcome::Failure((HttpStatus::ServiceUnavailable, ApiKeyError::Internal)),
    };
    Outcome::Success(RequiredUser(user))
  }
}

impl Deref for RequiredUser {
  type Target = User;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for OptionalUser {
  type Error = ApiKeyError;

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let auth = match request.headers().get_one("Authorization") {
      Some(a) => a,
      None => return Outcome::Success(OptionalUser(None)),
    };
    if !auth.to_lowercase().starts_with("key ") {
      return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::BadHeader));
    }
    let uuid = match Uuid::from_str(&auth[4..]) {
      Ok(u) => u,
      Err(_) => return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::Invalid)),
    };
    let conn = match request.guard::<State<PostgresPool>>() {
      Outcome::Success(p) => match p.get() {
        Ok(c) => c,
        Err(_) => return Outcome::Failure((HttpStatus::ServiceUnavailable, ApiKeyError::Internal)),
      },
      Outcome::Failure((status, _)) => return Outcome::Failure((status, ApiKeyError::Internal)),
      Outcome::Forward(()) => return Outcome::Forward(()),
    };
    let user = schema::users::table
      .inner_join(schema::api_keys::table)
      .filter(schema::api_keys::dsl::key.eq(uuid))
      .select(schema::users::all_columns)
      .first(&*conn)
      .optional();
    let user = match user {
      Ok(Some(u)) => u,
      Ok(None) => return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::NotLinked)),
      Err(_) => return Outcome::Failure((HttpStatus::ServiceUnavailable, ApiKeyError::Internal)),
    };
    Outcome::Success(OptionalUser(Some(user)))
  }
}

impl Deref for OptionalUser {
  type Target = Option<User>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
