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
  // let mut links = crate::routes::web::Links::default();
  // if let Some(ref u) = *user {
  //   links.add("user", uri!(crate::routes::web::users::get::get: u.username(), None).to_string());
  // }
  ctx["languages"] = json!(Language::context());
  ctx["honeypot"] = json!(honeypot);
  // ctx["links"] = json!(links);
  AddCsp::new(
    Template::render("index", ctx),
    vec![format!("style-src '{}'", honeypot.integrity_hash)],
  )
}
