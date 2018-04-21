use database::PostgresPool;
use database::models::users::User;
use database::schema::users;

use rocket::{State, Outcome};
use rocket::http::Status as HttpStatus;
use rocket::request::{self, Request, FromRequest};

use diesel::prelude::*;

use uuid::Uuid;

use std::ops::Deref;

pub mod account;
pub mod auth;
pub mod index;
pub mod pastes;
pub mod static_files;

#[derive(Debug)]
pub struct OptionalWebUser(Option<User>);

impl OptionalWebUser {
  pub fn into_inner(self) -> Option<User> {
    self.0
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for OptionalWebUser {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let id = request
      .cookies()
      .get_private("user_id")
      .and_then(|x| Uuid::parse_str(x.value()).ok());
    let id = match id {
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

    match users::table.find(id).first(&*conn) {
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
