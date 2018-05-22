use config::Config;
use database::DbConn;
use database::models::password_resets::{PasswordReset, NewPasswordReset};
use database::models::users::User;
use database::schema::{users, password_resets};
use errors::*;
use routes::web::{context, Session, OptionalWebUser};
use sidekiq_::Job;
use utils::ReCaptcha;

use base64;

use diesel;
use diesel::prelude::*;

use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;

use rocket_contrib::{Template, UUID};

use sidekiq::Client as SidekiqClient;

#[get("/account/forgot_password")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Template {
  let ctx = context(&*config, user.as_ref(), &mut sess);
  Template::render("account/forgot_password", ctx)
}

#[post("/account/forgot_password", format = "application/x-www-form-urlencoded", data = "<data>")]
fn post(data: Form<ResetRequest>, config: State<Config>, mut sess: Session, conn: DbConn, sidekiq: State<SidekiqClient>) -> Result<Redirect> {
  let data = data.into_inner();

  let res = Ok(Redirect::to("/account/forgot_password"));

  if !sess.check_token(&data.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return res;
  }

  if !data.recaptcha.verify(&config.recaptcha.secret_key)? {
    sess.add_data("error", "The captcha did not validate. Try again.");
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
      sess.add_data("info", msg);
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
        "https://{}/account/reset_password/?id={}&secret={}",
        config.general.site_domain,
        reset.id,
        base64::encode_config(&key, base64::URL_SAFE),
      ),
    }),
    config._path.as_ref().unwrap(),
    user.email(),
    user.name(),
    "Password reset",
  )?.into())?;

  sess.add_data("info", msg);
  res
}

#[get("/account/reset_password?<data>")]
fn reset(data: ResetPassword, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let secret = match base64::decode_config(&data.secret, base64::URL_SAFE) {
    Ok(s) => s,
    Err(_) => {
      sess.add_data("error", "Invalid password reset URL.");
      return Ok(Redirect::to("/account/reset_password"));
    },
  };

  let reset: Option<PasswordReset> = password_resets::table
    .find(*data.id)
    .first(&*conn)
    .optional()?;

  let reset = match reset {
    Some(r) => r,
    None => {
      sess.add_data("error", "Invalid password reset URL.");
      return Ok(Redirect::to("/account/reset_password"));
    },
  };

  if reset.check(&secret) {

  }
  unimplemented!()
}

#[derive(FromForm)]
struct ResetRequest {
  anti_csrf_token: String,
  email: String,
  #[form(field = "g-recaptcha-response")]
  recaptcha: ReCaptcha,
}

#[derive(FromForm)]
struct ResetPassword {
  id: UUID,
  secret: String,
}
