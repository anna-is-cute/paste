use config::Config;
use routes::web::{context, OptionalWebUser, Session};

use rocket::State;

use rocket_contrib::Template;

#[get("/")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Template {
  let ctx = context(&*config, user.as_ref(), &mut sess);
  Template::render("index", ctx)
}
