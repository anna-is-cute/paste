use rocket::{
  Outcome,
  request::{self, Request, FromRequest},
};

use unic_langid::LanguageIdentifier;

use std::convert::Infallible;

pub struct AcceptLanguage(pub Vec<LanguageIdentifier>);

impl AcceptLanguage {
  pub fn into_strings(self) -> Vec<String> {
    self.0.into_iter().map(|x| x.to_string()).collect()
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for AcceptLanguage {
  type Error = Infallible;

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let langs = request.headers()
      .get("accept-language")
      .flat_map(|x| x.split(','))
      // only accept 10 possible languages
      .take(10)
      .map(|x| x.split(';').next().unwrap().trim())
      .flat_map(str::parse)
      .collect();
    Outcome::Success(AcceptLanguage(langs))
  }
}

impl std::ops::Deref for AcceptLanguage {
  type Target = Vec<LanguageIdentifier>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
