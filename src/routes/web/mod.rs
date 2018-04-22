use database::PostgresPool;
use database::models::users::User;
use database::schema::users;

use rocket::{State, Outcome, Data};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Cookie, Method, Status as HttpStatus};
use rocket::http::hyper::header::Location;
use rocket::request::{self, Request, FromRequest};
use rocket::response::{Responder, Response, Redirect};

use rocket_contrib::Template;

use diesel::prelude::*;

use uuid::Uuid;

use std::collections::HashMap;
use std::ops::Deref;
use std::result;
use std::str::FromStr;
use std::sync::RwLock;

pub mod account;
pub mod auth;
pub mod index;
pub mod pastes;
pub mod static_files;

#[derive(Debug)]
pub struct OptionalWebUser(Option<User>);

impl OptionalWebUser {
  pub fn into_inner(self) -> Option<User> {
    self.0
  }
}

impl<'a, 'r> FromRequest<'a, 'r> for OptionalWebUser {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let id = request
      .cookies()
      .get_private("user_id")
      .and_then(|x| Uuid::parse_str(x.value()).ok());
    let id = match id {
      Some(id) => id,
      None => return Outcome::Success(OptionalWebUser(None)),
    };

    let conn = match request.guard::<State<PostgresPool>>() {
      Outcome::Success(p) => match p.get() {
        Ok(c) => c,
        Err(_) => return Outcome::Failure((HttpStatus::ServiceUnavailable, ())),
      },
      Outcome::Failure((status, _)) => return Outcome::Failure((status, ())),
      Outcome::Forward(()) => return Outcome::Forward(()),
    };

    match users::table.find(id).first(&*conn) {
      Ok(u) => Outcome::Success(OptionalWebUser(Some(u))),
      Err(_) => Outcome::Success(OptionalWebUser(None)),
    }
  }
}

impl Deref for OptionalWebUser {
  type Target = Option<User>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub enum Rst {
  Redirect(Redirect),
  Status(HttpStatus),
  Template(Template),
}

impl<'r> Responder<'r> for Rst {
  fn respond_to(self, request: &Request) -> result::Result<Response<'r>, HttpStatus> {
    match self {
      Rst::Redirect(r) => r.respond_to(request),
      Rst::Status(s) => Err(s),
      Rst::Template(t) => t.respond_to(request),
    }
  }
}


#[derive(Debug, Default)]
pub struct LastPage {
  map: RwLock<HashMap<Uuid, String>>,
}

impl Fairing for LastPage {
  fn info(&self) -> Info {
    Info {
      name: "Last page handler",
      kind: Kind::Request | Kind::Response,
    }
  }

  fn on_request(&self, req: &mut Request, _: &Data) {
    // only work on get requests
    if req.method() != Method::Get {
      return;
    }

    // get current path
    let path = req.uri().path();

    // don't track auth pages
    if path == "/login" || path == "/register" || path == "/favicon.ico" || path.starts_with("/static/") {
      return;
    }

    // get session (private cookie, so encrypted and authed)
    let (add, session) = match req.cookies().get_private("session") {
      Some(s) => (false, s.value().to_string()),
      None => (true, Uuid::new_v4().simple().to_string()),
    };

    if add {
      req.cookies().add_private(Cookie::new("session", session.clone()));
    }

    // get session as UUID
    let sess_id = match Uuid::from_str(&session) {
      Ok(u) => u,
      Err(_) => return,
    };

    // write this path as the last page for this session
    self.map.write().unwrap().insert(sess_id, path.to_string());
  }

  fn on_response(&self, req: &Request, resp: &mut Response) {
    if resp.status() != HttpStatus::SeeOther {
      return;
    }

    let loc = match resp.headers().get("Location").next() {
      Some(l) => l.to_string(),
      None => return,
    };

    if loc != "lastpage" {
      return;
    }

    // set header to / in case no session
    resp.set_header(Location("/".into()));

    // get session (private cookie, so encrypted and authed)
    let session = match req.cookies().get_private("session") {
      Some(s) => s,
      None => return,
    };

    // get session as UUID
    let sess_id = match Uuid::from_str(session.value()) {
      Ok(u) => u,
      Err(_) => return,
    };

    // write this path as the last page for this session
    let last_page = match self.map.read().unwrap().get(&sess_id) {
      Some(l) => l.clone(),
      None => return,
    };

    resp.set_header(Location(last_page));
  }
}
