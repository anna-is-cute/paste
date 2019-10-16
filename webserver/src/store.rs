use crate::{
  config::Config,
  errors::*,
  models::id::{PasteId, UserId},
};

use git2::Repository;

use uuid::Uuid;

use std::{fs, path::PathBuf};

pub struct Store<'c> {
  config: &'c Config,
}

impl Store<'c> {
  pub fn new(config: &'c Config) -> Self {
    Store { config }
  }

  pub fn directory(&self) -> PathBuf {
    self.config.read().store.path.clone()
  }

  pub fn new_paste(&self, author: Option<UserId>) -> Result<PasteId> {
    let id = PasteId(Uuid::new_v4());

    let user_path = author.map(|x| x.to_simple().to_string()).unwrap_or_else(|| "anonymous".into());

    // get the path to the paste
    let paste_path = self.config.read().store.path.join(user_path).join(id.to_simple().to_string());

    // get the files path for the paste
    let files_path = paste_path.join("files");

    // create directory for the paste
    fs::create_dir_all(&paste_path)?;

    // make the files repo for the paste
    Repository::init(&files_path)?;

    Ok(id)
  }
}
