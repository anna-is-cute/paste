use crate::{
  config::Config,
  database::{
    DbConn,
    schema::email_verifications,
    models::email_verifications::EmailVerification,
  },
  errors::*,
  models::id::EmailVerificationId,
  routes::web::{OptionalWebUser, Session},
  utils::HashedPassword,
};

use base64;

use chrono::Utc;

use diesel::prelude::*;

use rocket::{
  State,
  request::Form,
  response::Redirect,
};

use sidekiq::Client as SidekiqClient;

use sodiumoxide::randombytes;

#[post("/account/send_verification", format = "application/x-www-form-urlencoded", data = "<data>")]
fn resend(data: Form<Resend>, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn, sidekiq: State<SidekiqClient>) -> Result<Redirect> {
  let data = data.into_inner();

  if !sess.check_token(&data.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("/account/delete"));
  }

  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  if user.email_verified() {
    sess.add_data("error", "Your email is already verified.");
    return Ok(Redirect::to("/account"));
  }

  let ver: Option<EmailVerification> = email_verifications::table
    .filter(email_verifications::user_id.eq(*user.id())
      .and(email_verifications::email.eq(user.email())))
    .first(&*conn)
    .optional()?;

  let (mut ver, secret) = match ver {
    Some(v) => (v, None),
    None => {
      let (v, s) = user.create_email_verification(&conn, None)?;
      (v, Some(s))
    },
  };

  if !ver.can_send_again() {
    sess.add_data("error", "You must wait 15 minutes between verification email resends.");
    return Ok(Redirect::to("/account"));
  }

  let secret = match secret {
    Some(s) => s,
    None => {
      let s = randombytes::randombytes(32);
      let key = HashedPassword::from(&s).into_string();

      ver.set_key(key);
      ver.update(&conn)?;

      s
    },
  };

  ver.update_last_sent(&conn, Utc::now().naive_utc())?;

  sidekiq.push(ver.job(&config, &user, &secret)?.into())?;

  sess.add_data("info", "Email sent.");
  Ok(Redirect::to("/account"))
}

#[get("/account/verify?<data>")]
fn get(data: Verification, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let key = match base64::decode_config(&data.key, base64::URL_SAFE) {
    Ok(k) => k,
    Err(_) => {
      sess.add_data("error", "Invalid email verification.");
      return Ok(Redirect::to("/account"));
    },
  };

  let mut user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  if user.email_verified() {
    sess.add_data("error", "Your email is already verified.");
    return Ok(Redirect::to("/account"));
  }

  let verification: Option<EmailVerification> = email_verifications::table
    .find(*data.id)
    .filter(email_verifications::email.eq(user.email()))
    .first(&*conn)
    .optional()?;

  let verification = match verification {
    Some(v) => v,
    None => {
      sess.add_data("error", "Invalid email verification.");
      return Ok(Redirect::to("/account"));
    },
  };

  if !verification.check(&key) {
    sess.add_data("error", "Invalid email verification");
    return Ok(Redirect::to("/account"));
  }

  user.set_email_verified(true);
  user.update(&conn)?;

  diesel::delete(&verification).execute(&*conn)?;

  sess.add_data("info", "Email verified.");
  Ok(Redirect::to("/account"))
}

#[derive(Debug, FromForm)]
struct Verification {
  id: EmailVerificationId,
  key: String,
}

#[derive(Debug, FromForm)]
struct Resend {
  anti_csrf_token: String,
}
