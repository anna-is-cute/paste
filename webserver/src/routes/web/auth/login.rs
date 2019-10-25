use crate::{
  config::Config,
  database::{
    DbConn,
    models::{backup_codes::BackupCode, login_attempts::LoginAttempt, users::User},
    schema::{backup_codes, users},
  },
  errors::*,
  i18n::prelude::*,
  redis_store::Redis,
  routes::web::{context, AddCsp, Honeypot, Rst, OptionalWebUser, Session},
  utils::{
    AcceptLanguage,
    ClientIp,
    totp::totp_raw_skew,
  },
};

use diesel::prelude::*;

use oath::HashType;

use redis::Commands;

use rocket::State;
use rocket::request::Form;
use rocket::response::Redirect;

use rocket_contrib::templates::Template;

use serde_json::json;

#[get("/login")]
pub fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session, langs: AcceptLanguage) -> AddCsp<Rst> {
  if user.is_some() {
    return AddCsp::none(Rst::Redirect(Redirect::to("lastpage")));
  }

  let honeypot = Honeypot::new();
  let mut ctx = context(&*config, user.as_ref(), &mut sess, langs);
  ctx["honeypot"] = json!(honeypot);
  ctx["links"] = json!(links!(
    "login_action" => uri!(crate::routes::web::auth::login::post),
    "forgot_password" => uri!(crate::routes::web::account::reset_password::get),
  ));
  AddCsp::new(
    Rst::Template(Template::render("auth/login", ctx)),
    vec![format!("style-src '{}'", honeypot.integrity_hash)],
  )
}

#[derive(Debug, FromForm, Serialize)]
pub struct RegistrationData {
  username: String,
  #[serde(skip)]
  password: String,
  #[serde(skip)]
  tfa_code: Option<String>,
  #[serde(skip)]
  anti_csrf_token: String,
  #[serde(skip)]
  #[form(field = "email")]
  honeypot: String,
}

#[post("/login", format = "application/x-www-form-urlencoded", data = "<data>")]
pub fn post(data: Form<RegistrationData>, mut sess: Session, conn: DbConn, mut redis: Redis, addr: ClientIp, l10n: L10n) -> Result<Redirect> {
  let data = data.into_inner();
  sess.set_form(&data);

  if !sess.check_token(&data.anti_csrf_token) {
    sess.add_data("error", l10n.tr("error-csrf")?);
    return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get)));
  }

  if !data.honeypot.is_empty() {
    sess.add_data("error", "An error occurred. Please try again.");
    return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get)));
  }

  if let Some(msg) = LoginAttempt::find_check(&conn, *addr)? {
    sess.add_data("error", msg);
    return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get)));
  }

  let user: Option<User> = users::table
    .filter(users::username.eq(&data.username))
    .first(&*conn)
    .optional()?;

  let user = match user {
    Some(u) => u,
    None => {
      let msg = match LoginAttempt::find_increment(&conn, *addr)? {
        Some(msg) => msg,
        None => "Username not found.".into(),
      };
      sess.add_data("error", msg);
      return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get)));
    },
  };

  if !user.check_password(&data.password) {
    let msg = match LoginAttempt::find_increment(&conn, *addr)? {
      Some(msg) => msg,
      None => "Incorrect password.".into(),
    };
    sess.add_data("error", msg);
    return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get)));
  }

  let tfa_check = || -> Result<bool> {
    if !user.tfa_enabled() {
      return Ok(true);
    }

    let tfa_code_s = match data.tfa_code {
      Some(s) => s,
      None => return Ok(false),
    };

    match tfa_code_s.len() {
      6 => if_chain! {
        if let Some(ss) = user.shared_secret();
        if let Ok(tfa_code) = tfa_code_s.parse::<u64>();
        if !redis.exists::<_, bool>(format!("otp:{},{}", user.id(), tfa_code))?;
        if totp_raw_skew(ss, 6, 0, 30, &HashType::SHA1).iter().any(|&x| x == tfa_code);
        then {
          redis.set_ex(format!("otp:{},{}", user.id(), tfa_code), "", 120)?;
        } else {
          return Ok(false);
        }
      },
      12 => if_chain! {
        let backup_code = diesel::delete(backup_codes::table)
            .filter(backup_codes::code.eq(tfa_code_s))
            .get_result::<BackupCode>(&*conn)
            .optional()?;
        if backup_code.is_none();
        then {
          return Ok(false);
        }
      },
      _ => return Ok(false),
    }

    Ok(true)
  };

  if !tfa_check()? {
    let msg = match LoginAttempt::find_increment(&conn, *addr)? {
      Some(msg) => msg,
      None => "Invalid authentication code.".into(),
    };
    sess.add_data("error", msg);
    return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get)));
  }

  sess.user_id = Some(user.id());

  sess.take_form();
  Ok(Redirect::to("lastpage"))
}
