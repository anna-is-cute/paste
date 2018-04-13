use errors::*;
use models::paste::{Paste, Content};
use models::status::Status;
use store::Store;

use rocket::response::status::Custom;
use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

use std::fs::File;
use std::io::Write;

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
  let (id, internal) = Store::new_paste(&*info)?;

  let files = id.files_directory();

  // PasteId::write_files?
  // write the files
  for (i, (pf, map)) in info.into_inner().files.into_iter().zip(&*internal.names).enumerate() {
    let pf_path = files.join(map.0.simple().to_string());

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
  Ok(Status::show(HttpStatus::Ok, Status::success(Success::from(*id))))
}
