use config::Config;
use routes::web::{OptionalWebUser, Session};

use rocket::State;

use rocket_contrib::Template;

#[get("/")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Template {
  let ctx = json!({
    "config": &*config,
    "user": &*user,
    "error": sess.data.remove("error"),
    "server_version": ::SERVER_VERSION,
    "resources_version": &*::RESOURCES_VERSION,
  });
  Template::render("index", ctx)
}
