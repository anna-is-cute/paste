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
