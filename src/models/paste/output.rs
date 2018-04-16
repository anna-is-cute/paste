use super::{Paste, Metadata, Visibility, Content};
use utils::SimpleUuid;

use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Output {
  pub id: SimpleUuid,
  #[serde(flatten)]
  pub paste: Paste,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub deletion_key: Option<SimpleUuid>,
  pub files: Vec<OutputFile>,
}

impl Output {
  pub fn new<N, F>(paste_id: Uuid, name: Option<N>, vis: Visibility, deletion_key: Option<Uuid>, files: F) -> Self
    where N: AsRef<str>,
          F: IntoIterator<Item = OutputFile>,
  {
    Output {
      id: paste_id.into(),
      paste: Paste {
        metadata: Metadata {
          name: name.map(|x| x.as_ref().to_string()),
          visibility: vis,
        },
        files: Vec::new(),
      },
      deletion_key: deletion_key.map(Into::into),
      files: files.into_iter().collect(),
    }
  }
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
