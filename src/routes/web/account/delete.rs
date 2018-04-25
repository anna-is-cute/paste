use config::Config;
use database::DbConn;
use errors::*;
use routes::web::{Rst, OptionalWebUser};

use rocket::http::{Cookies, Cookie};
use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;

use rocket_contrib::Template;

#[get("/account/delete")]
fn get(config: State<Config>, user: OptionalWebUser, mut cookies: Cookies) -> Result<Rst> {
  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  let ctx = json!({
    "config": &*config,
    "user": user,
    "error": cookies.get_private("error").map(|x| x.value().to_string()),
    "server_version": ::SERVER_VERSION,
    "resources_version": &*::RESOURCES_VERSION,
  });
  cookies.remove_private(Cookie::named("error"));
  Ok(Rst::Template(Template::render("account/delete", ctx)))
}

#[post("/account/delete", format = "application/x-www-form-urlencoded", data = "<delete>")]
fn post(delete: Form<DeleteRequest>, user: OptionalWebUser, mut cookies: Cookies, conn: DbConn) -> Result<Redirect> {
  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  if delete.into_inner().username != user.username() {
    cookies.add_private(Cookie::new("error", "That username does not match your username."));
    return Ok(Redirect::to("/account/delete"));
  }

  // TODO: sweep for unowned pastes on the disk and destroy them
  user.delete(&conn)?;

  Ok(Redirect::to("/"))
}

#[derive(Debug, FromForm)]
struct DeleteRequest {
  username: String,
}
