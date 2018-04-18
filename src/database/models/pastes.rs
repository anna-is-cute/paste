use database::DbConn;
use errors::*;
use models::paste::update::MetadataUpdate;
use models::paste::Visibility;
use models::status::ErrorKind;
use super::super::schema::pastes;
use super::users::User;

use chrono::{NaiveDateTime, Utc};

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
  description: Option<String>,
  created_at: NaiveDateTime,
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

  pub fn description(&self) -> &Option<String> {
    &self.description
  }

  pub fn set_description<S: AsRef<str>>(&mut self, description: Option<S>) {
    self.description = description.map(|x| x.as_ref().to_string().into());
  }

  pub fn created_at(&self) -> &NaiveDateTime {
    &self.created_at
  }

  pub fn update(&mut self, conn: &DbConn, update: &MetadataUpdate) -> Result<()> {
    let changed = update.name.is_some()
      || update.visibility.is_some()
      || update.description.is_some();
    if !changed {
      return Ok(());
    }
    if let Some(ref update) = update.name {
      self.set_name(update.clone());
    }
    if let Some(ref update) = update.visibility {
      self.set_visibility(*update);
    }
    if let Some(ref update) = update.description {
      self.set_description(update.clone().map(|x| x.into_inner()));
    }
    diesel::update(pastes::table)
      .filter(pastes::id.eq(self.id))
      .set(&*self)
      .execute(&**conn)?;

    Ok(())
  }

  pub fn check_access<U>(&self, user: U) -> Option<(HttpStatus, ErrorKind)>
    where U: Into<Option<Uuid>>,
  {
    let user = user.into();
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
  description: Option<String>,
  created_at: NaiveDateTime,
}

impl NewPaste {
  pub fn new(
    id: Uuid,
    name: Option<String>,
    description: Option<String>,
    visibility: Visibility,
    author_id: Option<Uuid>,
    created_at: Option<NaiveDateTime>,
  ) -> Self {
    let created_at = created_at.unwrap_or_else(|| Utc::now().naive_utc());
    NewPaste { id, name, visibility, author_id, description, created_at }
  }
}
