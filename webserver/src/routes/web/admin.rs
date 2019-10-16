pub mod config;
pub mod index;
pub mod maintenance;
pub mod pastes;
pub mod users;

use crate::{
  database::models::users::User,
  routes::web::{Links, OptionalWebUser},
};

use rocket::{
  Outcome,
  request::{self, Request, FromRequest},
};

use std::ops::Deref;

pub(crate) fn admin_links() -> Links {
  links!(
    "overview" => uri!(crate::routes::web::admin::index::get),
    "pastes" => uri!(crate::routes::web::admin::pastes::get),
    "users" => uri!(crate::routes::web::admin::users::get),
    "config" => uri!(crate::routes::web::admin::config::get),
    "maintenance" => uri!(crate::routes::web::admin::maintenance::get),
  )
}

#[derive(Debug)]
pub struct AdminUser(User);

impl AdminUser {
  pub fn into_inner(self) -> User {
    self.0
  }
}

impl FromRequest<'a, 'r> for AdminUser {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let user = match request.guard::<OptionalWebUser>() {
      Outcome::Success(s) => match s.into_inner() {
        Some(u) => u,
        None => return Outcome::Forward(()),
      },
      Outcome::Failure((status, _)) => return Outcome::Failure((status, ())),
      Outcome::Forward(()) => return Outcome::Forward(()),
    };

    if !user.is_admin() {
      return Outcome::Forward(());
    }

    Outcome::Success(AdminUser(user))
  }
}

impl Deref for AdminUser {
  type Target = User;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
