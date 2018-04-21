use super::super::schema::api_keys;
use super::users::User;

use uuid::Uuid;

#[derive(Identifiable, Queryable, Associations)]
#[primary_key(key)]
#[belongs_to(User)]
pub struct ApiKey {
  key: Uuid,
  user_id: Uuid,
}

#[derive(Insertable)]
#[table_name = "api_keys"]
pub struct NewApiKey {
  key: Uuid,
  user_id: Uuid,
}

impl NewApiKey {
  pub fn new(key: Uuid, user_id: Uuid) -> Self {
    NewApiKey { key, user_id }
  }
}
