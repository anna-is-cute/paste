use config::Config;
use database::DbConn;
use database::schema::users;
use errors::*;
use routes::web::{context, Rst, OptionalWebUser, Session};
use utils::{HashedPassword, Validator};

use diesel::dsl::count;
use diesel::prelude::*;

use rocket::request::Form;
use rocket::response::Redirect;
use rocket::State;

use rocket_contrib::Template;

use unicode_segmentation::UnicodeSegmentation;

#[get("/account")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Result<Rst> {
  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  let ctx = context(&*config, Some(&user), &mut sess);
  Ok(Rst::Template(Template::render("account/index", ctx)))
}

#[patch("/account", format = "application/x-www-form-urlencoded", data = "<update>")]
fn patch(update: Form<AccountUpdate>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let update = update.into_inner();

  if !sess.check_token(&update.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("/account"));
  }

  let mut user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to("/login")),
  };


  if update.current_password.is_empty() {
    sess.add_data("error", "Current password cannot be empty.");
    return Ok(Redirect::to("/account"));
  }

  if !user.check_password(&update.current_password) {
    sess.add_data("error", "Incorrect password.");
    return Ok(Redirect::to("/account"));
  }

  if !update.email.is_empty() {
    user.set_email(update.email);
  }

  if !update.name.is_empty() {
    let name = match Validator::validate_display_name(&update.name) {
      Ok(n) => n,
      Err(e) => {
        sess.add_data("error", format!("Invalid display name: {}.", e));
        return Ok(Redirect::to("/account"));
      },
    };
    user.set_name(name.into_owned());
  }

  if !update.username.is_empty() {
    let username = match Validator::validate_username(&update.username) {
      Ok(n) => n,
      Err(e) => {
        sess.add_data("error", format!("Invalid username: {}.", e));
        return Ok(Redirect::to("/account"));
      },
    };
    // FIXME: refactor this logic out
    let existing_names: i64 = users::table
      .filter(users::username.eq(&username))
      .select(count(users::id))
      .get_result(&*conn)?;
    if existing_names > 0 {
      sess.add_data("error", "A user with that username already exists.");
      return Ok(Redirect::to("/account"));
    }
    user.set_username(username.into_owned());
  }

  if !update.password.is_empty() {
    if update.password != update.password_verify {
      sess.add_data("error", "New passwords did not match.");
      return Ok(Redirect::to("/account"));
    }
    if update.password.graphemes(true).count() < 10 {
      sess.add_data("error", "New password must be at least 10 characters long.");
      return Ok(Redirect::to("/account"));
    }
    if update.password == user.name() || update.password == user.username() || update.password == user.email() || update.password == "password" {
      sess.add_data("error", r#"New password cannot be your name, user, email, or "password"."#);
      return Ok(Redirect::to("/account"));
    }
    let hashed = HashedPassword::from(&update.password).into_string();
    user.set_password(hashed);
  }

  user.update(&conn)?;

  sess.add_data("info", "Account updated.");
  Ok(Redirect::to("/account"))
}

#[derive(Debug, FromForm)]
struct AccountUpdate {
  name: String,
  username: String,
  email: String,
  password: String,
  password_verify: String,
  current_password: String,
  anti_csrf_token: String,
}
