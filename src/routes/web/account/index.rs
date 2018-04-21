use config::Config;
use errors::*;
use routes::web::OptionalWebUser;

use rocket::http::Status as HttpStatus;
use rocket::request::Request;
use rocket::response::{Redirect, Responder, Response};
use rocket::State;

use rocket_contrib::Template;

use std::result;

enum Rt {
  Redirect(Redirect),
  Template(Template),
}

impl<'r> Responder<'r> for Rt {
  fn respond_to(self, request: &Request) -> result::Result<Response<'r>, HttpStatus> {
    match self {
      Rt::Redirect(r) => r.respond_to(request),
      Rt::Template(t) => t.respond_to(request),
    }
  }
}

#[get("/account")]
fn get(config: State<Config>, user: OptionalWebUser) -> Result<Rt> {
  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Rt::Redirect(Redirect::to("/login"))),
  };

  let ctx = json!({
    "config": &*config,
    "user": user,
  });
  Ok(Rt::Template(Template::render("account/index", ctx)))
}
