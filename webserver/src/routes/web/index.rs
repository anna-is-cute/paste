use config::Config;
use routes::web::{context, OptionalWebUser, Session};
use utils::Language;

use rocket::State;

use rocket_contrib::Template;

#[get("/")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Template {
  let mut ctx = context(&*config, user.as_ref(), &mut sess);
  ctx["languages"] = json!(Language::context());
  Template::render("index", ctx)
}
