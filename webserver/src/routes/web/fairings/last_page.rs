use models::id::SessionId;
use routes::web::Session;

use rocket::Outcome;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Method, Status as HttpStatus};
use rocket::http::hyper::header::Location;
use rocket::request::Request;
use rocket::response::Response;

use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct LastPage {
  map: RwLock<HashMap<SessionId, String>>,
}

impl LastPage {
  fn store(&self, req: &Request) {
    // get current path
    let path = req.uri().path();

    // don't track auth pages
    if path == "/login"
      || path == "/register"
      || path == "/favicon.ico"
      || path.starts_with("/static/")
      || path.starts_with("/account/reset_password")
    {
      return;
    }

    let session: Session = match req.guard() {
      Outcome::Success(s) => s,
      _ => {
        println!("could not create new session");
        return;
      }
    };

    // get session as UUID
    let sess_id = session.id;

    // write this path as the last page for this session
    self.map.write().unwrap().insert(sess_id, path.to_string());
  }
}

impl Fairing for LastPage {
  fn info(&self) -> Info {
    Info {
      name: "Last page handler",
      kind: Kind::Response,
    }
  }

  fn on_response(&self, req: &Request, resp: &mut Response) {
    if req.method() == Method::Get && resp.status() != HttpStatus::SeeOther {
      self.store(req);
      return;
    }

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
    let session: Session = match req.guard() {
      Outcome::Success(s) => s,
      _ => {
        println!("could not create new session");
        return;
      }
    };
    // get session as UUID
    let sess_id = session.id;

    // write this path as the last page for this session
    let last_page = match self.map.read().unwrap().get(&sess_id) {
      Some(l) => l.clone(),
      None => return,
    };

    resp.set_header(Location(last_page));
  }
}
