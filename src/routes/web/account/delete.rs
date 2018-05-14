use config::Config;
use database::DbConn;
use errors::*;
use routes::web::{context, Rst, OptionalWebUser, Session};

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

  let ctx = context(&*config, Some(&user), &mut sess);
  Ok(Rst::Template(Template::render("account/delete", ctx)))
}

#[delete("/account", format = "application/x-www-form-urlencoded", data = "<delete>")]
fn delete(delete: Form<DeleteRequest>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let delete = delete.into_inner();

  if !sess.check_token(&delete.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("/account/delete"));
  }

  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  if delete.username != user.username() {
    sess.add_data("error", "That username does not match your username.");
    return Ok(Redirect::to("/account/delete"));
  }

  // TODO: sweep for unowned pastes on the disk and destroy them
  user.delete(&conn)?;

  sess.add_data("info", "Account deleted.");
  Ok(Redirect::to("/"))
}

#[derive(Debug, FromForm)]
struct DeleteRequest {
  username: String,
  anti_csrf_token: String,
}
