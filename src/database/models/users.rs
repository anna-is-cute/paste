use super::super::schema::users;

use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
pub struct User {
  id: Uuid,
  username: String,
  password: String,
}

impl User {
  pub fn id(&self) -> Uuid {
    self.id
  }
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
  id: Uuid,
  username: String,
  password: String,
}
