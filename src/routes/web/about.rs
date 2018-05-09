use config::Config;
use errors::*;
use routes::web::{OptionalWebUser, Session};

use rocket::State;

use rocket_contrib::Template;

use std::fs::read_to_string;

#[get("/about")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Result<Template> {
  let about = match config.general.about_file {
    Some(ref f) => Some(read_to_string(f)?),
    None => None,
  };
  let ctx = json!({
    "config": &*config,
    "user": &*user,
    "error": sess.data.remove("error"),
    "info": sess.data.remove("info"),
    "server_version": ::SERVER_VERSION,
    "resources_version": &*::RESOURCES_VERSION,
    "about": about,
  });
  Ok(Template::render("about", ctx))
}
