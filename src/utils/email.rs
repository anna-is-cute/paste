/// Check if an email is valid.
///
/// This is a slightly-modified port of Firefox's input validation state machine.
///
/// This function explicitly allows single-character domain names, since some do exist.
pub fn check_email(email: &str) -> bool {
  const ALLOWED: &[u8] = &[
    b'.', b'!', b'#', b'$', b'%', b'&', b'\'', b'*', b'+', b'-',
    b'/', b'=', b'?', b'^', b'_', b'`', b'{', b'|', b'}', b'~',
  ];

  let email = email.as_bytes();

  let length = email.len();

  // If the email address is empty, begins with a '@' or ends with a '.',
  // we know it's invalid.
  if length == 0 || email[0] == b'@' || email[length - 1] == b'.' {
    return false;
  }

  let mut i = 0;

  // Parsing the username.
  for &c in email {
    i += 1;

    if c == b'@' {
      break;
    }

    // The username characters have to be in this list to be valid.
    if !(c.is_ascii_alphanumeric() || ALLOWED.contains(&c)) {
      return false;
    }
  }

  // There is no domain name, that's not a valid email address.
  if i >= length {
    return false;
  }

  // The domain name can't begin with a dot.
  if email[i] == b'.' {
    return false;
  }

  let mut last = None;
  // Parsing the domain name.
  for &c in &email[i..] {
    if c == b'.' {
      // A dot can't follow a dot.
      if last == Some(b'.') {
        return false;
      }
    } else if !(c.is_ascii_alphanumeric() || c == b'-') {
      // The domain characters have to be in this list to be valid.
      return false;
    }

    last = Some(c);
  }

  true
}
