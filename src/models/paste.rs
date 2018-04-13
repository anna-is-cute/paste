use errors::Result as PasteResult;
use store::Store;

use base64;

use rocket::http::RawStr;
use rocket::request::FromParam;

use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, Serializer, SerializeStruct};

use serde_json;

use uuid::Uuid;

use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::str::FromStr;

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
#[derive(Debug)]
pub struct PasteFile {
  pub name: Option<String>,
  pub content: Content,
}

/// The content of a [`PasteFile`].
#[derive(Debug)]
pub enum Content {
  /// Valid UTF-8 text
  Text(String),
  /// Base64-encoded data
  Base64(Vec<u8>),
  /// Base64-encoded gzip data
  Gzip(String),
  /// Base64-encoded xz data
  Xz(String),
}

impl Serialize for PasteFile {
  fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
  {
    let mut s = ser.serialize_struct("PasteFile", 2)?;

    s.serialize_field("name", &self.name)?;
    match self.content {
      Content::Text(ref text) => s.serialize_field("text", text)?,
      Content::Base64(ref bytes) => s.serialize_field("base64", &base64::encode(bytes))?,
      _ => unreachable!(),
    }
    s.end()
  }
}

// Allow PasteFile to be deserialized.
//
// This has to be done due to the way that the content field is deserialized.
impl<'de> Deserialize<'de> for PasteFile {
  fn deserialize<D>(des: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
  {
    enum Field {
      Name,
      Text,
      Base64,
      Gzip,
      Xz,
    }

    impl<'de> Deserialize<'de> for Field {
      fn deserialize<D>(des: D) -> Result<Field, D::Error>
        where D: Deserializer<'de>,
      {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
          type Value = Field;

          fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("one of `name` or [`text`, `base64`, `gzip`, `xz`]")
          }

          fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: de::Error,
          {
            match value {
              "name" => Ok(Field::Name),
              "text" => Ok(Field::Text),
              "base64" => Ok(Field::Base64),
              "gzip" => Ok(Field::Gzip),
              "xz" => Ok(Field::Xz),
              _ => Err(de::Error::unknown_field(value, FIELDS)),
            }
          }
        }

        des.deserialize_identifier(FieldVisitor)
      }
    }

    struct PasteFileVisitor;

    impl <'de> Visitor<'de> for PasteFileVisitor {
      type Value = PasteFile;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct PasteFile")
      }

      fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
        where V: MapAccess<'de>,
      {
        let mut name = None;
        let mut content = None;
        while let Some(key) = map.next_key()? {
          match key {
            Field::Name => {
              if name.is_some() {
                return Err(de::Error::duplicate_field("name"));
              }
              name = Some(map.next_value()?);
            },
            Field::Text => {
              if content.is_some() {
                return Err(de::Error::duplicate_field("content"));
              }
              content = Some(Content::Text(map.next_value()?));
            },
            Field::Base64 => {
              if content.is_some() {
                return Err(de::Error::duplicate_field("content"));
              }
              let string: String = map.next_value()?;
              let decoded = match base64::decode(&string) {
                Ok(d) => d,
                Err(_) => return Err(de::Error::invalid_value(
                  de::Unexpected::Str(&string),
                  &"valid base64",
                )),
              };
              content = Some(Content::Base64(decoded));
            },
            Field::Gzip => {
              if content.is_some() {
                return Err(de::Error::duplicate_field("content"));
              }
              content = Some(Content::Gzip(map.next_value()?));
            },
            Field::Xz => {
              if content.is_some() {
                return Err(de::Error::duplicate_field("content"));
              }
              content = Some(Content::Xz(map.next_value()?));
            },
          }
        }
        let content = content.ok_or_else(|| de::Error::missing_field("content"))?;
        Ok(PasteFile {
          name,
          content,
        })
      }
    }

    const FIELDS: &[&str] = &["name", "text", "base64", "gzip", "xz"];
    des.deserialize_struct("PasteFile", FIELDS, PasteFileVisitor)
  }
}
