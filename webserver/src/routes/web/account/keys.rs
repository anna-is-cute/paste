use crate::{
  config::Config,
  database::DbConn,
  errors::*,
  models::id::ApiKeyId,
  routes::web::{context, Rst, OptionalWebUser, Session},
};

use rocket::{
  request::Form,
  response::Redirect,
  State,
};
use rocket_contrib::templates::Template;

use serde_json::json;

#[get("/account/keys")]
pub fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  let mut ctx = context(&*config, Some(&user), &mut sess);
  ctx["keys"] = json!(&user.keys(&conn)?);
  Ok(Rst::Template(Template::render("account/keys", ctx)))
}

#[post("/account/keys", format = "application/x-www-form-urlencoded", data = "<new>")]
pub fn post(new: Form<NewKey>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let new = new.into_inner();

  if !sess.check_token(&new.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("/login"));
  }

  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  if new.name.is_empty() {
    sess.add_data("error", "API key name cannot be empty.");
    return Ok(Redirect::to("/account/keys"));
  }

  user.create_key(&conn, new.name)?;

  Ok(Redirect::to("/account/keys"))
}

#[derive(Debug, FromForm)]
pub struct NewKey {
  name: String,
  anti_csrf_token: String,
}

#[delete("/account/keys/<key>", data = "<data>")]
pub fn delete(key: ApiKeyId, data: Form<DeleteKey>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let data = data.into_inner();

  if !sess.check_token(&data.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("/account/keys"));
  }

  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  user.delete_key(&conn, key)?;

  Ok(Redirect::to("/account/keys"))
}

#[derive(FromForm)]
pub struct DeleteKey {
  anti_csrf_token: String,
}
