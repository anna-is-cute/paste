extern crate lettre;
extern crate lettre_email;
extern crate native_tls;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;

use lettre::{SmtpTransport, EmailTransport, ClientSecurity, ClientTlsParameters};
use lettre::smtp::authentication::{Credentials, Mechanism};

use lettre_email::EmailBuilder;

use native_tls::TlsConnector;

use std::ffi::CStr;
use std::os::raw::c_char;

mod config;

#[no_mangle]
pub unsafe fn email(path: *const c_char, email: *const c_char, name: *const c_char, subject: *const c_char, content: *const c_char) {
  let path = CStr::from_ptr(path).to_string_lossy();
  let email = CStr::from_ptr(email).to_string_lossy();
  let name = CStr::from_ptr(name).to_string_lossy();
  let subject = CStr::from_ptr(subject).to_string_lossy();
  let content = CStr::from_ptr(content).to_string_lossy();

  do_email(&path, &email, &name, &subject, &content);
}

fn do_email(path: &str, email: &str, name: &str, subject: &str, content: &str) {
  let config = match config::config(path) {
    Ok(c) => c.email,
    Err(e) => {
      eprintln!("could not deserialize config: {:?}", e);
      return;
    },
  };

  let email = EmailBuilder::new()
    .from((config.sender.from, config.sender.name))
    .to((email, name))
    .subject(subject)
    .text(content)
    .build();

  let email = match email {
    Ok(e) => e,
    Err(e) => {
      eprintln!("error creating email: {}", e);
      return;
    },
  };

  let connector = match TlsConnector::builder().and_then(|x| x.build()) {
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
  let builder = match SmtpTransport::builder(addr, security) {
    Ok(b) => b,
    Err(e) => {
      eprintln!("could not create smtp transport builder: {}", e);
      return;
    },
  };
  let mut transport = builder
    .credentials(Credentials::new(config.smtp.username, config.smtp.password))
    .authentication_mechanism(Mechanism::Login)
    .build();
  if let Err(e) = transport.send(&email) {
    eprintln!("could not send email: {}", e);
  }
}
