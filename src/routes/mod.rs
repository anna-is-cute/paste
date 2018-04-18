#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value, print_literal))]

use database::{PostgresPool, schema};
use database::models::users::User;
use database::models::deletion_keys::DeletionKey;
use errors::Result;
use models::status::Status;

use diesel::prelude::*;

use rocket::{Request, State, Outcome};
use rocket::http::Status as HttpStatus;
use rocket::request::{self, FromRequest};
use rocket::response::status::Custom;

use rocket_contrib::Json;

use uuid::Uuid;

use std::ops::Deref;
use std::str::FromStr;

pub type RouteResult<T> = Result<Custom<Json<Status<T>>>>;

pub mod pastes;

#[error(400)]
pub fn bad_request(req: &Request) -> String {
  if req.uri().path().starts_with("/api") {
    return r#"{"status":"error","error":"bad_request"}"#.into();
  }
  // FIXME: use template when frontend work starts
  Default::default()
}

#[error(404)]
pub fn not_found(req: &Request) -> String {
  if req.uri().path().starts_with("/api") {
    return r#"{"status":"error","error":"not_found"}"#.into();
  }
  // FIXME: use template when frontend work starts
  Default::default()
}

#[error(500)]
pub fn internal_server_error(req: &Request) -> String {
  if req.uri().path().starts_with("/api") {
    return r#"{"status":"error","error":"internal_server_error"}"#.into();
  }
  // FIXME: use template when frontend work starts
  Default::default()
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
