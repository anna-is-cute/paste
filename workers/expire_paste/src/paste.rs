use crate::schema::pastes;

use chrono::NaiveDateTime;

use uuid::Uuid;

#[derive(Debug, Identifiable, AsChangeset, Queryable, Associations)]
#[changeset_options(treat_none_as_null = "true")]
pub struct Paste {
  pub id: Uuid,
  pub name: Option<String>,
  pub visibility: i16,
  pub author_id: Option<Uuid>,
  pub description: Option<String>,
  pub created_at: NaiveDateTime,
  pub expires: Option<NaiveDateTime>,
  pub updated_at: Option<NaiveDateTime>,
}
