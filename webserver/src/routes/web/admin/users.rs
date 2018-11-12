use crate::{
  config::Config,
  database::{
    DbConn,
    models::users::User,
    schema::users,
  },
  errors::*,
  routes::web::{context, Rst, Session},
};

use super::AdminUser;

use diesel::prelude::*;

use rocket::State;

use rocket_contrib::templates::Template;

use serde_json::json;

#[get("/admin/users")]
pub fn get(config: State<Config>, user: AdminUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  let user = user.into_inner();

  let users: Vec<User> = users::table
    .limit(15)
    .load(&*conn)?;

  let mut ctx = context(&*config, Some(&user), &mut sess);
  ctx["links"] = json!(super::admin_links());
  ctx["users"] = json!(users);

  Ok(Rst::Template(Template::render("admin/users", ctx)))
}
