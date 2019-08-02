use r2d2::{ManageConnection, Pool, PooledConnection};

use rocket::{
  Outcome,
  Request,
  State,
  http::Status,
  request::{
    self,
    FromRequest,
  },
};

use sodiumoxide::randombytes::randombytes;

use url::Url;

use websocket::{
  ClientBuilder,
  message::{Message, OwnedMessage},
  result::WebSocketError,
  sync::{
    Client,
    stream::NetworkStream,
  },
};

use std::ops::{Deref, DerefMut};

lazy_static! {
  static ref HIGHLIGHT_URL: Url = Url::parse(&std::env::var("HIGHLIGHT_URL")
    .expect("missing HIGHLIGHT_URL env var"))
    .expect("HIGHLIGHT_URL env var was not a valid url");
}

pub fn init_pool() -> Pool<ConnectionManager> {
  let manager = ConnectionManager::new(HIGHLIGHT_URL.clone());
  Pool::new(manager).expect("ws pool")
}

pub struct ConnectionManager {
  url: Url,
}

impl ConnectionManager {
  pub fn new(url: Url) -> Self {
    ConnectionManager { url }
  }

  // pub fn from_str(url: &str) -> Result<Self, url::ParseError> {
  //   let url = Url::parse(url)?;
  //   Ok(ConnectionManager::new(url))
  // }
}

impl ManageConnection for ConnectionManager {
  type Connection = Client<Box<dyn NetworkStream + Send>>;
  type Error = WebSocketError;

  fn connect(&self) -> Result<Self::Connection, Self::Error> {
    ClientBuilder::from_url(&self.url).connect(None)
  }

  fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
    let bytes = randombytes(16);
    conn.send_message(&Message::ping(bytes.as_slice()))?;
    if conn.recv_message()? != OwnedMessage::Pong(bytes) {
      return Err(WebSocketError::ResponseError("pong did not have correct bytes"));
    }
    Ok(())
  }

  fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
    false
  }
}

pub struct WebSocket(pub PooledConnection<ConnectionManager>);

impl FromRequest<'a, 'r> for WebSocket {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let pool: State<Pool<ConnectionManager>> = request.guard()?;
    match pool.get() {
      Ok(conn) => Outcome::Success(WebSocket(conn)),
      Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
    }
  }
}

impl Deref for WebSocket {
  type Target = Client<Box<dyn NetworkStream + Send>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for WebSocket {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
