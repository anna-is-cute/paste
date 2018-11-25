use crypto::{
  digest::Digest,
  md5::Md5,
};

use tera::{Error, ErrorKind, Result, Value};

use std::{
  cell::RefCell,
  collections::HashMap,
};

thread_local! {
  static MD5: RefCell<Md5> = RefCell::new(Md5::new());
}

pub(crate) fn md5(value: Value, _args: HashMap<String, Value>) -> Result<Value> {
  let s = match value {
    Value::String(ref s) => s,
    _ => return Err(Error::from_kind(ErrorKind::Msg(format!(
      "Filter `md5` received an incorrect type for arg `value`: \
      got `{}` but expected String",
      value.to_string(),
    )))),
  };

  let hash = MD5.with(|m| {
    let mut m = m.borrow_mut();
    m.input_str(s.trim());
    let hash = m.result_str();
    m.reset();

    hash
  });

  Ok(Value::String(hash))
}
