use rocket::request::FromFormValue;
use rocket::http::RawStr;

use sodiumoxide::crypto::pwhash;

use std::ops::Deref;

#[derive(Debug)]
pub struct HashedPassword(Vec<u8>);

impl<'v> FromFormValue<'v> for HashedPassword {
  type Error = &'v RawStr;

  fn from_form_value(form_value: &'v RawStr) -> Result<Self, Self::Error> {
    let string = String::from_form_value(form_value)?;
    Ok(string.into())
  }
}

impl HashedPassword {
  pub fn into_string(mut self) -> String {
    self.0.pop(); // remove the 0x00
    unsafe { String::from_utf8_unchecked(self.0) }
  }
}

impl<S> From<S> for HashedPassword
  where S: AsRef<[u8]>,
{
  fn from(s: S) -> Self {
    let hash = pwhash::pwhash(
      s.as_ref(),
      pwhash::OPSLIMIT_INTERACTIVE,
      pwhash::MEMLIMIT_INTERACTIVE
    ).expect("hashing with pwhash");
    HashedPassword(hash[..].to_vec())
  }
}

impl Deref for HashedPassword {
  type Target = [u8];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
