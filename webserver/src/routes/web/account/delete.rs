use crate::{
  config::Config,
  database::DbConn,
  errors::*,
  routes::web::{context, Rst, OptionalWebUser, Session},
  sidekiq::Job,
};

use rocket::{
  request::Form,
  response::Redirect,
  State,
};

use rocket_contrib::templates::Template;

use sidekiq::Client as SidekiqClient;

#[get("/account/delete")]
pub fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Result<Rst> {
  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Rst::Redirect(Redirect::to(uri!(crate::routes::web::auth::login::get)))),
  };

  let ctx = context(&*config, Some(&user), &mut sess);
  Ok(Rst::Template(Template::render("account/delete", ctx)))
}

#[delete("/account", format = "application/x-www-form-urlencoded", data = "<delete>")]
pub fn delete(delete: Form<DeleteRequest>, user: OptionalWebUser, mut sess: Session, conn: DbConn, sidekiq: State<SidekiqClient>) -> Result<Redirect> {
  let delete = delete.into_inner();

  if !sess.check_token(&delete.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to(uri!(get)));
  }

  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get))),
  };

  if !user.check_password(&delete.password) {
    sess.add_data("error", "Incorrect password.");
    return Ok(Redirect::to(uri!(get)));
  }

  user.delete(&conn)?;

  sidekiq.push(Job::DeleteAllPastes(user.id()).into())?;

  sess.add_data("info", "Account deleted.");
  Ok(Redirect::to(uri!(crate::routes::web::index::get)))
}

#[derive(Debug, FromForm)]
pub struct DeleteRequest {
  password: String,
  anti_csrf_token: String,
}
