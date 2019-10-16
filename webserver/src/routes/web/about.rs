use crate::{
  config::Config,
  errors::*,
  routes::web::{context, OptionalWebUser, Session},
  utils::AcceptLanguage,
};

use rocket::State;

use rocket_contrib::templates::Template;

use serde_json::json;

use std::fs::read_to_string;

#[get("/about")]
pub fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session, langs: AcceptLanguage) -> Result<Template> {
  let about = match config.read().general.about_file {
    Some(ref f) => Some(read_to_string(f)?),
    None => None,
  };
  let mut ctx = context(&*config, user.as_ref(), &mut sess, langs);
  ctx["about"] = json!(about);
  Ok(Template::render("about", ctx))
}
