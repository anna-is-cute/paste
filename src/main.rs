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
extern crate lazy_static;
extern crate libflate;
extern crate rocket_contrib;
extern crate rocket;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate uuid;
extern crate xz2;

mod database;
mod errors;
mod models;
mod routes;
mod store;

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

fn main() {
  dotenv::dotenv().ok();

  rocket::ignite()
    .manage(database::init_pool())
    .mount("/", routes![index])
    .mount("/api/pastes", routes![
      routes::pastes::get::get,
      routes::pastes::get::get_query,
      routes::pastes::create::create,
      routes::pastes::edit,
      routes::pastes::delete,

      routes::pastes::files::get::get_files,
      routes::pastes::files::file::get_file_id,
    ])
    .launch();
}
