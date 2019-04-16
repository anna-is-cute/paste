use crate::{
  backend::pastes::{PastePayload, FilePayload},
  config::Config,
  errors::*,
};

use lazy_static::lazy_static;

use url::Url;

use reqwest::Client;

use rocket::{
  Request, Outcome,
  request::{self, FromRequest},
};

use std::net::{
  IpAddr,
  SocketAddr,
};

lazy_static! {
  static ref BASE_URL: Url = Url::parse("https://rest.akismet.com/1.1/").unwrap();
}

pub struct SubmitterInfo<'a> {
  ip: IpAddr,
  user_agent: &'a str,
  referrer: &'a str,
}

impl FromRequest<'a, 'r> for SubmitterInfo<'a> {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let socket_addr: SocketAddr = match request.guard() {
      Outcome::Success(s) => s,
      Outcome::Failure((status, _)) => return Outcome::Failure((status, ())),
      Outcome::Forward(_) => return Outcome::Forward(()),
    };
    let ip = socket_addr.ip();

    let user_agent = request.headers().get("User-Agent").last().unwrap_or_default();
    let referrer = request.headers().get("Referer").last().unwrap_or_default();

    Outcome::Success(SubmitterInfo {
      ip,
      user_agent,
      referrer,
    })
  }
}

#[derive(Debug, Clone, Copy)]
pub enum AkismetRoute {
  VerifyKey,
  CommentCheck,
  SubmitSpam,
  SubmitHam,
}

impl AkismetRoute {
  pub fn as_str(self) -> &'static str {
    match self {
      AkismetRoute::VerifyKey => "verify-key",
      AkismetRoute::CommentCheck => "comment-check",
      AkismetRoute::SubmitSpam => "submit-spam",
      AkismetRoute::SubmitHam => "submit-ham",
    }
  }
}

pub struct AkismetClient<'a> {
  config: &'a Config,
  client: &'a Client,
  api_key: &'a str,
}

impl<'a> AkismetClient<'a> {
  pub fn new(config: &'a Config, client: &'a Client) -> Self {
    let api_key = &config.spam.akismet.api_key;

    AkismetClient {
      config,
      client,
      api_key,
    }
  }

  fn make_url(&self, route: AkismetRoute) -> Url {
    let mut url = BASE_URL.clone();
    url.set_host(Some(&format!(
      "{}.{}",
      self.api_key,
      BASE_URL.host_str().expect("akismet base url host"),
    ))).expect("setting host on akismet base url");

    url.join(route.as_str()).expect("joining route on akismet base url")
  }

  pub fn check_file_spam(&self, info: &SubmitterInfo, payload: &FilePayload) -> Result<bool> {
    let blog = format!("https://{}/", self.config.general.site_domain);
    let mut params = AkismetParams::new(
      &blog,
      info.ip,
      info.user_agent,
    );
    params.referrer = Some(info.referrer);
    params.comment_type = Some("pastebin");
    params.is_test = Some(true);
    if let Ok(s) = std::str::from_utf8(payload.content.as_bytes()) {
      params.comment_content = Some(s);
    }

    let response_text = self.client.post(self.make_url(AkismetRoute::CommentCheck))
      .form(&params)
      .send()?
      .text()?;

    Ok(response_text.trim() == "true")
  }

  pub fn check_paste_spam(&self, info: &SubmitterInfo, payload: &PastePayload) -> Result<Vec<bool>> {
    payload.files.iter()
      .map(|f| self.check_file_spam(info, f))
      .collect()
  }
}

#[derive(Debug, Serialize)]
struct AkismetParams<'a> {
  blog: &'a str,
  user_ip: IpAddr,
  user_agent: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  referrer: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  permalink: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_type: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_author: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_author_email: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_author_url: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_content: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_date_gmt: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  comment_post_modified_gmt: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  blog_lang: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  blog_charset: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  user_role: Option<&'a str>,
  #[serde(skip_serializing_if = "Option::is_none")]
  is_test: Option<bool>,
}

impl AkismetParams<'a> {
  fn new(blog: &'a str, user_ip: IpAddr, user_agent: &'a str) -> Self {
    AkismetParams {
      blog,
      user_ip,
      user_agent,
      referrer: None,
      permalink: None,
      comment_type: None,
      comment_author: None,
      comment_author_email: None,
      comment_author_url: None,
      comment_content: None,
      comment_date_gmt: None,
      comment_post_modified_gmt: None,
      blog_lang: None,
      blog_charset: None,
      user_role: None,
      is_test: None,
    }
  }
}
