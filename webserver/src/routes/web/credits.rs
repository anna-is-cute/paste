use crate::{
  config::Config,
  errors::*,
  routes::web::{context, OptionalWebUser, Session},
  utils::AcceptLanguage,
};

use rocket::State;

use rocket_contrib::templates::Template;

#[get("/credits")]
pub fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session, langs: AcceptLanguage) -> Result<Template> {
  let ctx = context(&*config, user.as_ref(), &mut sess, langs);
  Ok(Template::render("credits", ctx))
}
