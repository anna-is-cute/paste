use errors::*;

use reqwest::Client;

use rocket::request::FromFormValue;
use rocket::http::RawStr;

use serde_json;

use std::result;

thread_local! {
  static CLIENT: Client = Client::new();
}

#[derive(Debug)]
pub struct ReCaptcha(String);

impl ReCaptcha {
  pub fn verify(&self, secret: &str) -> Result<bool> {
    let data = ReCaptchaData {
      secret,
      response: &self.0,
      remote_ip: None,
    };
    let response = CLIENT.with(|c| c
      .post("https://www.google.com/recaptcha/api/siteverify")
      .form(&data)
      .send()
    )?;

    let response: ReCaptchaResponse = serde_json::from_reader(response)?;

    Ok(response.success)
  }
}

impl<'v> FromFormValue<'v> for ReCaptcha {
  type Error = &'v RawStr;

  fn from_form_value(form_value: &'v RawStr) -> result::Result<Self, Self::Error> {
    let string = String::from_form_value(form_value)?;
    Ok(ReCaptcha(string))
  }
}

#[derive(Debug, Serialize)]
struct ReCaptchaData<'a> {
  secret: &'a str,
  response: &'a str,
  #[serde(rename = "remoteip")]
  remote_ip: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReCaptchaResponse {
  success: bool,
  #[serde(default)]
  challenge_ts: Option<String>,
  #[serde(default)]
  hostname: Option<String>,
}
