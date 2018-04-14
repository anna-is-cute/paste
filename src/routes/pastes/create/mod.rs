use models::paste::{Paste, Content};
use models::status::{Status, ErrorKind};
use routes::RouteResult;
use store::Store;

use git2::{Repository, Signature};

use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

use std::fs::File;
use std::io::Write;

mod output;

use self::output::Success;

#[post("/", format = "application/json", data = "<info>")]
fn create(info: ::std::result::Result<Json<Paste>, ::rocket_contrib::SerdeError>) -> RouteResult<Success> {
  // TODO: can this be a request guard?
  let info = match info {
    Ok(x) => x,
    Err(e) => {
      let message = format!("could not parse json: {}", e);
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some(message))));
    },
  };

  // check that files are valid
  // move validate_files to Paste?
  if let Err(e) = Store::validate_files(&info.files) {
    return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::InvalidFile(Some(e))));
  }
  // move this to PasteId::create?
  // rocket has already verified the paste info is valid, so create a paste
  let (id, internal) = Store::new_paste(&*info)?;

  let files = id.files_directory();

  // PasteId::write_files?
  // write the files
  for (pf, map) in info.into_inner().files.into_iter().zip(&*internal.names) {
    let pf_path = files.join(map.0.simple().to_string());

    let mut file = File::create(pf_path)?;
    let content = match pf.content {
      Content::Text(c) => c.into_bytes(),
      // all base64/compress/decompress is handled via serde
      Content::Base64(b) | Content::Gzip(b) | Content::Xz(b) => b,
    };
    file.write_all(&content)?;
  }

  // commit initial state
  let repo = Repository::open(&files)?;
  // TODO: change this for authed via api key
  let sig = Signature::now("No one", "no-one@example.com")?;
  let mut index = repo.index()?;
  let tree_id = index.write_tree()?;
  let tree = repo.find_tree(tree_id)?;
  repo.commit(Some("HEAD"), &sig, &sig, "create paste", &tree, &[])?;

  // return success
  Ok(Status::show_success(HttpStatus::Ok, Success::from(*id)))
}
