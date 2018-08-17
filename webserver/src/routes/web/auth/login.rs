use crate::{
  config::Config,
  database::{
    DbConn,
    models::{login_attempts::LoginAttempt, users::User},
    schema::users,
  },
  errors::*,
  redis_store::Redis,
  routes::web::{context, Rst, OptionalWebUser, Session},
  utils::totp::totp_raw_skew,
};

use cookie::{Cookie, SameSite};

use chrono::Duration;

use diesel::prelude::*;

use oath::HashType;

use redis::Commands;

use rocket::State;
use rocket::http::Cookies;
use rocket::request::Form;
use rocket::response::Redirect;

use rocket_contrib::Template;

use std::net::SocketAddr;

#[get("/login")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Rst {
  if user.is_some() {
    return Rst::Redirect(Redirect::to("lastpage"));
  }

  let ctx = context(&*config, user.as_ref(), &mut sess);
  Rst::Template(Template::render("auth/login", ctx))
}

#[derive(Debug, FromForm, Serialize)]
struct RegistrationData {
  username: String,
  #[serde(skip)]
  password: String,
  #[serde(skip)]
  tfa_code: Option<u64>,
  #[serde(skip)]
  anti_csrf_token: String,
}

#[post("/login", format = "application/x-www-form-urlencoded", data = "<data>")]
fn post(data: Form<RegistrationData>, mut sess: Session, mut cookies: Cookies, conn: DbConn, redis: Redis, addr: SocketAddr) -> Result<Redirect> {
  let data = data.into_inner();
  sess.set_form(&data);

  if !sess.check_token(&data.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("/login"));
  }

  if let Some(msg) = LoginAttempt::find_check(&conn, addr.ip())? {
    sess.add_data("error", msg);
    return Ok(Redirect::to("/login"));
  }

  let user: Option<User> = users::table
    .filter(users::username.eq(&data.username))
    .first(&*conn)
    .optional()?;

  let user = match user {
    Some(u) => u,
    None => {
      let msg = match LoginAttempt::find_increment(&conn, addr.ip())? {
        Some(msg) => msg,
        None => "Username not found.".into(),
      };
      sess.add_data("error", msg);
      return Ok(Redirect::to("/login"));
    },
  };

  if !user.check_password(&data.password) {
    let msg = match LoginAttempt::find_increment(&conn, addr.ip())? {
      Some(msg) => msg,
      None => "Incorrect password.".into(),
    };
    sess.add_data("error", msg);
    return Ok(Redirect::to("/login"));
  }

  if_chain! {
    if user.tfa_enabled();
    if let Some(ss) = user.shared_secret();
    if let Some(tfa_code) = data.tfa_code;
    if !redis.exists::<_, bool>(format!("otp:{},{}", user.id(), tfa_code))?;
    if totp_raw_skew(ss, 6, 0, 30, &HashType::SHA1).iter().any(|&x| x == tfa_code);
    then {
      redis.set_ex(format!("otp:{},{}", user.id(), tfa_code), "", 120)?;
    } else {
      if user.tfa_enabled() {
        sess.add_data("error", "Invalid authentication code.");
        return Ok(Redirect::to("/login"));
      }
    }
  }

  let cookie = Cookie::build("user_id", user.id().simple().to_string())
    .secure(true)
    .http_only(true)
    .same_site(SameSite::Lax)
    .max_age(Duration::days(30))
    .finish();
  cookies.add_private(cookie);

  sess.take_form();
  Ok(Redirect::to("lastpage"))
}
