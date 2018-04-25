use config::Config;
use routes::web::OptionalWebUser;

use rocket::http::{Cookies, Cookie};
use rocket::State;

use rocket_contrib::Template;

#[get("/")]
fn get(config: State<Config>, user: OptionalWebUser, mut cookies: Cookies) -> Template {
  let ctx = json!({
    "config": &*config,
    "user": &*user,
    "error": cookies.get_private("error").map(|x| x.value().to_string()),
    "server_version": ::SERVER_VERSION,
    "resources_version": &*::RESOURCES_VERSION,
  });
  cookies.remove_private(Cookie::named("error"));
  Template::render("index", ctx)
}
