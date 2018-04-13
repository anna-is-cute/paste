use errors::*;
use models::paste::{Metadata, PasteFile, PasteId};

use git2::Repository;

use uuid::Uuid;

use std::fs::{self, File};
use std::path::PathBuf;
use std::result;

pub struct Store;

impl Store {
  pub fn directory() -> PathBuf {
    PathBuf::from("store")
  }

  pub fn new_paste(meta: &Metadata) -> Result<PasteId> {
    let id = PasteId(Uuid::new_v4());

    // get the path to the repo
    let repo_path = Store::directory().join(id.simple().to_string());

    // make the repo for the paste
    let repo = Repository::init(&repo_path);

    // create a metadata file
    let meta_file = File::create(repo_path.join("metadata.json"))?;
    serde_json::to_writer(meta_file, &info.metadata)?;

    // create the files directory
    let files = repo_path.join("files");
    fs::create_dir_all(&files)?;

    Ok(id)
  }

  pub fn validate_files(files: &[PasteFile]) -> result::Result<(), String> {
    let mut names: Vec<&String> = files.iter().filter_map(|x| x.name.as_ref()).collect();
    let len = names.len();
    names.sort();
    names.dedup();
    if len != names.len() {
      return Err("duplicate names".into());
    }

    if names.iter().any(|x| x.contains(|c| c == '\\' || c == '/')) {
      return Err("names contained path separators".into());
    }

    Ok(())
  }
}
