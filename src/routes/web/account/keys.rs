use config::Config;
use database::DbConn;
use database::models::api_keys::ApiKey;
use database::schema::api_keys;
use errors::*;
use routes::web::{Rst, OptionalWebUser};

use diesel::dsl::count;
use diesel::prelude::*;

use rocket::http::{Cookies, Cookie};
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;

use rocket_contrib::Template;

use utils::HashedPassword;

#[get("/account/keys")]
fn get(config: State<Config>, user: OptionalWebUser, mut cookies: Cookies, conn: DbConn) -> Result<Rst> {
  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  let ctx = json!({
    "config": &*config,
    "user": user,
    "error": cookies.get("error").map(|x| x.value()),
    "keys": &user.keys(&conn)?,
  });
  cookies.remove(Cookie::named("error"));
  Ok(Rst::Template(Template::render("account/keys", ctx)))
}

#[post("/account/keys", format = "application/x-www-form-urlencoded", data = "<new>")]
fn post(new: Form<NewKey>, user: OptionalWebUser, mut cookies: Cookies, conn: DbConn) -> Result<Redirect> {
  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  let new = new.into_inner();
  if new.name.is_empty() {
    cookies.add(Cookie::new("error", "API key name cannot be empty"));
    return Ok(Redirect::to("/account/keys"));
  }

  user.create_key(&conn, new.name)?;

  Ok(Redirect::to("/account/keys"))
}

#[derive(Debug, FromForm)]
struct NewKey {
  name: String,
}
