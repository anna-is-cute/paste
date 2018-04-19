use rocket_contrib::Template;

use std::collections::HashMap;

#[get("/register")]
fn get() -> Template {
  let map: HashMap<String, String> = HashMap::default();
  Template::render("auth/register", map)
}
