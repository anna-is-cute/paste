#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod pastes;

#[get("/")]
fn index() -> &'static str {
  "Hello, world!"
}

fn main() {
  rocket::ignite()
    .mount("/", routes![index])
    .mount("/api/pastes", routes![
      pastes::get,
      pastes::create::create,
      pastes::edit,
      pastes::delete,
    ])
    .launch();
}
