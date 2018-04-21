use errors::*;
use super::super::DbConn;
use super::super::models::api_keys::{ApiKey, NewApiKey};
use super::super::schema::{users, api_keys};

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

  pub fn keys(&self, conn: &DbConn) -> Result<Vec<ApiKey>> {
    let keys = api_keys::table
      .filter(api_keys::user_id.eq(self.id))
      .load(&**conn)?;

    Ok(keys)
  }

  pub fn create_key(&self, conn: &DbConn, name: String) -> Result<ApiKey> {
    let new_key = NewApiKey::new(name, Uuid::new_v4(), self.id);
    let key = diesel::insert_into(api_keys::table)
      .values(&new_key)
      .get_result(&**conn)?;
    Ok(key)
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
