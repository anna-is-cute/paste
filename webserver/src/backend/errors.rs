use anyhow::Error;

use std::borrow::Cow;

pub trait BackendError: Sized {
  fn into_message(self) -> Result<Cow<'static, str>, Error>;

  fn into_web_message(self) -> Result<String, Error> {
    let msg = self.into_message()?;
    let mut chars = msg.chars();

    let first = match chars.next() {
      Some(c) => c.to_uppercase(),
      None => return Ok(Default::default()),
    };

    Ok(format!("{}{}.", first, chars.collect::<String>()))
  }
}
