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

  pub fn exists(&self) -> bool {
    self.directory().exists()
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
  #[serde(default)]
  pub visibility: Visibility,
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

use diesel::backend::Backend;
use diesel::serialize::{self, ToSql};
use diesel::deserialize::{self, FromSql};
use diesel::sql_types::SmallInt;
use std::io::Write;

/// Visibility of a [`Paste`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsExpression)]
#[sql_type = "SmallInt"]
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

impl Default for Visibility {
  fn default() -> Self {
    Visibility::Unlisted
  }
}

impl<DB: Backend> ToSql<SmallInt, DB> for Visibility {
  fn to_sql<W: Write>(&self, out: &mut serialize::Output<W, DB>) -> serialize::Result {
    let visibility: i16 = match *self {
      Visibility::Public => 0,
      Visibility::Unlisted => 1,
      Visibility::Private => 2,
    };

    <i16 as ToSql<SmallInt, DB>>::to_sql(&visibility, out)
  }
}

impl<DB: Backend<RawValue = [u8]>> FromSql<SmallInt, DB> for Visibility {
  fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
    let visibility = match <i16 as FromSql<SmallInt, DB>>::from_sql(bytes)? {
      0 => Visibility::Public,
      1 => Visibility::Unlisted,
      2 => Visibility::Private,
      x => return Err(Box::new(format_err!("bad visibility enum: {}", x).compat())),
    };
    Ok(visibility)
  }
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
  #[serde(with = "gzip_base64_serde")]
  Gzip(Vec<u8>),
  /// Base64-encoded xz data
  #[serde(with = "xz_base64_serde")]
  Xz(Vec<u8>),
}

mod base64_serde {
  use base64;

  use serde::de::{self, Deserializer, Visitor};
  use serde::ser::Serializer;

  use std::fmt::{self, Formatter};

  pub struct Base64Visitor;

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

  pub fn serialize<T, S>(data: &T, ser: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
          T: AsRef<[u8]> + ?Sized,
  {
    ser.serialize_str(&base64::encode(data))
  }

  pub fn deserialize<'de, D>(des: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de>,
  {
    des.deserialize_string(Base64Visitor)
  }
}

mod gzip_base64_serde {
  use super::base64_serde::Base64Visitor;

  use base64;

  use libflate::gzip::{Encoder, Decoder};

  use serde::de::{self, Deserializer, Visitor};
  use serde::ser::{self, Serializer};

  use std::io::{Read, Write};

  pub fn serialize<T, S>(data: &T, ser: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
          T: AsRef<[u8]> + ?Sized,
  {
    let mut encoder = Encoder::new(Vec::new()).map_err(|e| ser::Error::custom(e))?;
    encoder.write_all(data.as_ref()).map_err(|e| ser::Error::custom(e))?;
    let encoded_bytes = encoder.finish().into_result().map_err(|e| ser::Error::custom(e))?;
    ser.serialize_str(&base64::encode(&encoded_bytes))
  }

  pub fn deserialize<'de, D>(des: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de>,
  {
    let bytes = des.deserialize_string(Base64Visitor)?;
    let mut decoder = Decoder::new(bytes.as_slice()).map_err(|e| de::Error::custom(e))?;
    let mut decoded_bytes = Vec::new();
    decoder.read_to_end(&mut decoded_bytes).map_err(|e| de::Error::custom(e))?;
    Ok(decoded_bytes)
  }
}

mod xz_base64_serde {
  use super::base64_serde::Base64Visitor;

  use base64;

  use xz2::read::XzDecoder;
  use xz2::write::XzEncoder;

  use serde::de::{self, Deserializer, Visitor};
  use serde::ser::{self, Serializer};

  use std::io::{Read, Write};

  pub fn serialize<T, S>(data: &T, ser: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
          T: AsRef<[u8]> + ?Sized,
  {
    let mut encoder = XzEncoder::new(Vec::new(), 9);
    encoder.write_all(data.as_ref()).map_err(|e| ser::Error::custom(e))?;
    let encoded_bytes = encoder.finish().map_err(|e| ser::Error::custom(e))?;
    ser.serialize_str(&base64::encode(&encoded_bytes))
  }

  pub fn deserialize<'de, D>(des: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de>,
  {
    let bytes = des.deserialize_string(Base64Visitor)?;
    let mut decoder = XzDecoder::new(bytes.as_slice());
    let mut decoded_bytes = Vec::new();
    decoder.read_to_end(&mut decoded_bytes).map_err(|e| de::Error::custom(e))?;
    Ok(decoded_bytes)
  }
}
