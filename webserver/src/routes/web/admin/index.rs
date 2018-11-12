use crate::{
  config::Config,
  database::{DbConn, schema::pastes, schema::users},
  errors::*,
  routes::web::{context, Rst, Session},
};

use super::AdminUser;

use diesel::prelude::*;

use rocket::State;

use rocket_contrib::templates::Template;

use serde_json::json;

#[get("/admin")]
pub fn get(config: State<Config>, user: AdminUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  let user = user.into_inner();

  let total_pastes: i64 = pastes::table
    .select(diesel::dsl::count_star())
    .first(&*conn)?;
  let total_users: i64 = users::table
    .select(diesel::dsl::count_star())
    .first(&*conn)?;

  let mut ctx = context(&*config, Some(&user), &mut sess);
  ctx["links"] = json!(super::admin_links());
  ctx["total_pastes"] = json!(total_pastes);
  ctx["total_users"] = json!(total_users);

  Ok(Rst::Template(Template::render("admin/index", ctx)))
}
