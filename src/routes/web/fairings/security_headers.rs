use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::request::Request;
use rocket::response::Response;

pub struct SecurityHeaders;

impl Fairing for SecurityHeaders {
  fn info(&self) -> Info {
    Info {
      name: "Security headers",
      kind: Kind::Response,
    }
  }

  fn on_response(&self, _: &Request, resp: &mut Response) {
    resp.set_header(Header::new("X-Frame-Options", "SAMEORIGIN"));
    resp.set_header(Header::new("X-XSS-Protection", "1; mode=block"));
    resp.set_header(Header::new("X-Content-Type-Options", "nosniff"));
    resp.set_header(Header::new("Referrer-Policy", "origin-when-cross-origin"));
  }
}
