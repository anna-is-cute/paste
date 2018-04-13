use models::paste::PasteId;

#[get("/<id>")]
fn get(id: PasteId) -> String {
  format!("getting paste {}", *id)
}
