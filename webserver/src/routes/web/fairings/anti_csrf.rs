use crate::routes::web::Session;

use hex;

use rocket::{
  Data, Outcome,
  fairing::{Fairing, Info, Kind},
  http::Method,
  request::Request,
};

use sodiumoxide::randombytes;

pub struct AntiCsrf;

impl AntiCsrf {
  fn on_get(req: &mut Request) {
    let path = req.uri().path();

    if path.starts_with("/api/") || path.starts_with("/static/") {
      return;
    }

    let mut session: Session = match req.guard() {
      Outcome::Success(s) => s,
      _ => return,
    };

    session.purge_tokens();

    let token = hex::encode(randombytes::randombytes(64));
    session.add_token(token.as_str());
    session.data.insert("anti_csrf_token".into(), token);
  }
}

impl Fairing for AntiCsrf {
  fn info(&self) -> Info {
    Info {
      name: "Anti-CSRF tokens",
      kind: Kind::Request | Kind::Response,
    }
  }

  fn on_request(&self, req: &mut Request, _: &Data) {
    // generate a token for every get request
    if req.method() != Method::Get && req.method() != Method::Head {
      return;
    }
    AntiCsrf::on_get(req);
  }
}
