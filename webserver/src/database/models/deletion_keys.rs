use crate::models::id::PasteId;
use super::pastes::Paste;
use super::super::schema::deletion_keys;

use sodiumoxide::crypto::pwhash::{
  self,
  HashedPassword,
};

use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct SecretDeletionKey(pub Uuid);

impl SecretDeletionKey {
  pub fn uuid(&self) -> Uuid {
    self.0
  }

  pub fn to_hashed_password(&self) -> HashedPassword {
    // get the simple string
    let key_str = self.0.to_simple().to_string();
    // hash the simple string
    pwhash::pwhash(
      key_str.as_bytes(),
      pwhash::OPSLIMIT_INTERACTIVE,
      pwhash::MEMLIMIT_INTERACTIVE,
    ).expect("pwhash failed")
  }

  pub fn to_hashed_string(&self) -> String {
    // get the hash
    let hashed = self.to_hashed_password();
    // hash is all ascii and nul-terminated, so strip off the last byte and interpret as string
    let hashed_str = unsafe {
      std::str::from_utf8_unchecked(&hashed[..pwhash::HASHEDPASSWORDBYTES - 1])
    };
    // clone the bytes
    hashed_str.to_string()
  }
}

#[derive(Debug, Identifiable, Queryable, Associations)]
#[primary_key(key)]
#[belongs_to(Paste)]
pub struct DeletionKey {
  key: String,
  paste_id: PasteId,
}

impl DeletionKey {
  pub fn check_key(&self, input: &str) -> bool {
    match self.hashed_password() {
      Some(hashed) => pwhash::pwhash_verify(&hashed, input.as_bytes()),
      None => self.key == input,
    }
  }

  pub fn hashed_password(&self) -> Option<HashedPassword> {
    let key_bytes = self.key.as_bytes();
    if key_bytes.len() != pwhash::HASHEDPASSWORDBYTES - 1 {
      return None;
    }
    let mut bytes = key_bytes.to_vec();
    bytes.push(0x00);
    HashedPassword::from_slice(&bytes)
  }

  pub fn paste_id(&self) -> PasteId {
    self.paste_id
  }
}

#[derive(Insertable)]
#[table_name = "deletion_keys"]
pub struct NewDeletionKey {
  key: String,
  paste_id: PasteId,
}

impl NewDeletionKey {
  pub fn new(key: String, paste_id: PasteId) -> Self {
    NewDeletionKey { key, paste_id }
  }

  pub fn generate(paste_id: PasteId) -> (Self, SecretDeletionKey) {
    // generate the uuid
    let key = SecretDeletionKey(Uuid::new_v4());
    // get the hashed version
    let hashed_str = key.to_hashed_string();
    // create the new deletion key
    let ndk = NewDeletionKey::new(
      hashed_str,
      paste_id,
    );
    // return the uuid and the ndk
    (ndk, key)
  }
}
