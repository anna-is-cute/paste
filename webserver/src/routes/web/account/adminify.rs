use crate::{
  config::Config,
  database::{
    DbConn,
    schema::users,
  },
  errors::*,
  i18n::prelude::*,
  models::user::Admin,
  routes::web::{OptionalWebUser, Session},
};

use diesel::prelude::*;

use rocket::{
  response::Redirect,
  State,
};

#[get("/account/adminify?<key>")]
pub fn get(key: String, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn, l10n: L10n) -> Result<Redirect> {
  let mut user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to(uri!(crate::routes::web::auth::login::get))),
  };

  let config = config.read();

  let correct_key = match config.admin.key {
    Some(ref k) => k,
    None => {
      sess.add_data("error", l10n.tr("admin-no-key")?);
      return Ok(Redirect::to("lastpage"));
    },
  };

  if key != *correct_key {
    sess.add_data("error", l10n.tr("admin-bad-key")?);
    return Ok(Redirect::to("lastpage"));
  }

  if user.is_admin() {
    sess.add_data("error", l10n.tr("admin-already-admin")?);
    return Ok(Redirect::to("lastpage"));
  }

  let num_admins: i64 = users::table
    .select(diesel::dsl::count_star())
    .filter(users::admin.gt(Admin::None))
    .first(&*conn)?;

  if num_admins > 0 {
    sess.add_data("error", l10n.tr("admin-exists")?);
    return Ok(Redirect::to("lastpage"));
  }

  user.set_admin(Admin::Super);
  user.update(&conn)?;

  sess.add_data("info", l10n.tr("admin-success")?);
  Ok(Redirect::to("lastpage"))
}
