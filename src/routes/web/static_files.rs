use std::io;
use std::path::PathBuf;

use rocket::response::NamedFile;

#[get("/<path..>")]
fn get(path: PathBuf) -> io::Result<NamedFile> {
  let static_path = PathBuf::from("web/static/");
  let resource_path = static_path.join(path);
  NamedFile::open(resource_path)
}
