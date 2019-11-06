pub mod login;
pub mod logout;
pub mod register;

use chrono::Duration;

use cookie::SameSite;

use redis::Commands;

use rocket::{
  Outcome,
  http::{Cookie, Cookies, Status},
  request::{self, FromRequest, Request},
};

use uuid::Uuid;

use crate::{
  errors::*,
  models::id::UserId,
  redis_store::Redis,
};

use std::{
  ops::Deref,
  str::FromStr,
};

pub struct PotentialUser {
  id: Uuid,
  user_id: UserId,
}

impl Deref for PotentialUser {
  type Target = UserId;

  fn deref(&self) -> &Self::Target {
    &self.user_id
  }
}

impl PotentialUser {
  fn make_cookie<'a>(id: Uuid) -> Cookie<'a> {
    Cookie::build("auth", id.to_simple().to_string())
      .max_age(Duration::minutes(10))
      .http_only(true)
      .secure(true)
      .same_site(SameSite::Strict)
      .path(uri!(crate::routes::web::auth::login::tfa).to_string())
      .finish()
  }

  pub fn set(redis: &mut Redis, cookies: &mut Cookies, user_id: UserId) -> Result<()> {
    // generate a uuid for this auth session
    let auth_id = Uuid::new_v4();
    // set up redis first, since adding a cookie is infallible
    redis.set_ex(
      format!("auth:{}", auth_id.to_simple()),
      user_id.to_simple().to_string(),
      10 * 60, // expire in 10 minutes
    )?;
    // build a cookie that expires in 10 minutes, is http-only, is secure, is samesite strict, and
    // is only accessible on the 2fa page
    let cookie = PotentialUser::make_cookie(auth_id);
    // add the encrypted cookie
    cookies.add_private(cookie);

    Ok(())
  }

  pub fn remove(&self, redis: &mut Redis, cookies: &mut Cookies) -> Result<()> {
    // remove cookie first, since it cannot fail
    let cookie = PotentialUser::make_cookie(self.id);
    cookies.remove_private(cookie);
    // remove redis
    redis.del(format!("auth:{}", self.id.to_simple()))?;

    Ok(())
  }
}

impl <'a, 'r> FromRequest<'a, 'r> for PotentialUser {
  type Error = ();

  fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    // get the auth cookie or 404
    let cookie = match req.cookies().get_private("auth") {
      Some(c) => c,
      None => return Outcome::Forward(()),
    };
    // parse the auth cookie as a uuid or 404
    let auth_id = match Uuid::from_str(cookie.value()) {
      Ok(u) => u,
      Err(_) => return Outcome::Forward(()),
    };
    // get redis connection
    let mut redis: Redis = req.guard()?;
    // look up the auth key or 404
    let user_id: String = match redis.get(format!("auth:{}", auth_id.to_simple())) {
      Ok(s) => s,
      Err(_) => return Outcome::Forward(()),
    };
    // parse user id as uuid or 500
    let user_id = match Uuid::from_str(&user_id) {
      Ok(s) => s,
      Err(_) => return Outcome::Failure((Status::InternalServerError, ())),
    };
    // return the user id for the potential user
    Outcome::Success(PotentialUser {
      id: auth_id,
      user_id: UserId(user_id),
    })
  }
}
