use routes::web::Session;

use rocket::{Outcome, Data};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Method, Status as HttpStatus};
use rocket::http::hyper::header::Location;
use rocket::request::Request;
use rocket::response::Response;

use uuid::Uuid;

use std::collections::HashMap;
use std::sync::RwLock;

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
