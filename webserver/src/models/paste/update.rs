#![cfg_attr(feature = "cargo-clippy", allow(option_option))]

use models::id::FileId;
use models::paste::{Content, CountedText, Visibility};
use utils::Language;

use chrono::{DateTime, Utc};

use serde::de::{Deserialize, Deserializer};

use std::fmt::{self, Debug, Formatter};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Update<V> {
  Ignore,
  Set(V),
  Remove,
}

impl<V> Update<V> {
  pub fn is_ignore(&self) -> bool {
    match *self {
      Update::Ignore => true,
      _ => false,
    }
  }

  pub fn is_remove(&self) -> bool {
    match *self {
      Update::Remove => true,
      _ => false,
    }
  }

  pub fn is_set(&self) -> bool {
    match *self {
      Update::Set(_) => true,
      _ => false,
    }
  }

  pub fn unwrap_set(self) -> V {
    match self {
      Update::Ignore => panic!("unwrap_set on Ignore"),
      Update::Set(v) => v,
      Update::Remove => panic!("unwrap_set on Remove"),
    }
  }

  pub fn set(self) -> Option<V> {
    match self {
      Update::Set(v) => Some(v),
      _ => None,
    }
  }
}

impl<V> Default for Update<V> {
  fn default() -> Self {
    Update::Ignore
  }
}

impl<V> Debug for Update<V>
  where V: Debug,
{
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match *self {
      Update::Ignore => f.write_str("Ignore"),
      Update::Remove => f.write_str("Remove"),
      Update::Set(ref c) => {
        f.write_str("Set(")?;
        Debug::fmt(c, f)?;
        f.write_str(")")
      }
    }
  }
}

impl<'de, V> Deserialize<'de> for Update<V>
  where V: Deserialize<'de>,
{
  fn deserialize<D>(de: D) -> Result<Update<V>, D::Error>
    where D: Deserializer<'de>,
  {
    let up = match Option::deserialize(de).map(Some)? {
      None => Update::Ignore,
      Some(None) => Update::Remove,
      Some(Some(v)) => Update::Set(v),
    };

    Ok(up)
  }
}

#[derive(Debug, Deserialize)]
pub struct PasteUpdate {
  #[serde(flatten)]
  pub metadata: MetadataUpdate,
  // single option because files can only be changed or left alone (all pastes must have files)
  #[serde(default)]
  pub files: Option<Vec<PasteFileUpdate>>,
}

#[derive(Debug, Deserialize)]
pub struct MetadataUpdate {
  // double option because name can be removed, changed, or left alone
  #[serde(default)]
  pub name: Update<CountedText>,
  // double option because description can be removed, changed, or left alone
  #[serde(default)]
  pub description: Update<CountedText>,
  // single option because visibility can only be changed or left alone (all pastes must have
  // visibility)
  #[serde(default)]
  pub visibility: Option<Visibility>,
  // double option because expires can be removed, changed, or left alone
  #[serde(default)]
  pub expires: Update<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct PasteFileUpdate {
  // single option because id can be specified to mean "update this file" or omitted to mean "add
  // this file"
  #[serde(default)]
  pub id: Option<FileId>,
  // single option because name can only be changed or left alone (all pastes must have name)
  #[serde(default)]
  pub name: Option<String>,
  // double option because highlight language can be removed, changed, or left alone
  #[serde(default)]
  pub highlight_language: Update<Language>,
  // double option because content can be removed (file deletion), changed, or left alone
  #[serde(default)]
  pub content: Update<Content>,
}
