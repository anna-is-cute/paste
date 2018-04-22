use rocket::http::{Cookies, Cookie};
use rocket::response::Redirect;

#[post("/logout")]
fn post(mut cookies: Cookies) -> Redirect {
  cookies.remove_private(Cookie::named("user_id"));

  Redirect::to("/")
}
