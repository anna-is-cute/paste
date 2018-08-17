use crate::{
  config::Config,
  errors::*,
  routes::web::{context, OptionalWebUser, Session},
};

use rocket::State;

use rocket_contrib::Template;

use serde_json::json;

use std::fs::read_to_string;

#[get("/about")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Result<Template> {
  let about = match config.general.about_file {
    Some(ref f) => Some(read_to_string(f)?),
    None => None,
  };
  let mut ctx = context(&*config, user.as_ref(), &mut sess);
  ctx["about"] = json!(about);
  Ok(Template::render("about", ctx))
}
