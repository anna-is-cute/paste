use crate::{
  config::Config,
  database::{
    DbConn,
    models::{
      pastes::Paste as DbPaste,
      users::User,
    },
    schema::{pastes, users},
  },
  errors::*,
  models::paste::output::{Output, OutputAuthor},
  routes::web::{context, Links, Rst, Session},
  utils::AcceptLanguage,
};

use super::AdminUser;

use diesel::prelude::*;

use rocket::State;

use rocket_contrib::templates::Template;

use serde_json::json;

#[get("/admin/pastes")]
pub fn get(config: State<Config>, user: AdminUser, mut sess: Session, conn: DbConn, langs: AcceptLanguage) -> Result<Rst> {
  let user = user.into_inner();

  let pastes: Vec<(DbPaste, Option<User>)> = pastes::table
    .left_join(users::table)
    .order_by(pastes::created_at.desc())
    .limit(15)
    .load(&*conn)?;

  let outputs: Vec<Output> = pastes
    .into_iter()
    .map(|(paste, user)| Output::new(
      paste.id(),
      user.map(|user| OutputAuthor::new(user.id(), user.username(), user.name())),
      paste.name(),
      paste.description(),
      paste.visibility(),
      paste.created_at(),
      paste.updated_at(&*config).ok(),
      paste.expires(),
      None,
      Vec::new(),
    ))
    .collect();

  let mut ctx = context(&*config, Some(&user), &mut sess, langs);
  ctx["links"] = json!(super::admin_links()
    .add_value("paste_links", outputs
      .iter()
      .fold(&mut Links::default(), |l, x| l.add(
        x.id.to_simple().to_string(),
        uri!(
          crate::routes::web::pastes::get::users_username_id:
          x.author.as_ref().map(|x| x.username.as_str()).unwrap_or("anonymous"),
          x.id,
        )
      ))));
  ctx["pastes"] = json!(outputs);

  Ok(Rst::Template(Template::render("admin/pastes", ctx)))
}
