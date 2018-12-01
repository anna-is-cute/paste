use rocket::{
  fairing::{Fairing, Info, Kind},
  http::{ContentType, Header},
  request::Request,
  response::Response,
};

pub struct Push;

impl Fairing for Push {
  fn info(&self) -> Info {
    Info {
      name: "HTTP/2 push",
      kind: Kind::Response,
    }
  }

  fn on_response(&self, _: &Request, response: &mut Response) {
    if response.content_type() != Some(ContentType::HTML) {
      return;
    }

    let version = crate::RESOURCES_VERSION.as_ref().map(|x| x.as_str()).unwrap_or_default();

    response.adjoin_header(Header::new(
      "Link",
      format!("</static/js/detect-js.js?v={}>; rel=preload; as=script", version),
    ));
    response.adjoin_header(Header::new(
      "Link",
      format!("</static/css/style.css?v={}>; rel=preload; as=style", version),
    ));
    response.adjoin_header(Header::new(
      "Link",
      format!("</static/css/dark-style.css?v={}>; rel=preload; as=style", version),
    ));
    response.adjoin_header(Header::new(
      "Link",
      format!("</static/js/style.js?v={}>; rel=preload; as=script", version),
    ));
  }
}
