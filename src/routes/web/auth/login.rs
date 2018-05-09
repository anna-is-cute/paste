use config::Config;
use database::DbConn;
use database::models::login_attempts::LoginAttempt;
use database::models::users::User;
use database::schema::users;
use errors::*;
use routes::web::{Rst, OptionalWebUser, Session};

use diesel::prelude::*;

use rocket::State;
use rocket::http::{Cookies, Cookie};
use rocket::request::Form;
use rocket::response::Redirect;

use rocket_contrib::Template;

use std::net::SocketAddr;

#[get("/login")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Rst {
  if user.is_some() {
    return Rst::Redirect(Redirect::to("lastpage"));
  }

  let ctx = json!({
    "config": &*config,
    // TODO: this can be made into an optional request guard
    "error": sess.data.remove("error"),
    "info": sess.data.remove("info"),
    "server_version": ::SERVER_VERSION,
    "resources_version": &*::RESOURCES_VERSION,
  });
  Rst::Template(Template::render("auth/login", ctx))
}

#[derive(Debug, FromForm)]
struct RegistrationData {
  username: String,
  password: String,
}

#[post("/login", format = "application/x-www-form-urlencoded", data = "<data>")]
fn post(data: Form<RegistrationData>, mut sess: Session, mut cookies: Cookies, conn: DbConn, addr: SocketAddr) -> Result<Redirect> {
  if let Some(msg) = LoginAttempt::find_check(&conn, addr.ip())? {
    sess.data.insert("error".into(), msg);
    return Ok(Redirect::to("/login"));
  }

  let data = data.into_inner();

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
      sess.data.insert("error".into(), msg);
      return Ok(Redirect::to("/login"));
    },
  };

  if !user.check_password(&data.password) {
    let msg = match LoginAttempt::find_increment(&conn, addr.ip())? {
      Some(msg) => msg,
      None => "Incorrect password.".into(),
    };
    sess.data.insert("error".into(), msg);
    return Ok(Redirect::to("/login"));
  }

  cookies.add_private(Cookie::new("user_id", user.id().simple().to_string()));

  Ok(Redirect::to("lastpage"))
}
