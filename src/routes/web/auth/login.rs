use config::Config;
use database::DbConn;
use database::models::users::User;
use database::schema::users;
use errors::*;

use diesel::prelude::*;

use rocket::State;
use rocket::http::{Cookies, Cookie};
use rocket::request::Form;
use rocket::response::Redirect;

use rocket_contrib::Template;

use sodiumoxide::crypto::pwhash;

#[get("/login")]
fn get(config: State<Config>, mut cookies: Cookies) -> Template {
  let ctx = json!({
    "config": &*config,
    // TODO: this can be made into an optional request guard
    "error": cookies.get("error").map(|x| x.value()),
  });
  cookies.remove(Cookie::named("error"));
  Template::render("auth/login", ctx)
}

#[derive(Debug, FromForm)]
struct RegistrationData {
  username: String,
  password: String,
}

#[post("/login", format = "application/x-www-form-urlencoded", data = "<data>")]
fn post(data: Form<RegistrationData>, mut cookies: Cookies, conn: DbConn) -> Result<Redirect> {
  let data = data.into_inner();

  let user: Option<User> = users::table
    .filter(users::username.eq(&data.username))
    .first(&*conn)
    .optional()?;

  let user = match user {
    Some(u) => u,
    None => {
      cookies.add(Cookie::new("error", "Username not found."));
      return Ok(Redirect::to("/login"));
    },
  };

  let mut stored_bytes = user.password().clone().into_bytes();
  stored_bytes.push(0);
  let hash = pwhash::HashedPassword::from_slice(&stored_bytes).expect("hashed password");
  if !pwhash::pwhash_verify(&hash, data.password.as_bytes()) {
    cookies.add(Cookie::new("error", "Incorrect password."));
    return Ok(Redirect::to("/login"));
  }

  cookies.add_private(Cookie::new("user_id", user.id().simple().to_string()));

  Ok(Redirect::to("/"))
}
