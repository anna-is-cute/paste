use crate::{
  config::Config,
  database::DbConn,
  errors::*,
  models::id::ApiKeyId,
  routes::web::{context, Links, Rst, OptionalWebUser, Session},
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
    None => return Ok(Rst::Redirect(Redirect::to(uri!(crate::routes::web::auth::login::get)))),
  };

  let keys = user.keys(&conn)?;

  let mut ctx = context(&*config, Some(&user), &mut sess);
  ctx["keys"] = json!(&keys);
  ctx["links"] = json!(
    links!(super::account_links(),
      "add_key" => uri!(crate::routes::web::account::keys::post),
    ).add_value(
      "delete_key_links",
      keys
        .iter()
        .fold(&mut Links::default(), |l, x| l.add(x.key.to_simple().to_string(), uri!(
          crate::routes::web::account::keys::delete:
          x.key,
        )))
    )
  );
  Ok(Rst::Template(Template::render("account/keys", ctx)))
}

#[post("/account/keys", format = "application/x-www-form-urlencoded", data = "<new>")]
pub fn post(new: Form<NewKey>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let new = new.into_inner();

  if !sess.check_token(&new.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get)));
  }

  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get))),
  };

  if new.name.is_empty() {
    sess.add_data("error", "API key name cannot be empty.");
    return Ok(Redirect::to(uri!(get)));
  }

  user.create_key(&conn, new.name)?;

  Ok(Redirect::to(uri!(get)))
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
    return Ok(Redirect::to(uri!(get)));
  }

  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get))),
  };

  user.delete_key(&conn, key)?;

  Ok(Redirect::to(uri!(get)))
}

#[derive(FromForm)]
pub struct DeleteKey {
  anti_csrf_token: String,
}
