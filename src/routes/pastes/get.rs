#[get("/<id>")]
fn get(id: String) -> String {
  format!("getting paste {}", id)
}
