use rocket::http::Status as HttpStatus;
use rocket::response::status::Custom;

use rocket_contrib::Json;

use serde;

use std::fmt::Debug;

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Status<T> {
  Success {
    result: T
  },
  Error(ErrorKind),
}

impl<T> Status<T>
  where T: Debug + serde::Serialize
{
  pub fn success(t: T) -> Status<T> {
    Status::Success { result: t }
  }

  pub fn error(error: ErrorKind) -> Status<T> {
    Status::Error(error)
  }

  pub fn show_error(status: HttpStatus, error: ErrorKind) -> Custom<Json<Status<T>>> {
    Custom(status, Json(Status::error(error)))
  }

  pub fn show_success(status: HttpStatus, t: T) -> Custom<Json<Status<T>>> {
    Custom(status, Json(Status::success(t)))
  }
}

#[derive(Debug, Serialize)]
#[serde(tag = "error", content = "message", rename_all = "snake_case")]
pub enum ErrorKind {
  InvalidFile(#[serde(skip_serializing_if = "Option::is_none")] Option<String>),
  BadJson(#[serde(skip_serializing_if = "Option::is_none")] Option<String>),
  MissingPaste,
  MissingFile,
  BadApiKey(#[serde(skip_serializing_if = "Option::is_none")] Option<String>),
}
