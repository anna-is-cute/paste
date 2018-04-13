use models::paste::PasteId;

use rocket_contrib::UUID;

#[get("/<paste_id>/files/<file_id>")]
fn get_file_id(paste_id: PasteId, file_id: UUID) {
  println!("get file {} from paste {}", file_id, paste_id.to_string());
}
