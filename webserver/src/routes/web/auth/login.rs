use config::Config;
use database::DbConn;
use database::models::login_attempts::LoginAttempt;
use database::models::users::User;
use database::schema::users;
use errors::*;
use routes::web::{context, Rst, OptionalWebUser, Session};

use cookie::{Cookie, SameSite};

use diesel::prelude::*;

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

#[derive(Debug, FromForm)]
struct RegistrationData {
  username: String,
  password: String,
  anti_csrf_token: String,
}

#[post("/login", format = "application/x-www-form-urlencoded", data = "<data>")]
fn post(data: Form<RegistrationData>, mut sess: Session, mut cookies: Cookies, conn: DbConn, addr: SocketAddr) -> Result<Redirect> {
  let data = data.into_inner();

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

  let cookie = Cookie::build("user_id", user.id().simple().to_string())
    .secure(true)
    .http_only(true)
    .same_site(SameSite::Lax)
    .finish();
  cookies.add_private(cookie);

  Ok(Redirect::to("lastpage"))
}
