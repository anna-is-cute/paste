use config::Config;
use database::DbConn;
use database::models::users::NewUser;
use database::schema::users;
use errors::*;
use utils::{ReCaptcha, HashedPassword};

use diesel;
use diesel::dsl::count;
use diesel::prelude::*;

use rocket::State;
use rocket::http::{Cookies, Cookie};
use rocket::request::Form;
use rocket::response::Redirect;

use rocket_contrib::Template;

use uuid::Uuid;

use std::collections::HashMap;

#[get("/register")]
fn get(config: State<Config>) -> Template {
  let mut map = HashMap::with_capacity(1);
  map.insert("recaptcha_site_key", &config.recaptcha.site_key);
  Template::render("auth/register", map)
}

#[derive(Debug, FromForm)]
struct RegistrationData {
  name: String,
  username: String,
  email: String,
  password: HashedPassword,
  #[form(field = "g-recaptcha-response")]
  recaptcha: ReCaptcha,
}

#[post("/register", format = "application/x-www-form-urlencoded", data = "<data>")]
fn post(data: Form<RegistrationData>, mut cookies: Cookies, conn: DbConn, config: State<Config>) -> Result<Redirect> {
  let data = data.into_inner();
  // FIXME: replace this with varable before commit
  if !data.recaptcha.verify(&config.recaptcha.secret_key)? {
    // FIXME: status message
    println!("captcha fail");
    return Ok(Redirect::to("/register"));
  }

  let existing_names: i64 = users::table
    .filter(users::username.eq(&data.username))
    .select(count(users::id))
    .get_result(&*conn)?;
  if existing_names > 0 {
    println!("duplicate name");
    // FIXME: status message
    return Ok(Redirect::to("/register"));
  }

  let id = Uuid::new_v4();
  let nu = NewUser::new(
    id,
    data.username,
    data.password.into_string(),
    Some(data.name),
    Some(data.email),
  );

  diesel::insert_into(users::table).values(&nu).execute(&*conn)?;

  // FIXME: log in
  cookies.add_private(Cookie::new("user_id", id.simple().to_string()));

  Ok(Redirect::to("/"))
}
