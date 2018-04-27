use config::Config;
use database::DbConn;
use database::models::users::NewUser;
use database::schema::users;
use errors::*;
use routes::web::{Rst, OptionalWebUser, Session};
use utils::{ReCaptcha, HashedPassword};

use diesel;
use diesel::dsl::count;
use diesel::prelude::*;

use rocket::State;
use rocket::http::{Cookies, Cookie};
use rocket::request::Form;
use rocket::response::Redirect;

use rocket_contrib::Template;

use unicode_segmentation::UnicodeSegmentation;

use uuid::Uuid;

#[get("/register")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Rst {
  if user.is_some() {
    return Rst::Redirect(Redirect::to("/"));
  }
  let ctx = json!({
    "config": &*config,
    "error": sess.data.remove("error"),
    "server_version": ::SERVER_VERSION,
    "resources_version": &*::RESOURCES_VERSION,
  });
  Rst::Template(Template::render("auth/register", ctx))
}

#[derive(Debug, FromForm)]
struct RegistrationData {
  name: String,
  username: String,
  email: String,
  password: String,
  password_verify: String,
  #[form(field = "g-recaptcha-response")]
  recaptcha: ReCaptcha,
}

#[post("/register", format = "application/x-www-form-urlencoded", data = "<data>")]
fn post(data: Form<RegistrationData>, mut sess: Session, mut cookies: Cookies, conn: DbConn, config: State<Config>) -> Result<Redirect> {
  let data = data.into_inner();

  if data.username.is_empty() || data.name.is_empty()  || data.email.is_empty() || data.password.is_empty() {
    sess.data.insert("error".into(), "No fields can be empty.".into());
    return Ok(Redirect::to("/register"));
  }
  if data.username == "anonymous" {
    sess.data.insert("error".into(), r#"Username cannot be "anonymous"."#.into());
    return Ok(Redirect::to("/register"));
  }

  if data.password != data.password_verify {
    sess.data.insert("error".into(), "Passwords did not match.".into());
    return Ok(Redirect::to("/register"));
  }

  if data.password.graphemes(true).count() < 10 {
    sess.data.insert("error".into(), "Password must be at least 10 characters long.".into());
    return Ok(Redirect::to("/register"));
  }
  if data.password == data.name || data.password == data.username || data.password == data.email || data.password == "password" {
    sess.data.insert("error".into(), r#"Password cannot be the same as your name, username, email, or "password"."#.into());
    return Ok(Redirect::to("/register"));
  }

  if !data.recaptcha.verify(&config.recaptcha.secret_key)? {
    sess.data.insert("error".into(), "The captcha did not validate. Try again.".into());
    return Ok(Redirect::to("/register"));
  }

  let existing_names: i64 = users::table
    .filter(users::username.eq(&data.username))
    .select(count(users::id))
    .get_result(&*conn)?;
  if existing_names > 0 {
    sess.data.insert("error".into(), "A user with that username already exists.".into());
    return Ok(Redirect::to("/register"));
  }

  let id = Uuid::new_v4();
  let nu = NewUser::new(
    id,
    data.username,
    HashedPassword::from(data.password).into_string(),
    Some(data.name),
    Some(data.email),
  );

  diesel::insert_into(users::table).values(&nu).execute(&*conn)?;

  cookies.add_private(Cookie::new("user_id", id.simple().to_string()));

  Ok(Redirect::to("lastpage"))
}
