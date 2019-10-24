use regex::Regex;

use serde_derive::{
  Deserialize as Deserialise,
  Serialize as Serialise,
};

use crate::{
  backend::pastes::models::PastePayload,
  models::paste::Content,
};

#[derive(Debug, Deserialise, Serialise)]
pub struct Filter {
  #[serde(flatten)]
  pub kind: FilterKind,
  #[serde(flatten)]
  pub action: Action,
  pub on: Vec<FilterSearch>,
  pub by: Vec<FilterTarget>,
}

impl Filter {
  pub fn matches<'u>(&self, paste: &PastePayload<'u>) -> bool {
    let by = self.by
      .iter()
      .any(|by| match by {
        FilterTarget::Anonymous if paste.author.is_none() => true,
        FilterTarget::Authenticated if paste.author.is_some() => true,
        _ => false,
      });
    if !by {
      return false;
    }

    for on in &self.on {
      let matches = match on {
        FilterSearch::Title => paste.name
          .as_ref()
          .map(|name| self.kind.matches(&name))
          .unwrap_or(false),
        FilterSearch::Description => paste.description
          .as_ref()
          .map(|desc| self.kind.matches(&desc))
          .unwrap_or(false),
        FilterSearch::FileName => paste.files
          .iter()
          .flat_map(|file| file.name.as_ref())
          .any(|name| self.kind.matches(&name)),
        FilterSearch::Content => paste.files
          .iter()
          .map(|file| &file.content)
          .any(|content| match content {
            Content::Text(t) if self.kind.matches(&t) => true,
            _ => false,
          }),
      };
      if matches {
        return true;
      }
    }

    false
  }
}

#[derive(Debug, Deserialise, Serialise)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FilterKind {
  Regex(RegexFilter),
}

impl FilterKind {
  fn matches<S: AsRef<str>>(&self, s: S) -> bool {
    match self {
      Self::Regex(f) => f.query.is_match(s.as_ref()),
    }
  }
}

#[derive(Debug, Deserialise, Serialise)]
pub struct RegexFilter {
  #[serde(rename = "match", with = "serde_regex")]
  query: Regex,
}

#[derive(Debug, Deserialise, Serialise)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
  Allow,
  Block,
  FakeError {
    message: String,
  },
  Ignore,
}

#[derive(Debug, Deserialise, Serialise)]
#[serde(rename_all = "snake_case")]
pub enum FilterSearch {
  Title,
  Description,
  Content,
  FileName,
}

#[derive(Debug, Deserialise, Serialise)]
#[serde(rename_all = "snake_case")]
pub enum FilterTarget {
  Anonymous,
  Authenticated,
}
