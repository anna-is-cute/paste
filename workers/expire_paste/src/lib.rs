extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate failure;
extern crate uuid;

use std::ffi::CStr;
use std::fs;
use std::os::raw::c_char;
use std::path::Path;
use std::str::FromStr;

use diesel::prelude::*;
use diesel::pg::PgConnection;

use failure::Error;

use uuid::Uuid;

mod paste;
mod schema;

use paste::Paste;

type Result<T> = std::result::Result<T, Error>;

#[no_mangle]
pub unsafe fn expire_paste(timestamp: i64, store_path: *const c_char, user_id: *const c_char, paste_id: *const c_char) {
  let store_path = CStr::from_ptr(store_path).to_string_lossy();
  let user_id = CStr::from_ptr(user_id).to_string_lossy();
  let paste_id = CStr::from_ptr(paste_id).to_string_lossy();

  let paste_id = match Uuid::from_str(&paste_id) {
    Ok(u) => u,
    Err(e) => {
      eprintln!("could not parse uuid {}: {}", paste_id, e);
      return;
    }
  };

  expire(timestamp, &store_path, &user_id, paste_id);
}

fn expire(timestamp: i64, store_path: &str, user_id: &str, paste_id: Uuid) {
  dotenv::dotenv().ok();

  let conn = match connection() {
    Ok(c) => c,
    Err(e) => {
      eprintln!("could not establish connection to database: {}", e);
      return;
    },
  };

  let paste: Paste = match schema::pastes::table.find(paste_id).get_result(&conn) {
    Ok(p) => p,
    Err(e) => {
      eprintln!("could not find paste {}: {}", paste_id, e);
      return;
    },
  };

  let expiration_date = match paste.expires {
    Some(e) => e,
    None => return,
  };

  if expiration_date.timestamp() != timestamp {
    return;
  }

  if let Err(e) = diesel::delete(&paste).execute(&conn) {
    eprintln!("could not delete paste {}: {}", paste_id, e);
  }

  let path = Path::new(store_path)
    .join(user_id)
    .join(paste_id.simple().to_string());
  if let Err(e) = fs::remove_dir_all(path) {
    eprintln!("could not delete paste {}: {}", paste_id, e);
  }
}

fn connection() -> Result<PgConnection> {
  let url = std::env::var("DATABASE_URL")?;
  let conn = PgConnection::establish(&url)?;

  Ok(conn)
}
