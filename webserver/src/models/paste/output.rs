use models::id::{DeletionKeyId, PasteId, FileId, UserId};
use super::{Paste, Metadata, Visibility, Content};
use utils::Language;

use chrono::{DateTime, Utc};

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
  pub fn new<N, D, F>(
    paste_id: PasteId,
    author: Option<OutputAuthor>,
    name: Option<N>,
    desc: Option<D>,
    vis: Visibility,
    created_at: DateTime<Utc>,
    expires: Option<DateTime<Utc>>,
    deletion_key: Option<DeletionKeyId>,
    files: F,
  ) -> Self
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
          expires,
          created_at: Some(created_at),
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
  pub id: FileId,
  pub name: Option<String>,
  pub highlight_language: Option<&'static str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<Content>,
}

impl OutputFile {
  pub fn new<S: Into<String>>(id: FileId, name: Option<S>, language: Option<Language>, content: Option<Content>) -> Self {
    OutputFile {
      id,
      name: name.map(Into::into),
      highlight_language: language.map(|x| x.hljs()),
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
