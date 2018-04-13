use rocket::http::Status as HttpStatus;
use rocket::response::status::Custom;

use rocket_contrib::Json;

use serde;

use std::fmt::Debug;

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Status<T> {
  Success(T),
  Error {
    code: u64,
    message: String,
  },
}

impl<T> Status<T>
  where T: Debug + serde::Serialize
{
  pub fn success(t: T) -> Status<T> {
    Status::Success(t)
  }

  pub fn error<S: Into<String>>(code: u64, message: S) -> Status<T> {
    let message = message.into();
    Status::Error { code, message }
  }

  pub fn show(status: HttpStatus, t: T) -> Custom<Json<T>> {
    Custom(status, Json(t))
  }
}
