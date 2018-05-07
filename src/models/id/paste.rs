use database::DbConn;
use database::models::files::{NewFile, File as DbFile};
use database::models::pastes::Paste as DbPaste;
use database::schema::{files, pastes};
use errors::*;
use models::paste::Content;
use store::Store;
use super::FileId;

use diesel;
use diesel::prelude::*;

use git2::{Repository, Signature, DiffOptions};

use uuid::Uuid;

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

uuid_wrapper!(
  /// An ID for a paste, which may or may not exist.
  ///
  /// Mostly useful for having Rocket accept only valid IDs in routes.
  PasteId
);

impl PasteId {
  pub fn directory(&self) -> PathBuf {
    Store::directory().join(self.0.simple().to_string())
  }

  pub fn files_directory(&self) -> PathBuf {
    self.directory().join("files")
  }

  pub fn repo_dirty(&self) -> Result<bool> {
    let repo = Repository::open(self.files_directory())?;
    let mut options = DiffOptions::new();
    options.ignore_submodules(true);
    let diff = repo.diff_index_to_workdir(None, Some(&mut options))?;
    Ok(diff.stats()?.files_changed() != 0)
  }

  pub fn commit_if_dirty(&self, username: &str, email: &str, message: &str) -> Result<()> {
    if self.repo_dirty()? {
      return self.commit(username, email, message);
    }

    Ok(())
  }

  pub fn commit(&self, username: &str, email: &str, message: &str) -> Result<()> {
    let repo = Repository::open(self.files_directory())?;

    let mut index = repo.index()?;

    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let parent = if repo.is_empty()? {
      None
    } else {
      let head_id = repo.refname_to_id("HEAD")?;
      Some(repo.find_commit(head_id)?)
    };
    let parents = parent.as_ref().map(|x| vec![x]).unwrap_or_default();

    let signature = Signature::now(username, email)?;

    repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &parents)?;

    Ok(())
  }

  pub fn create_file<S: AsRef<str>>(&self, conn: &DbConn, name: Option<S>, content: Content) -> Result<DbFile> {
    // generate file id
    let id = Uuid::new_v4();

    // check if content is binary for later
    let binary = content.is_binary();

    // create file on the system
    let file_path = self.files_directory().join(id.simple().to_string());
    let mut f = File::create(file_path)?;
    f.write_all(&content.into_bytes())?;

    let name = name
      .map(|s| s.as_ref().to_string()) // get a String
      .or_else(|| self.next_generic_name(conn).ok()) // try to get a generic name if no name specified
      .unwrap_or_else(|| id.simple().to_string()); // fall back to uuid if necessary

    // add file to the database
    let new_file = NewFile::new(id, **self, name, Some(binary), None);
    let db_file = diesel::insert_into(files::table).values(&new_file).get_result(&**conn)?;

    Ok(db_file)
  }

  pub fn len(&self, conn: &DbConn) -> Result<usize> {
    let size: i64 = files::table
      .filter(files::paste_id.eq(self.0))
      .select(diesel::dsl::count(files::id))
      .first(&**conn)?;

    Ok(size as usize)
  }

  pub fn is_empty(&self, conn: &DbConn) -> Result<bool> {
    Ok(self.len(conn)? == 0)
  }

  pub fn next_generic_name(&self, conn: &DbConn) -> Result<String> {
    Ok(format!("pastefile{}", self.len(conn)? + 1))
  }

  pub fn delete_file(&self, conn: &DbConn, id: FileId) -> Result<()> {
    diesel::delete(files::table.filter(files::id.eq(id))).execute(&**conn)?;
    fs::remove_file(self.files_directory().join(id.simple().to_string()))?;

    if self.is_empty(conn)? {
      self.delete(conn)?;
    }

    Ok(())
  }

  pub fn delete(&self, conn: &DbConn) -> Result<()> {
    // database will cascade and delete all files and deletion keys, as well
    diesel::delete(pastes::table.filter(pastes::id.eq(self.0))).execute(&**conn)?;
    // remove from system
    fs::remove_dir_all(self.directory())?;

    Ok(())
  }

  pub fn get(&self, conn: &DbConn) -> Result<Option<DbPaste>> {
    Ok(pastes::table.find(self.0).first(&**conn).optional()?)
  }

  pub fn files(&self, conn: &DbConn) -> Result<Vec<DbFile>> {
    Ok(files::table.filter(files::paste_id.eq(self.0)).load(&**conn)?)
  }

  pub fn file(&self, conn: &DbConn, id: Uuid) -> Result<Option<DbFile>> {
    Ok(files::table
      .find(id)
      .filter(files::paste_id.eq(self.0))
      .first(&**conn)
      .optional()?)
  }
}
