use config::Config;
use database::DbConn;
use errors::*;
use models::id::ApiKeyId;
use routes::web::{context, AntiCsrfToken, Rst, OptionalWebUser, Session};

use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;

use rocket_contrib::Template;

#[get("/account/keys")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  let mut ctx = context(&*config, Some(&user), &mut sess);
  ctx["keys"] = json!(&user.keys(&conn)?);
  Ok(Rst::Template(Template::render("account/keys", ctx)))
}

#[post("/account/keys", format = "application/x-www-form-urlencoded", data = "<new>")]
fn post(new: Form<NewKey>, csrf: AntiCsrfToken, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let new = new.into_inner();

  if !csrf.check(&new.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("/login"));
  }

  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  if new.name.is_empty() {
    sess.data.insert("error".into(), "API key name cannot be empty.".into());
    return Ok(Redirect::to("/account/keys"));
  }

  user.create_key(&conn, new.name)?;

  Ok(Redirect::to("/account/keys"))
}

#[derive(Debug, FromForm)]
struct NewKey {
  name: String,
  anti_csrf_token: String,
}

#[delete("/account/keys/<key>", data = "<data>")]
fn delete(key: ApiKeyId, data: Form<DeleteKey>, csrf: AntiCsrfToken, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let data = data.into_inner();

  if !csrf.check(&data.anti_csrf_token) {
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
struct DeleteKey {
  anti_csrf_token: String,
}
