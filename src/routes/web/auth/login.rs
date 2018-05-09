use config::Config;
use database::DbConn;
use database::models::users::User;
use database::schema::users;
use errors::*;
use routes::web::{Rst, OptionalWebUser, Session};

use chrono::{Utc, Duration, DateTime};

use diesel::prelude::*;

use rocket::State;
use rocket::http::{Cookies, Cookie};
use rocket::request::Form;
use rocket::response::Redirect;

use rocket_contrib::Template;

use std::collections::HashMap;
use std::net::{SocketAddr, IpAddr};
use std::sync::RwLock;

#[get("/login")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Rst {
  if user.is_some() {
    return Rst::Redirect(Redirect::to("/"));
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

// TODO: managed state or database instead?
lazy_static! {
  static ref ATTEMPTS: RwLock<HashMap<IpAddr, (DateTime<Utc>, usize)>> = Default::default();
}

#[post("/login", format = "application/x-www-form-urlencoded", data = "<data>")]
fn post(data: Form<RegistrationData>, mut sess: Session, mut cookies: Cookies, conn: DbConn, addr: SocketAddr) -> Result<Redirect> {
  {
    let mut attempts = ATTEMPTS.write().unwrap();
    // get the record for this ip or set it to last request now and 0 attempts
    let entry = attempts.entry(addr.ip()).or_insert_with(|| (Utc::now(), 0));

    // increment the attempts
    entry.1 += 1;

    // if it's been 30 minutes since any request, clear the rate limiting
    if Utc::now().signed_duration_since(entry.0) > Duration::minutes(30) {
      *entry = (Utc::now(), 0);
    }

    if entry.1 < 5 {
      // if rate limiting hasn't started yet, update the last request time so that we rate limit
      // based on the attempt that started the limiting
      entry.0 = Utc::now();
    } else {
      // otherwise, let them know that they'll need to wait
      let msg = if entry.1 == 5 {
        "Please try again in 30 minutes."
      } else {
        "Please try again later."
      };
      sess.data.insert("error".into(), msg.into());
      return Ok(Redirect::to("/login"));
    }
  }

  let data = data.into_inner();

  let user: Option<User> = users::table
    .filter(users::username.eq(&data.username))
    .first(&*conn)
    .optional()?;

  let user = match user {
    Some(u) => u,
    None => {
      sess.data.insert("error".into(), "Username not found.".into());
      return Ok(Redirect::to("/login"));
    },
  };

  if !user.check_password(&data.password) {
    sess.data.insert("error".into(), "Incorrect password.".into());
    return Ok(Redirect::to("/login"));
  }

  cookies.add_private(Cookie::new("user_id", user.id().simple().to_string()));

  Ok(Redirect::to("lastpage"))
}
