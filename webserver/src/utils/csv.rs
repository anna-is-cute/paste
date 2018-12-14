use tera::escape_html;

use csv::{Error, ErrorKind, Reader};

pub fn csv_to_table(content: &str) -> Result<String, String> {
  let mut table = String::new();

  let mut reader = Reader::from_reader(content.as_bytes());

  let headers = reader.headers().map_err(pretty_error)?;

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
    for field in rec.map_err(pretty_error)?.iter() {
      let safe_field = escape_html(&field);
      table.push_str(&format!("      <td>{}</td>\n", safe_field));
    }
    table.push_str("    </tr>\n");
  }
  table.push_str("  </tbody>\n");
  table.push_str("</table>\n");

  Ok(table)
}

fn pretty_error(e: Error) -> String {
  let opening = "paste would like to show you this CSV file as a table, but it";

  let explanation = match e.kind() {
    ErrorKind::Utf8 { pos, err } => format!(
      "couldn't be read as valid UTF-8{}: {}",
      pos.as_ref().map(|x| format!(" at line {} (byte {})", x.line(), x.byte())).unwrap_or_default(),
      err,
    ),
    ErrorKind::UnequalLengths { pos, expected_len, len } => format!(
      "has a row with {} field{}{} while the previous row had {} field{}",
      len,
      if *len == 1 { "" } else { "s" },
      pos.as_ref().map(|x| format!(" (line {}, byte {})", x.line(), x.byte())).unwrap_or_default(),
      expected_len,
      if *expected_len == 1 { "" } else { "s" },
    ),
    _ => return e.to_string(),
  };

  format!("{} {}.", opening, explanation)
}
