use unicode_categories::UnicodeCategories;

use unicode_segmentation::UnicodeSegmentation;

use std::borrow::Cow;

pub struct Validator<'a> {
  s: Cow<'a, str>,
}

impl Validator<'a> {
  fn bytes(&mut self) -> Result<(), &'static str> {
    const MAX_SIZE: usize = 5 * 1024;
    if self.s.len() > MAX_SIZE {
      return Err("must be no more than 5 KiB");
    }
    Ok(())
  }

  fn graphemes(&mut self) -> Result<(), &'static str> {
    let graphemes = self.s.graphemes(true).count();
    if graphemes < 1 {
      return Err("must be at least one character");
    }
    if graphemes > 255 {
      return Err("must be no more than 255 characters");
    }
    Ok(())
  }

  fn categories(&mut self, restrict_spaces: bool) -> Result<(), &'static str> {
    for c in self.s.chars() {
      if c.is_other() {
        return Err("must not contain any control characters");
      }
      if restrict_spaces && c != ' ' && c.is_separator_space() {
        return Err("must only contain normal spaces");
      }
    }
    Ok(())
  }

  fn reserved(&mut self) -> Result<(), &'static str> {
    if self.s == "anonymous" {
      return Err("cannot be \"anonymous\"");
    }
    if self.s == "." || self.s == ".." {
      return Err("cannot be \".\" or \"..\"");
    }
    Ok(())
  }

  fn repeating_space(&mut self) -> Result<(), &'static str> {
    let mut last_space = false;
    for &b in self.s.as_bytes() {
      let is_space = b == b' ';
      if last_space && is_space {
        return Err("cannot contain two or more adjacent spaces");
      }
      last_space = is_space;
    }
    Ok(())
  }

  fn all_ascii(&mut self) -> Result<(), &'static str> {
    if !self.s.is_ascii() {
      // FIXME: does the average user know what this means
      return Err("must be only ASCII characters");
    }
    Ok(())
  }

  fn one_byte(&mut self) -> Result<(), &'static str> {
    if self.s.is_empty() {
      return Err("must be at least one character long");
    }
    Ok(())
  }

  pub fn validate_display_name(s: &'a str) -> Result<Cow<'a, str>, &'static str> {
    let mut validator = Validator {
      s: Cow::Borrowed(s.trim()),
    };
    validator.bytes()?;
    validator.graphemes()?;
    validator.categories(false)?;

    Ok(validator.s)
  }

  pub fn validate_username(s: &'a str) -> Result<Cow<'a, str>, &'static str> {
    let mut validator = Validator {
      s: Cow::Borrowed(s.trim()),
    };
    validator.one_byte()?;
    validator.all_ascii()?;
    validator.reserved()?;
    validator.bytes()?;
    validator.categories(true)?;
    validator.repeating_space()?;

    Ok(validator.s)
  }
}
