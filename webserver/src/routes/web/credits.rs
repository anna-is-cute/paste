use config::Config;
use errors::*;
use routes::web::{context, OptionalWebUser, Session};

use rocket::State;

use rocket_contrib::Template;

#[get("/credits")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Result<Template> {
  let ctx = context(&*config, user.as_ref(), &mut sess);
  Ok(Template::render("credits", ctx))
}
