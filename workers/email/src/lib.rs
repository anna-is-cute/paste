extern crate lettre;
extern crate lettre_email;

use lettre::{FileEmailTransport, EmailTransport, SendableEmail};
use lettre_email::EmailBuilder;

use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub unsafe fn email(email: *const c_char, name: *const c_char, subject: *const c_char, content: *const c_char) {
  let email = CStr::from_ptr(email).to_string_lossy();
  let name = CStr::from_ptr(name).to_string_lossy();
  let subject = CStr::from_ptr(subject).to_string_lossy();
  let content = CStr::from_ptr(content).to_string_lossy();

  do_email(&email, &name, &subject, &content);
}

fn do_email(email: &str, name: &str, subject: &str, content: &str) {
  let email = EmailBuilder::new()
    .from(("no-reply@paste.gg", "paste.gg"))
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

  FileEmailTransport::new("emails").send(&email).unwrap();

  println!("sent email: {}!", email.message_id());
}
