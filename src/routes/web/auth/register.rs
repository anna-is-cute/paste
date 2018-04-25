use config::Config;
use database::DbConn;
use database::models::users::NewUser;
use database::schema::users;
use errors::*;
use routes::web::{Rst, OptionalWebUser};
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

#[get("/register")]
fn get(config: State<Config>, user: OptionalWebUser, mut cookies: Cookies) -> Rst {
  if user.is_some() {
    return Rst::Redirect(Redirect::to("/"));
  }
  let ctx = json!({
    "config": &*config,
    "error": cookies.get_private("error").map(|x| x.value()),
    "server_version": ::SERVER_VERSION,
    "resources_version": &*::RESOURCES_VERSION,
  });
  cookies.remove_private(Cookie::named("error"));
  Rst::Template(Template::render("auth/register", ctx))
}

#[derive(Debug, FromForm)]
struct RegistrationData {
  name: String,
  username: String,
  email: String,
  password: String,
  #[form(field = "g-recaptcha-response")]
  recaptcha: ReCaptcha,
}

#[post("/register", format = "application/x-www-form-urlencoded", data = "<data>")]
fn post(data: Form<RegistrationData>, mut cookies: Cookies, conn: DbConn, config: State<Config>) -> Result<Redirect> {
  let data = data.into_inner();

  if data.username.is_empty() || data.name.is_empty()  || data.email.is_empty() || data.password.is_empty() {
    cookies.add_private(Cookie::new("error", "No fields can be empty."));
    return Ok(Redirect::to("/register"));
  }
  if data.username == "anonymous" {
    cookies.add_private(Cookie::new("error", r#"Username cannot be "anonymous"."#));
    return Ok(Redirect::to("/register"));
  }
  if data.password == data.username || data.password == data.email || data.password == "password" {
    cookies.add_private(Cookie::new("error", r#"Password cannot be the same as your username, email, or "password"."#));
    return Ok(Redirect::to("/register"));
  }
  if !data.recaptcha.verify(&config.recaptcha.secret_key)? {
    cookies.add_private(Cookie::new("error", "The captcha did not validate. Try again."));
    return Ok(Redirect::to("/register"));
  }

  let existing_names: i64 = users::table
    .filter(users::username.eq(&data.username))
    .select(count(users::id))
    .get_result(&*conn)?;
  if existing_names > 0 {
    cookies.add_private(Cookie::new("error", "A user with that username already exists."));
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
