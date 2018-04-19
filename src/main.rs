#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate base64;
extern crate chrono;
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

use rocket_contrib::Template;

#[get("/")]
fn index() -> std::io::Result<NamedFile> {
  NamedFile::open("index.html")
}

fn main() {
  dotenv::dotenv().ok();

  rocket::ignite()
    .manage(database::init_pool())
    .attach(Template::fairing())
    .catch(errors![
      routes::bad_request,
      routes::not_found,
      routes::internal_server_error,
    ])
    .mount("/", routes![
      routes::web::index::get,
      routes::web::auth::register::get,
    ])
    .mount("/static", routes!{
      routes::web::static_files::get,
    })
    .mount("/api/v0/pastes", routes![
      routes::pastes::get::get_all,
      routes::pastes::get::get_all_query,

      routes::pastes::post::post,
      routes::pastes::delete::delete,
      routes::pastes::get::get_query,
      routes::pastes::get::get,
      routes::pastes::patch::patch,

      routes::pastes::files::get::get,
      routes::pastes::files::patch::patch,
      routes::pastes::files::post::post,

      routes::pastes::files::individual::delete::delete,
      routes::pastes::files::individual::get::get,
      routes::pastes::files::individual::patch::patch,

      routes::pastes::files::individual::raw::get::get,
    ])
    .launch();
}
