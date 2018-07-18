use crate::{
  errors::*,
  models::id::{PasteId, UserId},
};

use git2::Repository;

use uuid::Uuid;

use std::{fs, path::PathBuf};

pub struct Store;

impl Store {
  pub fn directory() -> PathBuf {
    PathBuf::from("store")
  }

  pub fn new_paste(author: Option<UserId>) -> Result<PasteId> {
    let id = PasteId(Uuid::new_v4());

    let user_path = author.map(|x| x.simple().to_string()).unwrap_or_else(|| "anonymous".into());

    // get the path to the paste
    let paste_path = Store::directory().join(user_path).join(id.simple().to_string());

    // get the files path for the paste
    let files_path = paste_path.join("files");

    // create directory for the paste
    fs::create_dir_all(&paste_path)?;

    // make the files repo for the paste
    Repository::init(&files_path)?;

    Ok(id)
  }
}
