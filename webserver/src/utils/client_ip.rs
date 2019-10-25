use rocket::{
  http::Status,
  request::{
    self,
    FromRequest,
    Request,
  },
  Outcome,
};

use std::{
  net::IpAddr,
  ops::Deref,
};

#[derive(Debug)]
pub struct ClientIp(IpAddr);

impl Deref for ClientIp {
  type Target = IpAddr;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl FromRequest<'a, 'r> for ClientIp {
  type Error = ();

  fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    match req.client_ip() {
      Some(c) => Outcome::Success(ClientIp(c)),
      None => Outcome::Failure((Status::BadRequest, ())),
    }
  }
}
