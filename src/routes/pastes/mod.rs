pub mod create;
pub mod get;
pub mod files;
pub mod delete;

#[patch("/<_id>")]
fn edit(_id: String) -> &'static str {
  "test patch"
}
