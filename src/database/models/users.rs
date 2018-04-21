use super::super::DbConn;
use super::super::schema::users;
use errors::*;

use diesel;
use diesel::prelude::*;

use sodiumoxide::crypto::pwhash::{HashedPassword, pwhash_verify};

use uuid::Uuid;

#[derive(Debug, Serialize, AsChangeset, Identifiable, Queryable)]
pub struct User {
  id: Uuid,
  username: String,
  #[serde(skip_serializing)]
  password: String,
  name: String,
  email: String,
}

impl User {
  pub fn id(&self) -> Uuid {
    self.id
  }

  pub fn username(&self) -> &str {
    &self.username
  }

  pub fn set_username(&mut self, username: String) {
    self.username = username;
  }

  pub fn password(&self) -> &str {
    &self.password
  }

  pub fn set_password(&mut self, password: String) {
    self.password = password;
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn set_name(&mut self, name: String) {
    self.name = name;
  }

  pub fn email(&self) -> &str {
    &self.email
  }

  pub fn set_email(&mut self, email: String) {
    self.email = email;
  }

  pub fn check_password(&self, pass: &str) -> bool {
    let mut stored_bytes = self.password.clone().into_bytes();
    stored_bytes.push(0);
    let hash = HashedPassword::from_slice(&stored_bytes).expect("hashed password");
    pwhash_verify(&hash, pass.as_bytes())
  }

  pub fn update(&self, conn: &DbConn) -> Result<()> {
    diesel::update(users::table)
      .filter(users::id.eq(self.id))
      .set(self)
      .execute(&**conn)?;

    Ok(())
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
