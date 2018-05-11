use super::Session;

use rocket::Outcome;
use rocket::request::{self, Request, FromRequest};

#[derive(Debug, Serialize, Deserialize)]
pub struct AntiCsrfToken(pub Option<String>);

impl AntiCsrfToken {
  pub fn check(&self, token: &str) -> bool {
    Some(token) == self.0.as_ref().map(|x| x.as_str())
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for AntiCsrfToken {
  type Error = String;

  fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let session: Session = otry!(req.guard());

    Outcome::Success(AntiCsrfToken(session.data.get("anti_csrf_token").cloned()))
  }
}
