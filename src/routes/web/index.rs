use std::collections::HashMap;

use rocket_contrib::Template;

#[get("/")]
fn get() -> Template {
  let mut map: HashMap<String, String> = HashMap::default();
  map.insert("site_name".into(), "paste.gg".into());
  Template::render("index", map)
}
