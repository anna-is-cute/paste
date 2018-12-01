use chrono::{DateTime, Utc};

use rocket::{
  http::{
    RawStr,
    uri::{Formatter, UriDisplay, UriPart},
  },
  request::FromFormValue,
};

use std::fmt;

#[derive(Debug)]
pub struct UrlDate(DateTime<Utc>);

impl UrlDate {
  pub fn into_inner(self) -> DateTime<Utc> {
    self.0
  }
}

impl<P> UriDisplay<P> for UrlDate
  where P: UriPart,
{
  fn fmt(&self, f: &mut Formatter<P>) -> fmt::Result {
    f.write_value(self.0.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true))
  }
}

impl FromFormValue<'v> for UrlDate {
  type Error = &'v RawStr;

  fn from_form_value(form_value: &'v RawStr) -> Result<UrlDate, Self::Error> {
    DateTime::parse_from_rfc3339(form_value.as_str())
      .map(|x| UrlDate(x.with_timezone(&Utc)))
      .map_err(|_| form_value)
  }
}

impl std::ops::Deref for UrlDate {
  type Target = DateTime<Utc>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<DateTime<Utc>> for UrlDate {
  fn from(d: DateTime<Utc>) -> Self {
    UrlDate(d)
  }
}
