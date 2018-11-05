use crate::routes::web::Session;

use rocket::{
  request::Form,
  response::Redirect,
};

#[post("/logout", data = "<data>")]
fn post(data: Form<Logout>, mut sess: Session) -> Redirect {
  let data = data.into_inner();
  if !sess.check_token(&data.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Redirect::to("lastpage");
  }

  sess.user_id = None;

  Redirect::to("lastpage")
}

#[derive(FromForm)]
struct Logout {
  anti_csrf_token: String,
}
