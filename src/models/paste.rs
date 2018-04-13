use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};

use std::fmt;

#[derive(Debug, Deserialize)]
pub struct Paste {
  #[serde(flatten)]
  pub metadata: Metadata,
  pub files: Vec<PasteFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
  pub name: Option<String>,
  pub visibility: Option<Visibility>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
  Public,
  Unlisted,
  Private,
}

#[derive(Debug)]
pub struct PasteFile {
  pub name: Option<String>,
  pub content: Content,
}

#[derive(Debug)]
pub enum Content {
  Text(String),
  Base64(String),
  Gzip(String),
  Xz(String),
}

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
              content = Some(Content::Base64(map.next_value()?));
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
