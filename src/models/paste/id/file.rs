use uuid::Uuid;

use rocket::http::RawStr;
use rocket::request::FromParam;

use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct FileId(pub Uuid);

impl Display for FileId {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0.simple())
  }
}

// Allow FileId to be dereferenced into its inner type
impl Deref for FileId {
  type Target = Uuid;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

// Allow Rocket to accept FileId in routes
impl<'a> FromParam<'a> for FileId {
  type Error = &'a RawStr;

  fn from_param(param: &'a RawStr) -> Result<Self, &'a RawStr> {
    match Uuid::from_str(param) {
      Ok(u) => Ok(FileId(u)),
      Err(_) => Err(param)
    }
  }
}
