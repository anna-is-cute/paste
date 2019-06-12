use rocket::{
  fairing::{Fairing, Info, Kind},
  http::Header,
  request::Request,
  response::Response,
};

pub struct SecurityHeaders;

impl Fairing for SecurityHeaders {
  fn info(&self) -> Info {
    Info {
      name: "Security headers",
      kind: Kind::Response,
    }
  }

  fn on_response(&self, req: &Request, resp: &mut Response) {
    resp.set_header(Header::new("X-Frame-Options", "DENY"));
    resp.set_header(Header::new("X-XSS-Protection", "1; mode=block"));
    resp.set_header(Header::new("X-Content-Type-Options", "nosniff"));
    resp.set_header(Header::new("Referrer-Policy", "strict-origin-when-cross-origin"));

    if req.uri().path().starts_with("/api/") {
      resp.set_header(Header::new("Access-Control-Allow-Origin", "*"));
    }
  }
}
