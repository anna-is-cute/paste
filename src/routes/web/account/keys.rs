use config::Config;
use database::DbConn;
use errors::*;
use routes::web::{Rst, OptionalWebUser};

use rocket::http::{Cookies, Cookie};
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;

use rocket_contrib::{Template, UUID};

#[get("/account/keys")]
fn get(config: State<Config>, user: OptionalWebUser, mut cookies: Cookies, conn: DbConn) -> Result<Rst> {
  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  let ctx = json!({
    "config": &*config,
    "user": user,
    "error": cookies.get_private("error").map(|x| x.value()),
    "server_version": ::SERVER_VERSION,
    "resources_version": &*::RESOURCES_VERSION,
    "keys": &user.keys(&conn)?,
  });
  cookies.remove_private(Cookie::named("error"));
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
    cookies.add_private(Cookie::new("error", "API key name cannot be empty"));
    return Ok(Redirect::to("/account/keys"));
  }

  user.create_key(&conn, new.name)?;

  Ok(Redirect::to("/account/keys"))
}

#[derive(Debug, FromForm)]
struct NewKey {
  name: String,
}

#[post("/account/keys/<key>/delete")]
fn delete(key: UUID, user: OptionalWebUser, conn: DbConn) -> Result<Redirect> {
  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  user.delete_key(&conn, *key)?;

  Ok(Redirect::to("/account/keys"))
}
