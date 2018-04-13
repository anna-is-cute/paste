use errors::*;
use models::paste::{Paste, PasteFile, PasteId, Internal};

use git2::Repository;

use serde_json;

use uuid::Uuid;

use std::fs::{self, File};
use std::path::PathBuf;
use std::result;

pub struct Store;

impl Store {
  pub fn directory() -> PathBuf {
    PathBuf::from("store")
  }

  pub fn new_paste(paste: &Paste) -> Result<(PasteId, Internal)> {
    let id = PasteId(Uuid::new_v4());

    // get the path to the paste
    let paste_path = Store::directory().join(id.simple().to_string());

    // get the files path for the paste
    let files_path = paste_path.join("files");

    // create directory for the paste
    fs::create_dir_all(&paste_path)?;

    // make the files repo for the paste
    let repo = Repository::init(&files_path);

    // create a metadata file
    let meta_file = File::create(paste_path.join("metadata.json"))?;
    serde_json::to_writer(meta_file, &paste.metadata)?;

    // create internal metadata
    let mut internal = Internal::default();
    let mut count = 0;
    for file in &paste.files {
      let file_id = Uuid::new_v4();
      let file_name = file.name.clone().unwrap_or_else(|| {
        count += 1;
        format!("pastefile{}", count)
      });
      internal.names.push((file_id, file_name));
    }

    // create internal metadata file
    let internal_file = File::create(paste_path.join("internal.json"))?;
    serde_json::to_writer(internal_file, &internal)?;

    Ok((id, internal))
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
