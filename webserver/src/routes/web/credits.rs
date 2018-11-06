use crate::{
  config::Config,
  errors::*,
  routes::web::{context, OptionalWebUser, Session},
};

use rocket::State;

use rocket_contrib::templates::Template;

#[get("/credits")]
pub fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Result<Template> {
  let ctx = context(&*config, user.as_ref(), &mut sess);
  Ok(Template::render("credits", ctx))
}
