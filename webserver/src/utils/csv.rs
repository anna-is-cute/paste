use tera::escape_html;

use csv::{Error, ErrorKind, Reader};

use crate::{
  errors::*,
  i18n::L10n,
};

pub fn csv_to_table(content: &str, l10n: &L10n) -> std::result::Result<String, Result<String>> {
  let mut table = String::new();

  let mut reader = Reader::from_reader(content.as_bytes());

  let headers = reader.headers().map_err(|e| pretty_error(e, l10n))?;

  table.push_str("<table>\n");
  table.push_str("  <thead>\n");
  table.push_str("    <tr>\n");
  for header in headers.iter() {
    let safe_header = escape_html(&header);
    table.push_str(&format!("      <th>{}</th>\n", safe_header));
  }
  table.push_str("    </tr>\n");
  table.push_str("  </thead>\n");
  table.push_str("  <tbody>\n");
  for rec in reader.records() {
    table.push_str("    <tr>\n");
    for field in rec.map_err(|e| pretty_error(e, l10n))?.iter() {
      let safe_field = escape_html(&field);
      table.push_str(&format!("      <td>{}</td>\n", safe_field));
    }
    table.push_str("    </tr>\n");
  }
  table.push_str("  </tbody>\n");
  table.push_str("</table>\n");

  Ok(table)
}

fn pretty_error(e: Error, l10n: &L10n) -> Result<String> {
  let stringified = e.to_string();

  let explanation = match e.into_kind() {
    ErrorKind::Utf8 { pos: Some(pos), err } => l10n.tr_ex(
      ("csv-error", "utf-8-pos"),
      |req| req
        .arg("line", pos.line())
        .arg("byte", pos.byte())
        .arg("err", err.to_string()),
    )?,
    ErrorKind::Utf8 { pos: None, err } => l10n.tr_ex(
      ("csv-error", "utf-8"),
      |req| req.arg("err", err.to_string()),
    )?,
    ErrorKind::UnequalLengths { pos: Some(pos), expected_len, len } => l10n.tr_ex(
      ("csv-error", "lengths-pos"),
      |req| req
        .arg("secondRowFields", len)
        .arg("firstRowFields", expected_len)
        .arg("line", pos.line())
        .arg("byte", pos.byte()),
    )?,
    ErrorKind::UnequalLengths { pos: None, expected_len, len } => l10n.tr_ex(
      ("csv-error", "lengths"),
      |req| req
        .arg("secondRowFields", len)
        .arg("firstRowFields", expected_len),
    )?,
    _ => return Ok(stringified),
  };

  Ok(explanation)
}
