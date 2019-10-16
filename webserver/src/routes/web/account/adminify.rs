use crate::{
  config::Config,
  database::{
    DbConn,
    schema::users,
  },
  errors::*,
  models::user::Admin,
  routes::web::{OptionalWebUser, Session},
};

use diesel::prelude::*;

use rocket::{
  response::Redirect,
  State,
};

#[get("/account/adminify?<key>")]
pub fn get(key: String, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let mut user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get))),
  };

  let config = config.read();

  let correct_key = match config.admin.key {
    Some(ref k) => k,
    None => {
      sess.add_data("error", "No admin key is set.");
      return Ok(Redirect::to("lastpage"));
    },
  };

  if key != *correct_key {
    sess.add_data("error", "Incorrect key.");
    return Ok(Redirect::to("lastpage"));
  }

  if user.is_admin() {
    sess.add_data("error", "You're already an admin.");
    return Ok(Redirect::to("lastpage"));
  }

  let num_admins: i64 = users::table
    .select(diesel::dsl::count_star())
    .filter(users::admin.gt(Admin::None))
    .first(&*conn)?;

  if num_admins > 0 {
    sess.add_data("error", "You can't become an admin this way if there are already admins.");
    return Ok(Redirect::to("lastpage"));
  }

  user.set_admin(Admin::Super);
  user.update(&conn)?;

  sess.add_data("info", "You are now an admin.");
  Ok(Redirect::to("lastpage"))
}
