use base64;

use rocket::request::FromParam;
use rocket::http::RawStr;

use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};

use uuid::Uuid;

use std::str::FromStr;
use std::ops::Deref;
use std::fmt;

/// An ID for a paste, which may or may not exist.
///
/// Mostly useful for having Rocket accept only valid IDs in routes.
#[derive(Debug)]
pub struct PasteId(Uuid);

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
#[derive(Debug, Deserialize)]
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
