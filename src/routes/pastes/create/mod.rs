use errors::*;
use models::paste::{Paste, Metadata, Content};
use models::status::{Status, Error};

use base64;

use git2::Repository;

use rocket_contrib::Json;

use serde_json;

use uuid::Uuid;

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

mod output;

use self::output::Success;

#[post("/", format = "application/json", data = "<info>")]
fn create(info: Json<Paste>) -> Result<Json<Status<Success>>> {
  // check for repeat file names
  // TODO: check for path separator
  {
    let mut names: Vec<&String> = info.files.iter().filter_map(|x| x.name.as_ref()).collect();
    let names_len = names.len();
    names.sort();
    names.dedup();
    if names.len() != names_len {
      return Ok(Json(Status::Error(Error::new(1, "duplicate file names"))));
    }
  }

  // rocket has already verified the paste info is valid, so create a paste id
  let paste_id = Uuid::new_v4();

  // get the path to the repo
  let repo_path = Path::new("repos").join(paste_id.simple().to_string());

  // FIXME: refactor this out into centralized paste handling
  // make the repo for the paste
  let repo = Repository::init(&repo_path);

  // create a metadata file
  let meta_file = File::create(repo_path.join("metadata.json"))?;
  serde_json::to_writer(meta_file, &info.metadata)?;

  // create the files directory
  let files = repo_path.join("files");
  fs::create_dir_all(&files)?;

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
  Ok(Json(Status::Success(paste_id.into())))
}
