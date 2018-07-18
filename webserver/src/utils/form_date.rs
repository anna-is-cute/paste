use chrono::{DateTime, Utc};

use rocket::{request::FromFormValue, http::RawStr};

use std::{ops::Deref, str::FromStr};

#[derive(Debug, Serialize, Deserialize)]
pub struct FormDate(DateTime<Utc>);

impl FormDate {
  pub fn into_inner(self) -> DateTime<Utc> {
    self.0
  }
}

impl Deref for FormDate {
  type Target = DateTime<Utc>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl FromFormValue<'v> for FormDate {
  type Error = &'v RawStr;

  fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
    let string = String::from_form_value(form_value)?;

    DateTime::from_str(&string)
      .map(FormDate)
      .map_err(|_| form_value)
  }
}
