use models::paste::{Content, Visibility};

use serde::de::{Deserialize, Deserializer};

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
  #[serde(default, deserialize_with = "double_option")]
  pub name: Option<Option<String>>,
  // single option because visibility can only be changed or left alone (all pastes must have
  // visibility)
  #[serde(default)]
  pub visibility: Option<Visibility>,
}

#[derive(Debug, Deserialize)]
pub struct PasteFileUpdate {
  // double option because name can be removed, changed, or left alone
  #[serde(default, deserialize_with = "double_option")]
  pub name: Option<Option<String>>,
  // single option because content can only be changed or left alone (all pastes must have content)
  #[serde(default)]
  pub content: Option<Content>,
}

fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
  where T: Deserialize<'de>,
        D: Deserializer<'de>
{
  Deserialize::deserialize(de).map(Some)
}
