pub mod create;
pub mod get;
pub mod files;

#[patch("/<_id>")]
fn edit(_id: String) -> &'static str {
  "test patch"
}

#[delete("/<_id>")]
fn delete(_id: String) -> &'static str {
  "test delete"
}
