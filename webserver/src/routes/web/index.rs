use crate::{
  config::Config,
  routes::web::{context, AddCsp, Honeypot, OptionalWebUser, Session},
  utils::Language,
};

use rocket::State;

use rocket_contrib::templates::Template;

use serde_json::json;

#[get("/")]
pub fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> AddCsp<Template> {
  let honeypot = Honeypot::new();
  let mut ctx = context(&*config, user.as_ref(), &mut sess);
  ctx["languages"] = json!(Language::context());
  ctx["honeypot"] = json!(honeypot);
  ctx["links"] = json!(links!("upload" => uri!(crate::routes::web::pastes::post::post)));
  AddCsp::new(
    Template::render("index", ctx),
    vec![format!("style-src '{}'", honeypot.integrity_hash)],
  )
}
