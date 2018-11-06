use std::path::PathBuf;

#[cfg(debug_assertions)]
use rocket::response::NamedFile;

#[cfg(not(debug_assertions))]
use rocket::response::status::BadRequest;

#[cfg(debug_assertions)]
#[get("/<path..>", rank = 1)]
pub fn get(path: PathBuf) -> std::io::Result<NamedFile> {
  let static_path = PathBuf::from("webserver/web/static/");
  let resource_path = static_path.join(path);
  NamedFile::open(resource_path)
}

#[cfg(not(debug_assertions))]
#[get("/<_path..>", rank = 1)]
pub fn get(_path: PathBuf) -> BadRequest<&'static str> {
  BadRequest(Some("The /static route is disabled in release versions. Please set up a reverse proxy and handle static files through it."))
}
