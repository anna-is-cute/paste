use routes::web::{AntiCsrfToken, Session};

use rocket::http::{Cookies, Cookie};
use rocket::request::Form;
use rocket::response::Redirect;

#[post("/logout", data = "<data>")]
fn post(data: Form<Logout>, csrf: AntiCsrfToken, mut sess: Session, mut cookies: Cookies) -> Redirect {
  let data = data.into_inner();
  if !csrf.check(&data.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Redirect::to("lastpage");
  }

  cookies.remove_private(Cookie::named("user_id"));

  Redirect::to("lastpage")
}

#[derive(FromForm)]
struct Logout {
  anti_csrf_token: String,
}
