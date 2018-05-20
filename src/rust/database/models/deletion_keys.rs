use models::id::{DeletionKeyId, PasteId};
use super::pastes::Paste;
use super::super::schema::deletion_keys;

use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, Associations)]
#[primary_key(key)]
#[belongs_to(Paste)]
pub struct DeletionKey {
  key: DeletionKeyId,
  paste_id: PasteId,
}

impl DeletionKey {
  pub fn key(&self) -> DeletionKeyId {
    self.key
  }

  pub fn paste_id(&self) -> PasteId {
    self.paste_id
  }
}

#[derive(Insertable)]
#[table_name = "deletion_keys"]
pub struct NewDeletionKey {
  key: DeletionKeyId,
  paste_id: PasteId,
}

impl NewDeletionKey {
  pub fn new(key: DeletionKeyId, paste_id: PasteId) -> Self {
    NewDeletionKey { key, paste_id }
  }

  pub fn generate(paste_id: PasteId) -> Self {
    NewDeletionKey::new(
      DeletionKeyId(Uuid::new_v4()),
      paste_id,
    )
  }

  pub fn key(&self) -> DeletionKeyId {
    self.key
  }
}
