use serde;

use std::fmt::Debug;

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum Status<T: Debug + serde::Serialize> {
  Success(T),
  Error(Error),
}

#[derive(Debug, Serialize)]
pub struct Error {
  code: u64,
  message: String,
}

impl Error {
  pub fn new<S: Into<String>>(code: u64, message: S) -> Error {
    let message = message.into();
    Error { code, message }
  }
}
