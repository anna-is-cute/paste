use config::Config;
use database::DbConn;
use errors::*;
use routes::web::OptionalWebUser;

use rocket::http::{Cookies, Cookie, Status as HttpStatus};
use rocket::request::{Request, Form};
use rocket::response::{Redirect, Responder, Response};
use rocket::State;

use rocket_contrib::Template;

use utils::HashedPassword;

use std::result;

enum Rt {
  Redirect(Redirect),
  Template(Template),
}

impl<'r> Responder<'r> for Rt {
  fn respond_to(self, request: &Request) -> result::Result<Response<'r>, HttpStatus> {
    match self {
      Rt::Redirect(r) => r.respond_to(request),
      Rt::Template(t) => t.respond_to(request),
    }
  }
}

#[get("/account")]
fn get(config: State<Config>, user: OptionalWebUser, mut cookies: Cookies) -> Result<Rt> {
  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Rt::Redirect(Redirect::to("/login"))),
  };

  let ctx = json!({
    "config": &*config,
    "user": user,
    "error": cookies.get("error").map(|x| x.value()),
  });
  cookies.remove(Cookie::named("error"));
  Ok(Rt::Template(Template::render("account/index", ctx)))
}

#[post("/account", format = "application/x-www-form-urlencoded", data = "<update>")]
fn post(update: Form<AccountUpdate>, user: OptionalWebUser, mut cookies: Cookies, conn: DbConn) -> Result<Redirect> {
  let mut user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to("/login")),
  };
  println!("{:#?}", update);

  let update = update.into_inner();

  if update.current_password.is_empty() {
    cookies.add(Cookie::new("error", "Current password cannot be empty."));
    return Ok(Redirect::to("/account"));
  }

  if !user.check_password(&update.current_password) {
    cookies.add(Cookie::new("error", "Incorrect password."));
    return Ok(Redirect::to("/account"));
  }

  if !update.new_password.is_empty() {
    let hashed = HashedPassword::from(&update.new_password).into_string();
    user.set_password(hashed);
  }

  if !update.email.is_empty() {
    user.set_email(update.email);
  }

  if !update.name.is_empty() {
    user.set_name(update.name);
  }

  if !update.username.is_empty() {
    // FIXME: check for conflicts
    user.set_username(update.username);
  }

  user.update(&conn)?;

  Ok(Redirect::to("/account"))
}

#[derive(Debug, FromForm)]
struct AccountUpdate {
  name: String,
  username: String,
  email: String,
  new_password: String,
  current_password: String,
}
