macro_rules! otry {
  ($e:expr) => {{
    match $e {
      ::rocket::Outcome::Success(x) => x,
      ::rocket::Outcome::Failure(x) => return ::rocket::Outcome::Failure(x),
      ::rocket::Outcome::Forward(x) => return ::rocket::Outcome::Forward(x),
    }
  }}
}

pub mod anti_csrf_token;
pub mod session;

pub use self::anti_csrf_token::AntiCsrfToken;
pub use self::session::Session;
