#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate base64;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate failure;
extern crate git2;
#[macro_use]
extern crate if_chain;
#[macro_use]
extern crate lazy_static;
extern crate libflate;
extern crate rocket_contrib;
extern crate rocket;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate unicode_segmentation;
extern crate uuid;
extern crate xz2;

mod database;
mod errors;
mod models;
mod routes;
mod store;
mod utils;

use rocket::response::NamedFile;

#[get("/")]
fn index() -> std::io::Result<NamedFile> {
  NamedFile::open("index.html")
}

fn main() {
  dotenv::dotenv().ok();

  rocket::ignite()
    .manage(database::init_pool())
    .catch(errors![
      routes::bad_request,
      routes::not_found,
      routes::internal_server_error,
    ])
    .mount("/", routes![index])
    .mount("/api/pastes", routes![
      routes::pastes::post::post,
      routes::pastes::delete::delete,
      routes::pastes::get::get_query,
      routes::pastes::get::get,
      routes::pastes::patch::patch,

      routes::pastes::files::file::get_file_id,
      routes::pastes::files::get::get_files,
    ])
    .launch();
}
