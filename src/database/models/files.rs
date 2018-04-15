use super::super::schema::files;
use super::pastes::Paste;

use uuid::Uuid;

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(Paste)]
pub struct File {
  id: Uuid,
  paste_id: Uuid,
  name: String,
  is_binary: Option<bool>,
}

impl File {
  pub fn id(&self) -> Uuid {
    self.id
  }

  pub fn paste_id(&self) -> Uuid {
    self.paste_id
  }

  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn is_binary(&self) -> &Option<bool> {
    &self.is_binary
  }
}

#[derive(Insertable)]
#[table_name = "files"]
pub struct NewFile {
  id: Uuid,
  paste_id: Uuid,
  name: String,
  is_binary: Option<bool>,
}

impl NewFile {
  pub fn new(id: Uuid, paste_id: Uuid, name: String, is_binary: Option<bool>) -> Self {
    NewFile { id, paste_id, name, is_binary }
  }
}
