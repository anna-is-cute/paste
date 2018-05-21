extern crate git2;

use git2::Repository;

use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
  let to_write = match version() {
    Some(v) => format!("Some(\"{}\")", v),
    None => "None".into(),
  };

  let out_dir = std::env::var("OUT_DIR").unwrap();
  let p = Path::new(&out_dir);
  let mut f = File::create(p.join("version")).unwrap();
  f.write_all(to_write.as_bytes()).unwrap();
}

fn version() -> Option<String> {
  let repo = Repository::open("..").ok()?;
  let revparse = repo.revparse_single("HEAD").ok()?;
  Some(revparse.id().to_string())
}
