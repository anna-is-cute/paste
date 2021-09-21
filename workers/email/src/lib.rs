#[macro_use]
extern crate serde_derive;

use lettre::{
  SmtpTransport, Transport, Message,
  address::Address,
  message::{Mailbox, SinglePart},
  transport::smtp::{
    client::{Tls, TlsParameters},
    authentication::{Credentials, Mechanism}
  },
};

use std::{
  ffi::CStr,
  os::raw::c_char,
};

mod config;

#[no_mangle]
pub unsafe fn email(path: *const c_char, email: *const c_char, subject: *const c_char, content: *const c_char) {
  let path = CStr::from_ptr(path).to_string_lossy();
  let email = CStr::from_ptr(email).to_string_lossy();
  let subject = CStr::from_ptr(subject).to_string_lossy();
  let content = CStr::from_ptr(content).to_string_lossy();

  do_email(&path, &email, &subject, &content);
}

fn do_email(path: &str, email: &str, subject: &str, content: &str) {
  let config = match config::config(path) {
    Ok(c) => c.email,
    Err(e) => {
      eprintln!("could not deserialize config: {:?}", e);
      return;
    },
  };

  let email: Address = match email.parse() {
    Ok(a) => a,
    Err(e) => {
      eprintln!("invalid email address: {}", e);
      return;
    },
  };

  let sender_addr: Address = match config.sender.from.parse() {
    Ok(a) => a,
    Err(e) => {
      eprintln!("invalid email address: {}", e);
      return;
    },
  };
  let sender = Mailbox::new(Some(config.sender.name), sender_addr);
  let email = Message::builder()
    .from(sender)
    .to(Mailbox::new(None, email))
    .subject(subject)
    .singlepart(SinglePart::html(content.to_string()));

  let email = match email {
    Ok(e) => e,
    Err(e) => {
      eprintln!("error creating email: {}", e);
      return;
    },
  };

  let params = match TlsParameters::new(config.smtp.domain) {
    Ok(p) => p,
    Err(e) => {
      eprintln!("could not create tls parameters: {}", e);
      return;
    },
  };

  let transport = SmtpTransport::builder_dangerous(config.smtp.address.as_str())
    .port(config.smtp.port)
    .tls(Tls::Wrapper(params))
    .credentials(Credentials::new(config.smtp.username, config.smtp.password))
    .authentication(vec![Mechanism::Login])
    .build();
  if let Err(e) = transport.send(&email) {
    eprintln!("could not send email: {}", e);
  }
}
