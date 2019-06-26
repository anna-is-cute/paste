use crate::errors::Result;

use reqwest::Client;

use url::Url;

pub enum HighlightKind {
  Snippet,
  File,
}

impl HighlightKind {
  fn as_str(&self) -> &str {
    match *self {
      HighlightKind::Snippet => "snippet",
      HighlightKind::File => "file",
    }
  }
}

pub fn highlight(client: &Client, kind: HighlightKind, name: &str, code: &str) -> Result<Vec<String>> {
  let mut url = Url::parse("http://highlight:8080/highlight/")?
    .join(kind.as_str())?;
  url.query_pairs_mut().append_pair("name", name);

  let highlighted = client.post(url)
    .body(code.to_owned())
    .send()?
    .text()?;

  let lines = split_lines_inclusive(&spanner::spanner(&highlighted))
    .into_iter()
    .map(ToOwned::to_owned)
    .collect();

  Ok(lines)
}

fn split_lines_inclusive(s: &str) -> Vec<&str> {
    let mut result = Vec::new();

    let mut last = 0;
    for (index, matched) in s.match_indices('\n') {
      if last <= index {
        result.push(&s[last..index + 1]);
      }
      last = index + matched.len();
    }
    if last < s.len() {
      result.push(&s[last..]);
    }

    result
}
