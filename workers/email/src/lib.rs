#[macro_use]
extern crate serde_derive;

use lettre::{
  ClientSecurity, ClientTlsParameters, SmtpClient, SmtpTransport, Transport,
  smtp::authentication::{Credentials, Mechanism},
};

use lettre_email::EmailBuilder;

use native_tls::TlsConnector;

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

  let email = EmailBuilder::new()
    .from((config.sender.from, config.sender.name))
    .to(email)
    .subject(subject)
    .html(content)
    .build();

  let email = match email {
    Ok(e) => e,
    Err(e) => {
      eprintln!("error creating email: {}", e);
      return;
    },
  };

  let connector = match TlsConnector::new() {
    Ok(c) => c,
    Err(e) => {
      eprintln!("could not create tls connector: {}", e);
      return;
    },
  };
  let security = ClientSecurity::Wrapper(
    ClientTlsParameters::new(
      config.smtp.domain,
      connector,
    ),
  );
  let addr = (config.smtp.address.as_str(), config.smtp.port);
  let client = match SmtpClient::new(addr, security) {
    Ok(c) => c
      .credentials(Credentials::new(config.smtp.username, config.smtp.password))
      .authentication_mechanism(Mechanism::Login),
    Err(e) => {
      eprintln!("could not create smtp transport builder: {}", e);
      return;
    },
  };
  let mut transport = SmtpTransport::new(client);
  if let Err(e) = transport.send(email.into()) {
    eprintln!("could not send email: {}", e);
  }
}
