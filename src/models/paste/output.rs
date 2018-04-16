use super::{Paste, Content};
use utils::SimpleUuid;

use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Output {
  pub id: SimpleUuid,
  #[serde(flatten)]
  pub paste: Paste,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub deletion_key: Option<Uuid>,
  pub files: Vec<OutputFile>,
}

#[derive(Debug, Serialize)]
pub struct OutputFile {
  id: SimpleUuid,
  name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  content: Option<Content>,
}

impl OutputFile {
  pub fn new<S: Into<String>>(id: &Uuid, name: Option<S>, content: Option<Content>) -> Self {
    OutputFile {
      id: (*id).into(),
      name: name.map(Into::into),
      content,
    }
  }
}
