use config::Config;
use database::DbConn;
use database::schema::email_verifications;
use database::models::email_verifications::EmailVerification;
use errors::*;
use models::id::{EmailVerificationId, EmailVerificationKey};
use routes::web::{OptionalWebUser, Session};

use chrono::Utc;

use diesel;
use diesel::prelude::*;

use rocket::State;
use rocket::request::Form;
use rocket::response::Redirect;

use sidekiq::Client as SidekiqClient;

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

  let mut ver = match ver {
    Some(v) => v,
    None => user.create_email_verification(&conn, None)?,
  };

  if !ver.can_send_again() {
    sess.add_data("error", "You must wait 15 minutes between verification email resends.");
    return Ok(Redirect::to("/account"));
  }

  ver.update_last_sent(&conn, Utc::now().naive_utc())?;

  sidekiq.push(ver.job(&config, &user)?.into())?;

  sess.add_data("info", "Email sent.");
  Ok(Redirect::to("/account"))
}

#[get("/account/verify?<data>")]
fn get(data: Verification, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
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
    .filter(email_verifications::key.eq(*data.key)
      .and(email_verifications::email.eq(user.email())))
    .first(&*conn)
    .optional()?;

  let verification = match verification {
    Some(v) => v,
    None => {
      sess.add_data("error", "Invalid email verification.");
      return Ok(Redirect::to("/account"));
    },
  };

  user.set_email_verified(true);
  user.update(&conn)?;

  diesel::delete(&verification).execute(&*conn)?;

  sess.add_data("info", "Email verified.");
  Ok(Redirect::to("/account"))
}

#[derive(Debug, FromForm)]
struct Verification {
  id: EmailVerificationId,
  key: EmailVerificationKey,
}

#[derive(Debug, FromForm)]
struct Resend {
  anti_csrf_token: String,
}
