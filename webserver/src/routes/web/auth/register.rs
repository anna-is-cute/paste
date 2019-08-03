use crate::{
  config::Config,
  database::{
    DbConn,
    models::users::{User, NewUser},
    schema::users,
  },
  errors::*,
  models::id::UserId,
  routes::web::{context, AddCsp, Honeypot, Rst, OptionalWebUser, Session},
  utils::{email, AcceptLanguage, PasswordContext, HashedPassword, Validator},
};

use chrono::Utc;

use diesel::{dsl::count, prelude::*};

use rocket::{
  State,
  request::Form,
  response::Redirect,
};

use rocket_contrib::templates::Template;

use serde_json::json;

use sidekiq::Client as SidekiqClient;

use uuid::Uuid;

#[get("/register")]
pub fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session, langs: AcceptLanguage) -> AddCsp<Rst> {
  if user.is_some() {
    return AddCsp::none(Rst::Redirect(Redirect::to(uri!(crate::routes::web::index::get))));
  }

  let honeypot = Honeypot::new();
  let mut ctx = context(&*config, user.as_ref(), &mut sess, langs);
  ctx["honeypot"] = json!(honeypot);
  ctx["links"] = json!(links!(
    "register_action" => uri!(crate::routes::web::auth::register::post),
  ));
  AddCsp::new(
    Rst::Template(Template::render("auth/register", ctx)),
    vec![format!("style-src '{}'", honeypot.integrity_hash)],
  )
}

#[derive(Debug, FromForm, Serialize)]
pub struct RegistrationData {
  name: String,
  username: String,
  email: String,
  #[serde(skip)]
  password: String,
  #[serde(skip)]
  password_verify: String,
  #[serde(skip)]
  anti_csrf_token: String,
  #[serde(skip)]
  #[form(field = "title")]
  honeypot: String,
}

#[post("/register", format = "application/x-www-form-urlencoded", data = "<data>")]
pub fn post(data: Form<RegistrationData>, mut sess: Session, conn: DbConn, config: State<Config>, sidekiq: State<SidekiqClient>) -> Result<Redirect> {
  let data = data.into_inner();
  sess.set_form(&data);

  if !sess.check_token(&data.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to(uri!(get)));
  }

  if !data.honeypot.is_empty() {
    sess.add_data("error", "An error occurred. Please try again.");
    return Ok(Redirect::to(uri!(get)));
  }

  if data.username.is_empty() || data.name.is_empty()  || data.email.is_empty() || data.password.is_empty() {
    sess.add_data("error", "No fields can be empty.");
    return Ok(Redirect::to(uri!(get)));
  }
  let username = match Validator::validate_username(&data.username) {
    Ok(u) => u,
    Err(e) => {
      sess.add_data("error", format!("Invalid username: {}.", e));
      return Ok(Redirect::to(uri!(get)));
    },
  };
  let display_name = match Validator::validate_display_name(&data.name) {
    Ok(d) => d.into_owned(),
    Err(e) => {
      sess.add_data("error", format!("Invalid display name: {}.", e));
      return Ok(Redirect::to(uri!(get)));
    },
  };

  if !email::check_email(&data.email) {
    sess.add_data("error", "Invalid email.");
    return Ok(Redirect::to(uri!(get)));
  }

  {
    let pw_ctx = PasswordContext::new(
      &data.password,
      &data.password_verify,
      &data.name,
      &data.username,
      &data.email,
    );
    if let Err(e) = pw_ctx.validate() {
      sess.add_data("error", e);
      return Ok(Redirect::to(uri!(get)));
    }
  }

  let existing_names: i64 = users::table
    .filter(users::username.eq(&username))
    .select(count(users::id))
    .get_result(&*conn)?;
  if existing_names > 0 {
    sess.add_data("error", "A user with that username already exists.");
    return Ok(Redirect::to(uri!(get)));
  }

  let id = UserId(Uuid::new_v4());

  let nu = NewUser::new(
    id,
    username.into_owned(),
    HashedPassword::from(data.password).into_string(),
    Some(display_name),
    Some(data.email),
  );

  let user: User = diesel::insert_into(users::table)
    .values(&nu)
    .get_result(&*conn)?;

  let (ver, secret) = user.create_email_verification(&conn, Some(Utc::now().naive_utc()))?;

  sidekiq.push(ver.job(&*config, &user, &secret)?.into())?;

  sess.user_id = Some(id);

  sess.take_form();
  Ok(Redirect::to("lastpage"))
}
