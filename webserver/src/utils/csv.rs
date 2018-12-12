use tera::escape_html;

use csv::Reader;

pub fn csv_to_table(content: &str) -> Option<String> {
  let mut table = String::new();

  let mut reader = Reader::from_reader(content.as_bytes());

  let headers = reader.headers().ok()?;

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
    for field in rec.ok()?.iter() {
      let safe_field = escape_html(&field);
      table.push_str(&format!("      <td>{}</td>\n", safe_field));
    }
    table.push_str("    </tr>\n");
  }
  table.push_str("  </tbody>\n");
  table.push_str("</table>\n");

  Some(table)
}
