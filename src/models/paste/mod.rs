use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::Queryable;
use diesel::serialize::{self, ToSql};
use diesel::sql_types::SmallInt;

use rocket::http::RawStr;
use rocket::request::FromFormValue;

use serde::de::{self, Deserialize, Deserializer};

use unicode_segmentation::UnicodeSegmentation;

use std::io::Write;
use std::ops::Deref;

pub mod output;
pub mod update;

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
  pub description: Option<Description>,
  #[serde(default)]
  pub visibility: Visibility,
}

#[derive(Debug, Clone, Serialize)]
pub struct Description(String);

impl Description {
  pub fn new(s: String) -> Self {
    Description(s)
  }

  pub fn into_inner(self) -> String {
    self.0
  }
}

impl<'de> Deserialize<'de> for Description {
  fn deserialize<D>(des: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
  {
    let string: String = String::deserialize(des)?;

    let bytes = string.len();
    if bytes > 25 * 1024 {
      return Err(de::Error::invalid_length(bytes, &"<= 25KiB"));
    }

    let graphemes = string.graphemes(true).count();
    if graphemes > 255 {
      return Err(de::Error::invalid_length(graphemes, &"<= 255 extended grapheme clusters"));
    }

    Ok(Description(string))
  }
}

impl Deref for Description {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<S> From<S> for Description
  where S: AsRef<str>,
{
  fn from(s: S) -> Self {
    Description(s.as_ref().into())
  }
}

/// Visibility of a [`Paste`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, AsExpression)]
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

impl<DB: Backend<RawValue = [u8]>> Queryable<SmallInt, DB> for Visibility {
  type Row = i16;

  fn build(row: Self::Row) -> Self {
    match row {
      0 => Visibility::Public,
      1 => Visibility::Unlisted,
      2 => Visibility::Private,
      _ => panic!("invalid visibility in database")
    }
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

impl<'v> FromFormValue<'v> for Visibility {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
      let vis = match form_value.as_str() {
        "public" => Visibility::Public,
        "unlisted" => Visibility::Unlisted,
        "private" => Visibility::Private,
        _ => return Err(form_value),
      };

      Ok(vis)
    }

    fn default() -> Option<Self> {
      Some(Default::default())
    }
}

/// A file in a [`Paste`].
#[derive(Debug, Serialize, Deserialize)]
pub struct PasteFile {
  pub name: Option<String>,
  pub content: Content,
}

/// The content of a [`PasteFile`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "format", content = "value")]
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

impl Content {
  pub fn into_bytes(self) -> Vec<u8> {
    match self {
      Content::Text(s) => s.into_bytes(),
      Content::Base64(b) | Content::Gzip(b) | Content::Xz(b) => b,
    }
  }

  pub fn is_binary(&self) -> bool {
    // TODO: allow this to be specified in the paste?
    match *self {
      Content::Base64(_) | Content::Gzip(_) | Content::Xz(_) => true,
      _ => false,
    }
  }
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

  use serde::de::{self, Deserializer};
  use serde::ser::{self, Serializer};

  use std::io::{Read, Write};

  pub fn serialize<T, S>(data: &T, ser: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
          T: AsRef<[u8]> + ?Sized,
  {
    let mut encoder = Encoder::new(Vec::new()).map_err(ser::Error::custom)?;
    encoder.write_all(data.as_ref()).map_err(ser::Error::custom)?;
    let encoded_bytes = encoder.finish().into_result().map_err(ser::Error::custom)?;
    ser.serialize_str(&base64::encode(&encoded_bytes))
  }

  pub fn deserialize<'de, D>(des: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de>,
  {
    let bytes = des.deserialize_string(Base64Visitor)?;
    let mut decoder = Decoder::new(bytes.as_slice()).map_err(de::Error::custom)?;
    let mut decoded_bytes = Vec::new();
    decoder.read_to_end(&mut decoded_bytes).map_err(de::Error::custom)?;
    Ok(decoded_bytes)
  }
}

mod xz_base64_serde {
  use super::base64_serde::Base64Visitor;

  use base64;

  use xz2::read::XzDecoder;
  use xz2::write::XzEncoder;

  use serde::de::{self, Deserializer};
  use serde::ser::{self, Serializer};

  use std::io::{Read, Write};

  pub fn serialize<T, S>(data: &T, ser: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
          T: AsRef<[u8]> + ?Sized,
  {
    let mut encoder = XzEncoder::new(Vec::new(), 9);
    encoder.write_all(data.as_ref()).map_err(ser::Error::custom)?;
    let encoded_bytes = encoder.finish().map_err(ser::Error::custom)?;
    ser.serialize_str(&base64::encode(&encoded_bytes))
  }

  pub fn deserialize<'de, D>(des: D) -> Result<Vec<u8>, D::Error>
    where D: Deserializer<'de>,
  {
    let bytes = des.deserialize_string(Base64Visitor)?;
    let mut decoder = XzDecoder::new(bytes.as_slice());
    let mut decoded_bytes = Vec::new();
    decoder.read_to_end(&mut decoded_bytes).map_err(de::Error::custom)?;
    Ok(decoded_bytes)
  }
}
