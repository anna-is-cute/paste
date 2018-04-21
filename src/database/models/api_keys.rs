use super::super::schema::api_keys;
use super::users::User;

use uuid::Uuid;

#[derive(Debug, Serialize, Identifiable, Queryable, Associations)]
#[primary_key(key)]
#[belongs_to(User)]
pub struct ApiKey {
  key: Uuid,
  user_id: Uuid,
  name: String,
}

#[derive(Insertable)]
#[table_name = "api_keys"]
pub struct NewApiKey {
  key: Uuid,
  user_id: Uuid,
  name: String,
}

impl NewApiKey {
  pub fn new(name: String, key: Uuid, user_id: Uuid) -> Self {
    NewApiKey { name, key, user_id }
  }
}
