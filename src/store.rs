use errors::*;
use models::id::PasteId;
use models::paste::PasteFile;

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
    let mut names: Vec<String> = files
      .iter()
      .enumerate()
      .map(|(i, x)| match x.name {
        None => format!("pastefile{}", i + 1),
        Some(ref x) => x.to_string(),
      })
      .collect();
    let len = names.len();
    names.sort();
    names.dedup();
    if len != names.len() {
      return Err("duplicate names".into());
    }

    if names.iter().any(|x| x.is_empty()) {
      return Err("names cannot be empty (for no name, omit the name field)".into());
    }

    Ok(())
  }
}
