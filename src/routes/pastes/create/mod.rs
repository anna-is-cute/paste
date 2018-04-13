use errors::*;
use models::paste::{Paste, Metadata, Content};
use models::status::Status;
use store::Store;

use base64;

use git2::Repository;

use rocket::response::status::Custom;
use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

use serde_json;

use uuid::Uuid;

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

mod output;

use self::output::Success;

#[post("/", format = "application/json", data = "<info>")]
fn create(info: ::std::result::Result<Json<Paste>, ::rocket_contrib::SerdeError>) -> Result<Custom<Json<Status<Success>>>> {
  // TODO: can this be a request guard?
  let info = match info {
    Ok(x) => x,
    Err(e) => {
      let message = format!("could not parse json: {}", e);
      return Ok(
        Status::show(HttpStatus::BadRequest, Status::error(2, message))
      )
    },
  };

  // check that files are valid
  // move validate_files to Paste?
  if let Err(e) = Store::validate_files(&info.files) {
    return Ok(
      Status::show(HttpStatus::BadRequest, Status::error(1, e))
    );
  }
  // move this to PasteId::create?
  // rocket has already verified the paste info is valid, so create a paste id
  let id = Store::new_paste(&info.metadata)?;

  // PasteId::write_files?
  // write the files
  for (i, pf) in info.into_inner().files.into_iter().enumerate() {
    let pf_path = files.join(pf.name.unwrap_or_else(|| format!("pastefile{}", i + 1)));

    let mut file = File::create(pf_path)?;
    let content = match pf.content {
      Content::Text(c) => c.into_bytes(),
      Content::Base64(b) => b,
      // FIXME: others
      _ => continue,
    };
    file.write_all(&content)?;
  }

  // TODO: commit

  // return success
  Ok(Status::show(HttpStatus::Ok, Status::success(paste_id.into())))
}
