use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;

use r2d2::{Pool, PooledConnection};

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};

use std::env;
use std::ops::Deref;

pub mod models;
pub mod schema;

pub type PostgresPool = Pool<ConnectionManager<PgConnection>>;

lazy_static! {
  static ref DATABASE_URL: String = env::var("DATABASE_URL").expect("missing DATABASE_URL env var");
}

pub fn init_pool() -> PostgresPool {
  let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.as_str());
  Pool::new(manager).expect("db pool")
}

pub struct DbConn(pub PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
  type Error = ();

  fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
    let pool: State<PostgresPool> = request.guard()?;
    match pool.get() {
      Ok(conn) => Outcome::Success(DbConn(conn)),
      Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
    }
  }
}

impl Deref for DbConn {
  type Target = PgConnection;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
