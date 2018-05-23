use models::id::{DeletionKeyId, PasteId, FileId, UserId};
use super::{Paste, Metadata, Visibility, Content};

#[derive(Debug, Serialize)]
pub struct Output {
  pub id: PasteId,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub author: Option<OutputAuthor>,
  #[serde(flatten)]
  pub paste: Paste,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub deletion_key: Option<DeletionKeyId>,
  pub files: Vec<OutputFile>,
}

impl Output {
  pub fn new<N, D, F>(paste_id: PasteId, author: Option<OutputAuthor>, name: Option<N>, desc: Option<D>, vis: Visibility, deletion_key: Option<DeletionKeyId>, files: F) -> Self
    where N: AsRef<str>,
          D: AsRef<str>,
          F: IntoIterator<Item = OutputFile>,
  {
    Output {
      id: paste_id,
      author,
      paste: Paste {
        metadata: Metadata {
          name: name.map(|x| x.as_ref().to_string().into()),
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
  id: FileId,
  name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  content: Option<Content>,
}

impl OutputFile {
  pub fn new<S: Into<String>>(id: FileId, name: Option<S>, content: Option<Content>) -> Self {
    OutputFile {
      id,
      name: name.map(Into::into),
      content,
    }
  }
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputAuthor {
  pub id: UserId,
  pub username: String,
  pub name: String,
}

impl OutputAuthor {
  pub fn new<U, S>(id: UserId, username: U, name: S) -> Self
    where U: Into<String>,
          S: Into<String>,
  {
    OutputAuthor {
      id,
      username: username.into(),
      name: name.into(),
    }
  }
}
