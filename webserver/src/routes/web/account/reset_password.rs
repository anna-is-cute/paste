use crate::{
  config::Config,
  database::{
    DbConn,
    models::{
      password_reset_attempts::PasswordResetAttempt,
      password_resets::{PasswordReset, NewPasswordReset},
      users::User,
    },
    schema::{users, password_resets},
  },
  errors::*,
  routes::web::{context, Session, Rst, OptionalWebUser},
  sidekiq::Job,
  utils::{email, PasswordContext, HashedPassword},
};

use base64;

use chrono::{DateTime, Utc};

use diesel::prelude::*;

use rocket::{
  http::RawStr,
  request::{Form, FromFormValue},
  response::Redirect,
  State,
};

use rocket_contrib::templates::Template;

use serde_json::json;

use sidekiq::Client as SidekiqClient;

use uuid::Uuid;

use std::net::SocketAddr;

#[get("/account/forgot_password")]
pub fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Template {
  let ctx = context(&*config, user.as_ref(), &mut sess);
  Template::render("account/forgot_password", ctx)
}

#[post("/account/forgot_password", format = "application/x-www-form-urlencoded", data = "<data>")]
pub fn post(data: Form<ResetRequest>, config: State<Config>, mut sess: Session, conn: DbConn, sidekiq: State<SidekiqClient>, addr: SocketAddr) -> Result<Redirect> {
  let data = data.into_inner();
  sess.set_form(&data);

  let res = Ok(Redirect::to(uri!(get)));

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
        None => {
          sess.take_form();
          ("info", msg)
        },
      };
      sess.add_data(k, m);
      return res;
    },
  };

  if !user.email_verified() {
    sess.take_form();
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
        base64::encode_config(&key, base64::URL_SAFE_NO_PAD),
      ),
    }),
    config._path.as_ref().unwrap(),
    user.email(),
    "Password reset",
  )?.into())?;

  sess.take_form();
  sess.add_data("info", msg);
  res
}

#[get("/account/reset_password?<data..>")]
pub fn reset_get(data: Form<ResetPassword>, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  if check_reset(&conn, *data.id, &data.secret).is_none() {
    sess.add_data("error", "Invalid password reset URL.");
    return Ok(Rst::Redirect(Redirect::to(uri!(get))));
  }

  let mut ctx = context(&*config, user.as_ref(), &mut sess);
  ctx["pr_id"] = json!(data.id.to_simple().to_string());
  ctx["pr_secret"] = json!(&data.secret);

  Ok(Rst::Template(Template::render("account/reset_password", ctx)))
}

#[post("/account/reset_password", data = "<data>")]
pub fn reset_post(data: Form<Reset>, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let data = data.into_inner();

  let res = Ok(Redirect::to(uri!(
    crate::routes::web::account::reset_password::reset_get:
    ResetPassword {
      id: data.id,
      secret: data.secret.clone(),
    },
  )));

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
      return Ok(Redirect::to(uri!(get)));
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
      return res;
    }
  }

  diesel::delete(&reset).execute(&*conn)?;

  let hashed = HashedPassword::from(&data.password).into_string();

  user.set_hashed_password(hashed);
  user.update(&conn)?;

  sess.add_data("info", "Password updated.");

  sess.user_id = Some(user.id());

  Ok(Redirect::to("lastpage"))
}

fn check_reset(conn: &DbConn, id: Uuid, secret: &str) -> Option<PasswordReset> {
  let secret = base64::decode_config(secret, base64::URL_SAFE_NO_PAD).ok()?;

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

#[derive(FromForm, Serialize)]
pub struct ResetRequest {
  #[serde(skip)]
  anti_csrf_token: String,
  email: String,
}

#[derive(FromForm, UriDisplay)]
pub struct ResetPassword {
  id: ResetId,
  secret: String,
}

#[derive(FromForm)]
pub struct Reset {
  id: ResetId,
  secret: String,
  password: String,
  password_verify: String,
  anti_csrf_token: String,
}

#[derive(Clone, Copy)]
struct ResetId(Uuid);

impl std::ops::Deref for ResetId {
  type Target = Uuid;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'v> FromFormValue<'v> for ResetId {
  type Error = &'v RawStr;

  /// A value is successfully parsed if `form_value` is a properly formatted
  /// Uuid. Otherwise, the raw form value is returned.
  #[inline(always)]
  fn from_form_value(form_value: &'v RawStr) -> std::result::Result<ResetId, &'v RawStr> {
    let uuid: Uuid = form_value.parse().map_err(|_| form_value)?;
    Ok(ResetId(uuid))
  }
}

impl rocket::http::uri::UriDisplay for ResetId {
  fn fmt(&self, f: &mut rocket::http::uri::Formatter) -> std::result::Result<(), std::fmt::Error> {
    use std::fmt::Write;
    f.write_fmt(format_args!("{}", self.to_simple()))
  }
}
