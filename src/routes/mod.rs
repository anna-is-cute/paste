use database::{PostgresPool, schema};
use database::models::users::User;
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
use std::result;
use std::str::FromStr;

pub type RouteResult<T> = Result<Custom<Json<Status<T>>>>;

pub mod pastes;

#[derive(Debug)]
pub enum ApiKeyError {
  NotPresent,
  Invalid,
  BadHeader,
  NotLinked,
  Internal,
}

#[derive(Debug)]
pub struct ApiKey(Uuid);

#[derive(Debug)]
pub struct OptionalUser(Option<User>);

impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
  type Error = ApiKeyError;

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let auth = match request.headers().get_one("Authorization") {
      Some(a) => a,
      None => return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::NotPresent)),
    };
    if !auth.to_lowercase().starts_with("key ") {
      return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::BadHeader));
    }
    match Uuid::from_str(&auth[4..]) {
      Ok(u) => Outcome::Success(ApiKey(u)),
      Err(_) => Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::Invalid)),
    }
  }
}

impl Deref for ApiKey {
  type Target = Uuid;

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
      Outcome::Forward(f) => return Outcome::Forward(f),
    };
    let user = schema::users::table
      .inner_join(schema::api_keys::table)
      .filter(schema::api_keys::dsl::key.eq(uuid))
      .select(schema::users::all_columns)
      .first(&*conn)
      .optional();
    let user = match user {
      Ok(u) => u,
      Err(_) => return Outcome::Failure((HttpStatus::BadRequest, ApiKeyError::NotLinked)),
    };
    Outcome::Success(OptionalUser(user))
  }
}

impl Deref for OptionalUser {
  type Target = Option<User>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
