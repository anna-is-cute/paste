use crate::{
  errors::*,
  models::{
    id::{ApiKeyId, UserId},
    user::AvatarProvider,
  },
};

use super::api_keys::{ApiKey, NewApiKey};
use super::email_verifications::{EmailVerification, NewEmailVerification};
use super::super::DbConn;
use super::super::schema::{users, api_keys, email_verifications};

use chrono::NaiveDateTime;

use diesel::prelude::*;

use sodiumoxide::crypto::pwhash::{HashedPassword, pwhash_verify};

use uuid::Uuid;

#[derive(Debug, Serialize, AsChangeset, Identifiable, Queryable)]
pub struct User {
  id: UserId,
  username: String,
  #[serde(skip_serializing)]
  password: String,
  name: String,
  email: String,
  email_verified: bool,
  #[serde(skip_serializing)]
  shared_secret: Option<Vec<u8>>,
  #[serde(skip_serializing)]
  tfa_enabled: bool,
  avatar_provider: AvatarProvider,
}

impl User {
  pub fn id(&self) -> UserId {
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

  pub fn set_hashed_password(&mut self, password: String) {
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

  pub fn email_verified(&self) -> bool {
    self.email_verified
  }

  pub fn set_email_verified(&mut self, verified: bool) {
    self.email_verified = verified;
  }

  pub fn shared_secret(&self) -> Option<&[u8]> {
    self.shared_secret.as_ref().map(|x| x.as_slice())
  }

  pub fn set_shared_secret(&mut self, secret: Option<Vec<u8>>) {
    self.shared_secret = secret;
  }

  pub fn tfa_enabled(&self) -> bool {
    self.tfa_enabled
  }

  pub fn set_tfa_enabled(&mut self, enabled: bool) {
    self.tfa_enabled = enabled;
  }

  pub fn avatar_provider(&self) -> AvatarProvider {
    self.avatar_provider
  }

  pub fn set_avatar_provider(&mut self, avatar_provider: AvatarProvider) {
    self.avatar_provider = avatar_provider;
  }

  pub fn create_email_verification(&self, conn: &DbConn, last_sent: Option<NaiveDateTime>) -> Result<(EmailVerification, Vec<u8>)> {
    let (nv, secret) = NewEmailVerification::new(
      self.email(),
      self.id(),
      last_sent,
    );
    let ver = diesel::insert_into(email_verifications::table)
      .values(&nv)
      .get_result(&**conn)?;
    Ok((ver, secret))
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
    let new_key = NewApiKey::new(name, ApiKeyId(Uuid::new_v4()), self.id);
    let key = diesel::insert_into(api_keys::table)
      .values(&new_key)
      .get_result(&**conn)?;
    Ok(key)
  }

  pub fn delete_key(&self, conn: &DbConn, key: ApiKeyId) -> Result<()> {
    diesel::delete(api_keys::table)
      .filter(api_keys::key.eq(key))
      .execute(&**conn)?;
    Ok(())
  }

  pub fn delete(&self, conn: &DbConn) -> Result<()> {
    diesel::delete(users::table)
      .filter(users::id.eq(self.id))
      .execute(&**conn)?;
    Ok(())
  }
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
  id: UserId,
  username: String,
  password: String,
  name: Option<String>,
  email: Option<String>,
  email_verified: bool,
  shared_secret: Option<Vec<u8>>,
  tfa_enabled: bool,
  avatar_provider: AvatarProvider,
}

impl NewUser {
  pub fn new(
    id: UserId,
    username: String,
    password: String,
    name: Option<String>,
    email: Option<String>,
  ) -> Self {
    NewUser {
      id,
      username,
      password,
      name,
      email,
      email_verified: false,
      shared_secret: None,
      tfa_enabled: false,
      avatar_provider: AvatarProvider::Gravatar,
    }
  }
}
