use database::DbConn;
use errors::*;
use models::paste::update::PasteUpdate;
use models::paste::Visibility;
use models::status::ErrorKind;
use super::super::schema::pastes;
use super::users::User;

use diesel;
use diesel::prelude::*;

use rocket::http::Status as HttpStatus;

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

  pub fn set_name<S: AsRef<str>>(&mut self, name: Option<S>) {
    self.name = name.map(|x| x.as_ref().to_string());
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

  pub fn update(&mut self, conn: &DbConn, update: &PasteUpdate) -> Result<()> {
    let changed = update.metadata.name.is_some() || update.metadata.visibility.is_some();
    if !changed {
      return Ok(());
    }
    if let Some(ref update) = update.metadata.name {
      self.set_name(update.clone());
    }
    if let Some(ref update) = update.metadata.visibility {
      self.set_visibility(*update);
    }
    diesel::update(pastes::table).set(&*self).execute(&**conn)?;

    Ok(())
  }

  pub fn check_access(&self, user: Option<Uuid>) -> Option<(HttpStatus, ErrorKind)> {
    let is_private = self.visibility == Visibility::Private;
    if self.author_id.is_none() || !is_private || self.author_id == user {
      return None;
    }
    if is_private {
      Some((HttpStatus::NotFound, ErrorKind::MissingPaste))
    } else {
      Some((HttpStatus::Forbidden, ErrorKind::NotAllowed))
    }
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
