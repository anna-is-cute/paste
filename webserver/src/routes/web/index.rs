use crate::{
  config::Config,
  routes::web::{context, OptionalWebUser, Session},
  utils::Language,
};

use rocket::State;

use rocket_contrib::Template;

use serde_json::{json, json_internal};

#[get("/")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Template {
  let mut ctx = context(&*config, user.as_ref(), &mut sess);
  ctx["languages"] = json!(Language::context());
  Template::render("index", ctx)
}
