use super::{Paste, Content};

#[derive(Debug, Serialize)]
pub struct Output {
  #[serde(flatten)]
  pub paste: Paste,
  pub files: Vec<OutputFile>,
}

#[derive(Debug, Serialize)]
pub struct OutputFile {
  pub id: String,
  pub name: Option<String>,
  // #[serde(flatten)]
  // FIXME: all of this mess
  // flatten does not work on optional types
  // flatten cannot be combined with skips
  // this will leave "content": null in the json, which is not desired
  #[serde(skip_serializing_if = "Option::is_none")]
  pub content: Option<Content>,
}
