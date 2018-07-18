use crate::{
  models::id::SessionId,
  redis_store::Redis,
  routes::web::guards::AntiCsrfToken,
};

use chrono::{Utc, Duration};

use cookie::{Cookie, SameSite};

use redis::{Commands, Value};

use rocket::{
  Outcome,
  request::{self, Request, FromRequest},
};

use serde::Serialize;

use serde_json::{Value as JsonValue, json, json_internal};

use uuid::Uuid;

use std::{collections::HashMap, str::FromStr};

// set session expiration to one day
const SESS_EXPIRE: usize = 24 * 60 * 60;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session<'a, 'r> where 'r: 'a {
  #[serde(skip)]
  pub request: Option<&'a Request<'r>>,
  pub id: SessionId,
  pub data: HashMap<String, String>,
  #[serde(default)]
  pub json: HashMap<String, JsonValue>,
  pub anti_csrf_tokens: Vec<AntiCsrfToken>,
}

impl Session<'a, 'r> {
  pub fn new(id: SessionId, request: &'a Request<'r>) -> Self {
    Session {
      request: Some(request),
      id,
      data: Default::default(),
      json: Default::default(),
      anti_csrf_tokens: Default::default(),
    }
  }

  pub fn set_form<T: Serialize>(&mut self, value: T) {
    self.json.insert("form".into(), json!(value));
  }

  pub fn take_form(&mut self) -> Option<JsonValue> {
    self.json.remove("form")
  }

  pub fn add_data<K, V>(&mut self, key: K, value: V)
    where K: Into<String>,
          V: Into<String>,
  {
    self.data.insert(key.into(), value.into());
  }

  pub fn add_token<S: Into<String>>(&mut self, token: S) {
    let expiry = Utc::now() + Duration::minutes(30);
    let token = AntiCsrfToken(
      token.into(),
      expiry,
    );
    self.anti_csrf_tokens.push(token);
  }

  pub fn purge_tokens(&mut self) {
    let now = Utc::now();
    self.anti_csrf_tokens.retain(|x| now <= x.1);

    if self.anti_csrf_tokens.len() > 50 {
      self.anti_csrf_tokens.reverse();
      self.anti_csrf_tokens.truncate(50);
      self.anti_csrf_tokens.reverse();
    }
  }

  pub fn check_token(&mut self, token: &str) -> bool {
    self.purge_tokens();

    if let Some(i) = self.anti_csrf_tokens.iter().position(|x| x.0 == token) {
      self.anti_csrf_tokens.remove(i);
      return true;
    }

    false
  }
}

impl FromRequest<'a, 'r> for Session<'a, 'r> {
    type Error = String;

    fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
      // get session (private cookie, so encrypted and authed)
      let sess_id: Option<SessionId> = req
        .cookies()
        .get_private("session")
        .and_then(|x| Uuid::from_str(&x.value()).ok())
        .map(SessionId);

      let sess_id = match sess_id {
        Some(s) => s,
        None => return Outcome::Success(Session::new(SessionId(Uuid::new_v4()), req)),
      };

      let redis: Redis = match req.guard() {
        Outcome::Success(s) => s,
        Outcome::Failure((status, _)) => return Outcome::Failure((status, "could not get redis connection".into())),
        Outcome::Forward(()) => return Outcome::Forward(()),
      };

      let json: String = match redis.get(format!("session:{}", sess_id.simple())) {
        Ok(s) => s,
        Err(_) => return Outcome::Success(Session::new(SessionId(Uuid::new_v4()), req)),
      };

      let mut session: Session = match serde_json::from_str(&json) {
        Ok(s) => s,
        // if we receive invalid json, just make a new session and let the old one die
        Err(_) => return Outcome::Success(Session::new(SessionId(Uuid::new_v4()), req)),
      };

      session.request = Some(req);

      Outcome::Success(session)
    }
}

impl Drop for Session<'a, 'r> {
  fn drop(&mut self) {
    if let Some(req) = self.request {
      let json = match serde_json::to_string(self) {
        Ok(b) => b,
        Err(e) => {
          println!("could not serialize session: {}", e);
          return;
        },
      };

      let redis: Redis = match req.guard() {
        Outcome::Success(s) => s,
        Outcome::Failure(_) | Outcome::Forward(_) => {
          println!("could not get redis connection");
          return;
        },
      };

      let id = self.id.simple().to_string();

      match redis.set_ex(format!("session:{}", id), json, SESS_EXPIRE) {
        Ok(Value::Okay) => {},
        Ok(Value::Status(s)) => println!("redis responded with an unexpected status: {}", s),
        Ok(x) => println!("redis responded strangely: {:?}", x),
        Err(e) => {
          println!("could not update redis: {}", e);
          return;
        },
      }

      let current_cookie = req.cookies().get_private("session");
      let current_id = current_cookie.as_ref().map(|x| x.value());

      if current_id != Some(&id) {
        let cookie = Cookie::build("session", id)
          .secure(true)
          .http_only(true)
          .same_site(SameSite::Lax)
          .finish();
        req.cookies().add_private(cookie);
      }
    }
  }
}
