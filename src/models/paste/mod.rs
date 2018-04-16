use errors::Result as PasteResult;
use database::DbConn;
use database::models::files::{NewFile, File as DbFile};
use database::schema::{pastes, files};
use store::Store;

use diesel;
use diesel::prelude::*;
use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::Queryable;
use diesel::serialize::{self, ToSql};
use diesel::sql_types::SmallInt;

use git2::{Repository, Signature, DiffOptions};

use rocket::http::RawStr;
use rocket::request::FromParam;

use uuid::Uuid;

use std::fmt::{self, Display, Formatter};
use std::fs::{self, File};
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;

pub mod output;
pub mod update;

/// An ID for a paste, which may or may not exist.
///
/// Mostly useful for having Rocket accept only valid IDs in routes.
#[derive(Debug, Clone, Copy)]
pub struct PasteId(pub Uuid);

impl PasteId {
  pub fn directory(&self) -> PathBuf {
    Store::directory().join(self.0.simple().to_string())
  }

  pub fn files_directory(&self) -> PathBuf {
    self.directory().join("files")
  }

  pub fn repo_dirty(&self) -> PasteResult<bool> {
    let repo = Repository::open(self.files_directory())?;
    let mut options = DiffOptions::new();
    options.ignore_submodules(true);
    let diff = repo.diff_index_to_workdir(None, Some(&mut options))?;
    Ok(diff.stats()?.files_changed() != 0)
  }

  pub fn commit_if_dirty(&self, username: &str, email: &str, message: &str) -> PasteResult<()> {
    if self.repo_dirty()? {
      return self.commit(username, email, message);
    }

    Ok(())
  }

  pub fn commit(&self, username: &str, email: &str, message: &str) -> PasteResult<()> {
    let repo = Repository::open(self.files_directory())?;

    let mut index = repo.index()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let head_id = repo.refname_to_id("HEAD")?;
    let parent = repo.find_commit(head_id)?;

    let signature = Signature::now(username, email)?;

    repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[&parent])?;

    Ok(())
  }

  pub fn create_file<S: AsRef<str>>(&self, conn: &DbConn, paste: PasteId, name: Option<S>, content: Content) -> PasteResult<Uuid> {
    // generate file id
    let id = Uuid::new_v4();

    // check if content is binary for later
    let binary = content.is_binary();

    // create file on the system
    let file_path = self.files_directory().join(id.simple().to_string());
    let mut f = File::create(file_path)?;
    f.write_all(&content.into_bytes())?;

    let name = name
      .map(|s| s.as_ref().to_string()) // get a String
      .or_else(|| self.next_generic_name(conn).ok()) // try to get a generic name if no name specified
      .unwrap_or_else(|| id.simple().to_string()); // fall back to uuid if necessary

    // add file to the database
    let new_file = NewFile::new(id, *paste, name, Some(binary));
    diesel::insert_into(files::table).values(&new_file).execute(&**conn)?;

    Ok(id)
  }

  pub fn len(&self, conn: &DbConn) -> PasteResult<usize> {
    let size: i64 = files::table
      .filter(files::paste_id.eq(self.0))
      .select(diesel::dsl::count(files::id))
      .first(&**conn)?;

    Ok(size as usize)
  }

  pub fn is_empty(&self, conn: &DbConn) -> PasteResult<bool> {
    Ok(self.len(conn)? == 0)
  }

  pub fn next_generic_name(&self, conn: &DbConn) -> PasteResult<String> {
    Ok(format!("pastefile{}", self.len(conn)? + 1))
  }

  pub fn delete_file(&self, conn: &DbConn, id: Uuid) -> PasteResult<()> {
    diesel::delete(files::table.filter(files::id.eq(id))).execute(&**conn)?;
    fs::remove_dir_all(self.files_directory().join(id.simple().to_string()))?;

    if self.is_empty(conn)? {
      self.delete(conn)?;
    }

    Ok(())
  }

  pub fn delete(&self, conn: &DbConn) -> PasteResult<()> {
    // database will cascade and delete all files and deletion keys, as well
    diesel::delete(pastes::table.filter(pastes::id.eq(self.0))).execute(&**conn)?;
    // remove from system
    fs::remove_dir_all(self.directory())?;

    Ok(())
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

  use serde::de::{self, Deserializer};
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
