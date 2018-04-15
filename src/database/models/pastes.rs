use models::paste::Visibility;
use super::super::schema::pastes;
use super::users::User;

use uuid::Uuid;

#[derive(Identifiable, AsChangeset, Queryable, Associations)]
#[changeset_options(treat_none_as_null = "true")]
#[belongs_to(User, foreign_key = "author_id")]
pub struct Paste {
  id: Uuid,
  name: Option<String>,
  visibility: Visibility,
  author_id: Option<Uuid>,
}

impl Paste {
  pub fn id(&self) -> Uuid {
    self.id
  }

  pub fn name(&self) -> &Option<String> {
    &self.name
  }

  pub fn set_name(&mut self, name: Option<String>) {
    self.name = name;
  }

  pub fn visibility(&self) -> Visibility {
    self.visibility
  }

  pub fn set_visibility(&mut self, visibility: Visibility) {
    self.visibility = visibility;
  }

  pub fn author_id(&self) -> &Option<Uuid> {
    &self.author_id
  }
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
