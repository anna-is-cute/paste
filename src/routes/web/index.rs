use config::Config;

use rocket::State;

use rocket_contrib::Template;

#[get("/")]
fn get(config: State<Config>) -> Template {
  let ctx = json!({
    "config": &*config,
  });
  Template::render("index", ctx)
}
