use rocket_contrib::Template;

#[get("/")]
fn get() -> Template {
  Template::render("index", json!({}))
}
