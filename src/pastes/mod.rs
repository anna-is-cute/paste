pub mod create;

#[get("/<id>")]
fn get(id: String) -> String {
  format!("getting paste {}", id)
}

#[patch("/<id>")]
fn edit(id: String) -> &'static str {
  "test patch"
}

#[delete("/<id>")]
fn delete(id: String) -> &'static str {
  "test delete"
}
