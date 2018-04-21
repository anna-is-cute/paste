use config::Config;
use routes::web::OptionalWebUser;

use rocket::State;

use rocket_contrib::Template;

#[get("/")]
fn get(config: State<Config>, user: OptionalWebUser) -> Template {
  let ctx = json!({
    "config": &*config,
    "user": &*user,
  });
  Template::render("index", ctx)
}
