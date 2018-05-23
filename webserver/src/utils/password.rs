use unicode_segmentation::UnicodeSegmentation;

pub struct PasswordContext<'a> {
  password: &'a str,
  password_verify: &'a str,
  name: &'a str,
  username: &'a str,
  email: &'a str,
}

impl<'a> PasswordContext<'a> {
  pub fn new(password: &'a str, password_verify: &'a str, name: &'a str, username: &'a str, email: &'a str) -> Self {
    PasswordContext {
      password,
      password_verify,
      name,
      username,
      email,
    }
  }

  pub fn validate(&self) -> Result<(), &'static str> {
    if self.password.graphemes(true).count() < 10 {
      return Err("Password must be at least 10 characters long.");
    }

    if self.password != self.password_verify {
      return Err("Passwords did not match.")
    }

    if self.password == "password" {
      return Err(r#"Password cannot be "password"."#);
    }

    if self.password == self.name || self.password == self.username || self.password == self.email {
      return Err("Password cannot be the same as your name, username, or email.");
    }

    Ok(())
  }
}
