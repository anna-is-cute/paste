use super::super::schema::files;
use super::pastes::Paste;

use uuid::Uuid;

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Paste)]
pub struct File {
  id: Uuid,
  paste_id: Uuid,
  name: Option<String>,
  is_binary: Option<bool>,
}

#[derive(Insertable)]
#[table_name = "files"]
pub struct NewFile {
  id: Uuid,
  paste_id: Uuid,
  name: Option<String>,
  is_binary: Option<bool>,
}
