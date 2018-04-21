use super::super::schema::users;

use uuid::Uuid;

#[derive(Debug, Serialize, Identifiable, Queryable)]
pub struct User {
  id: Uuid,
  username: String,
  #[serde(skip_serializing)]
  password: String,
  name: Option<String>,
  email: Option<String>,
}

impl User {
  pub fn id(&self) -> Uuid {
    self.id
  }

  pub fn username(&self) -> &String {
    &self.username
  }

  pub fn password(&self) -> &String {
    &self.password
  }
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
  id: Uuid,
  username: String,
  password: String,
  name: Option<String>,
  email: Option<String>,
}

impl NewUser {
  pub fn new(
    id: Uuid,
    username: String,
    password: String,
    name: Option<String>,
    email: Option<String>,
  ) -> Self {
    NewUser { id, username, password, name, email }
  }
}
