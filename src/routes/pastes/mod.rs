pub mod create;
pub mod get;
pub mod files;

#[patch("/<id>")]
fn edit(id: String) -> &'static str {
  "test patch"
}

#[delete("/<id>")]
fn delete(id: String) -> &'static str {
  "test delete"
}
