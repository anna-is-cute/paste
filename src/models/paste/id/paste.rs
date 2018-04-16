use database::DbConn;
use database::models::files::NewFile;
use database::schema::{files, pastes};
use errors::*;
use models::paste::Content;
use store::Store;

use diesel;
use diesel::prelude::*;

use git2::{Repository, Signature, DiffOptions};

use uuid::Uuid;

use rocket::http::RawStr;
use rocket::request::FromParam;

use std::fmt::{self, Display, Formatter};
use std::fs::{self, File};
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::result;
use std::str::FromStr;

/// An ID for a paste, which may or may not exist.
///
/// Mostly useful for having Rocket accept only valid IDs in routes.
#[derive(Debug, Clone, Copy)]
pub struct PasteId(pub Uuid);

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

  pub fn create_file<S: AsRef<str>>(&self, conn: &DbConn, paste: PasteId, name: Option<S>, content: Content) -> Result<Uuid> {
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
    let new_file = NewFile::new(id, *paste, name, Some(binary));
    diesel::insert_into(files::table).values(&new_file).execute(&**conn)?;

    Ok(id)
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

  pub fn delete_file(&self, conn: &DbConn, id: Uuid) -> Result<()> {
    diesel::delete(files::table.filter(files::id.eq(id))).execute(&**conn)?;
    fs::remove_dir_all(self.files_directory().join(id.simple().to_string()))?;

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
}

impl Display for PasteId {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0.simple())
  }
}

// Allow PasteId to be dereferenced into its inner type
impl Deref for PasteId {
  type Target = Uuid;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

// Allow Rocket to accept PasteId in routes
impl<'a> FromParam<'a> for PasteId {
  type Error = &'a RawStr;

  fn from_param(param: &'a RawStr) -> result::Result<Self, &'a RawStr> {
    match Uuid::from_str(param) {
      Ok(u) => Ok(PasteId(u)),
      Err(_) => Err(param)
    }
  }
}
