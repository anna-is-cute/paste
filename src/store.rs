use errors::*;
use models::paste::{Paste, PasteFile, PasteId};

use git2::Repository;

use uuid::Uuid;

use std::fs;
use std::path::PathBuf;
use std::result;

pub struct Store;

impl Store {
  pub fn directory() -> PathBuf {
    PathBuf::from("store")
  }

  pub fn new_paste() -> Result<PasteId> {
    let id = PasteId(Uuid::new_v4());

    // get the path to the paste
    let paste_path = Store::directory().join(id.simple().to_string());

    // get the files path for the paste
    let files_path = paste_path.join("files");

    // create directory for the paste
    fs::create_dir_all(&paste_path)?;

    // make the files repo for the paste
    Repository::init(&files_path)?;

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
