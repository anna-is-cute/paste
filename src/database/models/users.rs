use super::super::schema::users;

use uuid::Uuid;

#[derive(Identifiable, Queryable)]
pub struct User {
  id: Uuid,
  username: String,
  password: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
  id: Uuid,
  username: String,
  password: String,
}
