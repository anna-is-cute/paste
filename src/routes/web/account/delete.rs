use config::Config;
use database::DbConn;
use errors::*;
use routes::web::{Rst, OptionalWebUser, Session};

use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;

use rocket_contrib::Template;

#[get("/account/delete")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Result<Rst> {
  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  let ctx = json!({
    "config": &*config,
    "user": user,
    "error": sess.data.remove("error"),
    "info": sess.data.remove("info"),
    "server_version": ::SERVER_VERSION,
    "resources_version": &*::RESOURCES_VERSION,
  });
  Ok(Rst::Template(Template::render("account/delete", ctx)))
}

#[delete("/account/delete", format = "application/x-www-form-urlencoded", data = "<delete>")]
fn delete(delete: Form<DeleteRequest>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  if delete.into_inner().username != user.username() {
    sess.data.insert("error".into(), "That username does not match your username.".into());
    return Ok(Redirect::to("/account/delete"));
  }

  // TODO: sweep for unowned pastes on the disk and destroy them
  user.delete(&conn)?;

  sess.data.insert("info".into(), "Account deleted".into());
  Ok(Redirect::to("/"))
}

#[derive(Debug, FromForm)]
struct DeleteRequest {
  username: String,
}
