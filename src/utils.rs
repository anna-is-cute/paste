use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use uuid::Uuid;

use std::fmt::{self, Display, Formatter};
use std::ops::Deref;

#[derive(Debug)]
pub struct SimpleUuid(Uuid);

impl SimpleUuid {
  pub fn into_inner(self) -> Uuid {
    self.0
  }
}

impl From<Uuid> for SimpleUuid {
  fn from(u: Uuid) -> Self {
    SimpleUuid(u)
  }
}

impl Deref for SimpleUuid {
  type Target = Uuid;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Display for SimpleUuid {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    Display::fmt(&self.simple(), f)
  }
}

impl Serialize for SimpleUuid {
  fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
  {
    self.0.simple().to_string().serialize(ser)
  }
}

impl<'de> Deserialize<'de> for SimpleUuid {
  fn deserialize<D>(des: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
  {
    Ok(SimpleUuid(Uuid::deserialize(des)?))
  }
}
