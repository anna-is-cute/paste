#![cfg_attr(feature = "cargo-clippy", allow(option_option))]

use models::paste::{Content, CountedText, Visibility};

use serde::de::{Deserialize, Deserializer};

use uuid::Uuid;

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
  // FIXME: use CountedText
  #[serde(default, deserialize_with = "double_option")]
  pub name: Option<Option<String>>,
  // double option because description can be removed, changed, or left alone
  #[serde(default, deserialize_with = "double_option")]
  pub description: Option<Option<CountedText>>,
  // single option because visibility can only be changed or left alone (all pastes must have
  // visibility)
  #[serde(default)]
  pub visibility: Option<Visibility>,
}

#[derive(Debug, Deserialize)]
pub struct PasteFileUpdate {
  // single option because id can be specified to mean "update this file" or omitted to mean "add
  // this file"
  #[serde(default)]
  pub id: Option<Uuid>,
  // single option because name can only be changed or left alone (all pastes must have name)
  #[serde(default)]
  pub name: Option<String>,
  // double option because content can be removed (file deletion), changed, or left alone
  #[serde(default, deserialize_with = "double_option")]
  pub content: Option<Option<Content>>,
}

fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
  where T: Deserialize<'de>,
        D: Deserializer<'de>
{
  Deserialize::deserialize(de).map(Some)
}
