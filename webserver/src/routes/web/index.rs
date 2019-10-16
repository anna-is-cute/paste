use crate::{
  config::Config,
  routes::web::{context, AddCsp, AntiSpam, Honeypot, OptionalWebUser, Session},
  utils::{AcceptLanguage, Language},
};

use rocket::State;

use rocket_contrib::templates::Template;

use serde_json::json;

#[get("/")]
pub fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session, antispam: AntiSpam, langs: AcceptLanguage) -> AddCsp<Template> {
  if config.read().pastes.sign_in_to_create && user.is_none() {
    let ctx = context(&*config, user.as_ref(), &mut sess, langs);
    // TODO: Not use AddCsp for this
    return AddCsp::new(
      Template::render("index_no_create", ctx),
      Vec::<String>::new(),
    );
  }
  let honeypot = Honeypot::new();
  let mut ctx = context(&*config, user.as_ref(), &mut sess, langs);
  ctx["languages"] = json!(Language::context());
  ctx["honeypot"] = json!(honeypot);
  ctx["antispam"] = json!(antispam);
  ctx["links"] = json!(links!("upload" => uri!(crate::routes::web::pastes::post::post)));
  AddCsp::new(
    Template::render("index", ctx),
    vec![
      format!("style-src '{}'", honeypot.integrity_hash),
      format!("script-src '{}'", antispam.integrity_hash),
    ],
  )
}
