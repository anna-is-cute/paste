use errors::Result as PasteResult;
use store::Store;

use rocket::http::RawStr;
use rocket::request::FromParam;

use serde_json;

use uuid::Uuid;

use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::str::FromStr;

pub mod output;

/// An ID for a paste, which may or may not exist.
///
/// Mostly useful for having Rocket accept only valid IDs in routes.
#[derive(Debug)]
pub struct PasteId(pub Uuid);

impl PasteId {
  pub fn directory(&self) -> PathBuf {
    Store::directory().join(self.0.simple().to_string())
  }

  pub fn files_directory(&self) -> PathBuf {
    self.directory().join("files")
  }

  pub fn metadata(&self) -> PasteResult<Metadata> {
    let file = File::open(self.directory().join("metadata.json"))?;
    Ok(serde_json::from_reader(file)?)
  }

  pub fn internal(&self) -> PasteResult<Internal> {
    let file = File::open(self.directory().join("internal.json"))?;
    Ok(serde_json::from_reader(file)?)
  }
}

impl Display for PasteId {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0.simple())
  }
}

// Allow PasteId to be dereferenced into its inner type
impl Deref for PasteId {
  type Target = Uuid;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

// Allow Rocket to accept PasteId in routes
impl<'a> FromParam<'a> for PasteId {
  type Error = &'a RawStr;

  fn from_param(param: &'a RawStr) -> Result<Self, &'a RawStr> {
    match Uuid::from_str(param) {
      Ok(u) => Ok(PasteId(u)),
      Err(_) => Err(param)
    }
  }
}

/// A paste with files and metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct Paste {
  #[serde(flatten)]
  pub metadata: Metadata,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub files: Vec<PasteFile>,
}

/// Metadata describing a [`Paste`].
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
  pub name: Option<String>,
  pub visibility: Option<Visibility>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Internal {
  pub names: NameMapping,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NameMapping(Vec<(Uuid, String)>);

impl Deref for NameMapping {
  type Target = Vec<(Uuid, String)>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for NameMapping {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl NameMapping {
  pub fn get_name<'a, 'b>(&'a self, uuid: &'b Uuid) -> Option<&'a String> {
    self.0
      .iter()
      .find(|(u, _)| u == uuid)
      .map(|(_, n)| n)
  }

  pub fn get_uuid<'a, 'b>(&'a self, name: &'b str) -> Option<&'a Uuid> {
    self.0
      .iter()
      .find(|(_, n)| n == name)
      .map(|(u, _)| u)
  }
}

/// Visibility of a [`Paste`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
  /// Paste is visible to everyone and can be crawled.
  Public,
  /// Paste is visible to everyone who knows the link and cannot be crawled.
  Unlisted,
  /// Paste is visible only to the user who created it.
  ///
  /// Not available for anonymous pastes.
  Private,
}

/// A file in a [`Paste`].
#[derive(Debug, Serialize, Deserialize)]
pub struct PasteFile {
  pub name: Option<String>,
  #[serde(flatten)]
  pub content: Content,
}

/// The content of a [`PasteFile`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "format", content = "content")]
pub enum Content {
  /// Valid UTF-8 text
  Text(String),
  /// Base64-encoded data
  #[serde(with = "base64_serde")]
  Base64(Vec<u8>),
  /// Base64-encoded gzip data
  Gzip(String),
  /// Base64-encoded xz data
  Xz(String),
}

mod base64_serde {
  use base64;

  use serde::de::{self, Deserializer, Visitor};
  use serde::ser::Serializer;

  use std::fmt::{self, Formatter};

  pub fn serialize<T, S>(data: &T, ser: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
          T: AsRef<[u8]> + ?Sized,
  {
    ser.serialize_str(&base64::encode(data))
  }

  pub fn deserialize<'de, D>(des: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de>,
  {
    struct Base64Visitor;

    impl<'de> Visitor<'de> for Base64Visitor {
      type Value = Vec<u8>;

      fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a string")
      }

      fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: de::Error,
      {
        base64::decode(v)
          .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(v), &"valid base64"))
      }
    }

    des.deserialize_string(Base64Visitor)
  }
}
