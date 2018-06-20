use failure::Error;

pub trait BackendError: Sized {
  fn into_message(self) -> Result<&'static str, Error>;

  fn into_web_message(self) -> Result<String, Error> {
    let mut msg = self.into_message()?.chars();

    let first = match msg.next() {
      Some(c) => c.to_uppercase(),
      None => return Ok(Default::default()),
    };

    Ok(format!("{}{}.", first, msg.collect::<String>()))
  }
}
