use models::paste::Visibility;
use super::super::schema::pastes;
use super::users::User;

use uuid::Uuid;

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(User, foreign_key = "author_id")]
pub struct Paste {
  id: Uuid,
  name: Option<String>,
  visibility: Visibility,
  author_id: Option<Uuid>,
}

#[derive(Insertable)]
#[table_name = "pastes"]
pub struct NewPaste {
  id: Uuid,
  name: Option<String>,
  visibility: Visibility,
  author_id: Option<Uuid>,
}

impl NewPaste {
  pub fn new(id: Uuid, name: Option<String>, visibility: Visibility, author_id: Option<Uuid>) -> Self {
    NewPaste { id, name, visibility, author_id }
  }
}
