use super::{Paste, Metadata, Visibility, Content};
use utils::SimpleUuid;

use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Output {
  pub id: SimpleUuid,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub author: Option<OutputAuthor>,
  #[serde(flatten)]
  pub paste: Paste,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub deletion_key: Option<SimpleUuid>,
  pub files: Vec<OutputFile>,
}

impl Output {
  pub fn new<N, D, F>(paste_id: Uuid, author: Option<OutputAuthor>, name: Option<N>, desc: Option<D>, vis: Visibility, deletion_key: Option<Uuid>, files: F) -> Self
    where N: AsRef<str>,
          D: AsRef<str>,
          F: IntoIterator<Item = OutputFile>,
  {
    Output {
      id: paste_id.into(),
      author,
      paste: Paste {
        metadata: Metadata {
          name: name.map(|x| x.as_ref().to_string()),
          description: desc.map(|x| x.as_ref().to_string().into()),
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

#[derive(Debug, Serialize)]
pub struct OutputAuthor {
  id: SimpleUuid,
  username: String,
}

impl OutputAuthor {
  pub fn new<S: Into<String>>(id: &Uuid, username: S) -> Self {
    OutputAuthor {
      id: (*id).into(),
      username: username.into(),
    }
  }
}
