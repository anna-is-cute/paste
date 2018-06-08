use config::Config;
use database::DbConn;
use database::models::password_reset_attempts::PasswordResetAttempt;
use database::models::password_resets::{PasswordReset, NewPasswordReset};
use database::models::users::User;
use database::schema::{users, password_resets};
use errors::*;
use routes::web::{context, Session, Rst, OptionalWebUser};
use sidekiq_::Job;
use utils::{email, PasswordContext, HashedPassword};

use base64;

use chrono::{DateTime, Duration, Utc};

use cookie::{Cookie, SameSite};

use diesel;
use diesel::prelude::*;

use rocket::http::Cookies;
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;

use rocket_contrib::{Template, UUID};

use sidekiq::Client as SidekiqClient;

use uuid::Uuid;

use std::net::SocketAddr;

#[get("/account/forgot_password")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Template {
  let ctx = context(&*config, user.as_ref(), &mut sess);
  Template::render("account/forgot_password", ctx)
}

#[post("/account/forgot_password", format = "application/x-www-form-urlencoded", data = "<data>")]
fn post(data: Form<ResetRequest>, config: State<Config>, mut sess: Session, conn: DbConn, sidekiq: State<SidekiqClient>, addr: SocketAddr) -> Result<Redirect> {
  let data = data.into_inner();

  let res = Ok(Redirect::to("/account/forgot_password"));

  if !sess.check_token(&data.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return res;
  }

  if !email::check_email(&data.email) {
    sess.add_data("error", "Invalid email.");
    return res;
  }

  if let Some(msg) = PasswordResetAttempt::find_check(&conn, addr.ip())? {
    sess.add_data("error", msg);
    return res;
  }

  let msg = format!(
    "If an account has a verified email address of {}, a password reset email was sent to it.",
    data.email,
  );

  let user: Option<User> = users::table
    .filter(users::email.eq(&data.email))
    .first(&*conn)
    .optional()?;

  let user = match user {
    Some(u) => u,
    None => {
      let (k, m) = match PasswordResetAttempt::find_increment(&conn, addr.ip())? {
        Some(m) => ("error", m),
        None => ("info", msg),
      };
      sess.add_data(k, m);
      return res;
    },
  };

  if !user.email_verified() {
    sess.add_data("info", msg);
    return res;
  }

  let (reset, key) = NewPasswordReset::generate(user.id());

  diesel::insert_into(password_resets::table)
    .values(&reset)
    .execute(&*conn)?;

  sidekiq.push(Job::email(
    "password_reset.html.tera",
    json!({
      "config": &*config,
      "user": user,
      "reset_url": format!(
        "https://{}/account/reset_password?id={}&secret={}",
        config.general.site_domain,
        reset.id,
        base64::encode_config(&key, base64::URL_SAFE),
      ),
    }),
    config._path.as_ref().unwrap(),
    user.email(),
    "Password reset",
  )?.into())?;

  sess.add_data("info", msg);
  res
}

#[get("/account/reset_password?<data>")]
fn reset_get(data: ResetPassword, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  if check_reset(&conn, *data.id, &data.secret).is_none() {
    sess.add_data("error", "Invalid password reset URL.");
    return Ok(Rst::Redirect(Redirect::to("/account/forgot_password")));
  }

  let mut ctx = context(&*config, user.as_ref(), &mut sess);
  ctx["pr_id"] = json!(data.id.simple().to_string());
  ctx["pr_secret"] = json!(&data.secret);

  Ok(Rst::Template(Template::render("account/reset_password", ctx)))
}

#[post("/account/reset_password", data = "<data>")]
fn reset_post(data: Form<Reset>, mut sess: Session, mut cookies: Cookies, conn: DbConn) -> Result<Redirect> {
  let data = data.into_inner();

  let url = format!("/account/reset_password?id={}&secret={}", data.id.simple(), data.secret);
  let res = Ok(Redirect::to(&url));

  if !sess.check_token(&data.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return res;
  }

  let reset = match check_reset(&conn, *data.id, &data.secret) {
    Some(r) => r,
    None => {
      sess.add_data("error", "Invalid password reset.");
      return res;
    },
  };

  let user: Option<User> = users::table
    .find(*reset.user_id())
    .first(&*conn)
    .optional()?;

  let mut user = match user {
    Some(u) => u,
    None => {
      diesel::delete(&reset).execute(&*conn)?;
      sess.add_data("error", "That account does not exist.");
      return Ok(Redirect::to("/account/forgot_password"));
    },
  };

  {
    let pw_ctx = PasswordContext::new(
      &data.password,
      &data.password_verify,
      user.name(),
      user.username(),
      user.email(),
    );
    if let Err(e) = pw_ctx.validate() {
      sess.add_data("error", e);
      return Ok(Redirect::to(&url));
    }
  }

  diesel::delete(&reset).execute(&*conn)?;

  let hashed = HashedPassword::from(&data.password).into_string();

  user.set_hashed_password(hashed);
  user.update(&conn)?;

  sess.add_data("info", "Password updated.");

  let cookie = Cookie::build("user_id", user.id().simple().to_string())
    .secure(true)
    .http_only(true)
    .same_site(SameSite::Lax)
    .max_age(Duration::days(30))
    .finish();
  cookies.add_private(cookie);

  Ok(Redirect::to("lastpage"))
}

fn check_reset(conn: &DbConn, id: Uuid, secret: &str) -> Option<PasswordReset> {
  let secret = base64::decode_config(secret, base64::URL_SAFE).ok()?;

  let reset: PasswordReset = password_resets::table
    .find(id)
    .first(&**conn)
    .optional()
    .ok()??;

  if DateTime::from_utc(reset.expiry(), Utc) < Utc::now() {
    return None;
  }

  if !reset.check(&secret) {
    return None;
  }

  Some(reset)
}

#[derive(FromForm)]
struct ResetRequest {
  anti_csrf_token: String,
  email: String,
}

#[derive(FromForm)]
struct ResetPassword {
  id: UUID,
  secret: String,
}

#[derive(FromForm)]
struct Reset {
  id: UUID,
  secret: String,
  password: String,
  password_verify: String,
  anti_csrf_token: String,
}
