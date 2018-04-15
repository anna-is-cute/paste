use super::super::schema::deletion_keys;
use super::pastes::Paste;

use uuid::Uuid;

#[derive(Identifiable, Queryable, Associations)]
#[primary_key(key)]
#[belongs_to(Paste)]
pub struct DeletionKey {
  key: Uuid,
  paste_id: Uuid,
}

#[derive(Insertable)]
#[table_name = "deletion_keys"]
pub struct NewDeletionKey {
  key: Uuid,
  paste_id: Uuid,
}

impl NewDeletionKey {
  pub fn new(key: Uuid, paste_id: Uuid) -> Self {
    NewDeletionKey { key, paste_id }
  }

  pub fn generate(paste_id: Uuid) -> Self {
    NewDeletionKey::new(Uuid::new_v4(), paste_id)
  }

  pub fn key(&self) -> Uuid {
    self.key
  }
}
