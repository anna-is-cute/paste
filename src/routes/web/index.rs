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
    "error": cookies.get("error").map(|x| x.value()),
    "version": ::VERSION,
  });
  cookies.remove(Cookie::named("error"));
  Template::render("index", ctx)
}
