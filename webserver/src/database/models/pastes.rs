use crate::{
  database::DbConn,
  errors::*,
  models::{
    id::{FileId, PasteId, UserId},
    paste::{
      Content, Visibility,
      update::{MetadataUpdate, Update},
    },
    status::ErrorKind,
  },
  sidekiq::Job,
  store::Store,
  utils::Language,
};

use super::files::{File as DbFile, NewFile};
use super::super::schema::{pastes, files};
use super::users::User;

use chrono::{NaiveDateTime, DateTime, Utc};

use diesel::prelude::*;

use git2::{Signature, Repository, IndexAddOption, Status};

use rocket::http::Status as HttpStatus;

use sidekiq::{Client as SidekiqClient, Value};

use uuid::Uuid;

use std::{
  fs::{self, File},
  io::Write,
  path::PathBuf,
};

#[derive(Debug, Identifiable, AsChangeset, Queryable, Associations)]
#[changeset_options(treat_none_as_null = "true")]
#[belongs_to(User, foreign_key = "author_id")]
pub struct Paste {
  id: PasteId,
  name: Option<String>,
  visibility: Visibility,
  author_id: Option<UserId>,
  description: Option<String>,
  created_at: NaiveDateTime,
  expires: Option<NaiveDateTime>,
}

impl Paste {
  pub fn id(&self) -> PasteId {
    self.id
  }

  pub fn name(&self) -> Option<&str> {
    self.name.as_ref().map(|x| x.as_str())
  }

  pub fn set_name<S: AsRef<str>>(&mut self, name: Option<S>) {
    self.name = name.map(|x| x.as_ref().to_string());
  }

  pub fn visibility(&self) -> Visibility {
    self.visibility
  }

  pub fn set_visibility(&mut self, visibility: Visibility) {
    self.visibility = visibility;
  }

  pub fn author_id(&self) -> Option<UserId> {
    self.author_id
  }

  pub fn description(&self) -> Option<&str> {
    self.description.as_ref().map(|x| x.as_str())
  }

  pub fn set_description<S: AsRef<str>>(&mut self, description: Option<S>) {
    self.description = description.map(|x| x.as_ref().to_string());
  }

  pub fn created_at(&self) -> DateTime<Utc> {
    DateTime::from_utc(self.created_at, Utc)
  }

  pub fn expires(&self) -> Option<DateTime<Utc>> {
    self.expires.map(|x| DateTime::from_utc(x, Utc))
  }

  pub fn set_expires(&mut self, expires: Option<DateTime<Utc>>) {
    self.expires = expires.map(|x| x.naive_utc());
  }

  pub fn update(&mut self, conn: &DbConn, sidekiq: &SidekiqClient, update: &MetadataUpdate) -> Result<()> {
    let changed = !update.name.is_ignore()
      || update.visibility.is_some()
      || !update.description.is_ignore()
      || !update.expires.is_ignore();
    if !changed {
      return Ok(());
    }
    match update.name {
      Update::Set(ref s) => self.set_name(Some(s.clone().into_inner())),
      Update::Remove => self.set_name(None::<String>),
      _ => {},
    }
    match update.description {
      Update::Set(ref s) => self.set_description(Some(s.clone().into_inner())),
      Update::Remove => self.set_description(None::<String>),
      _ => {},
    }
    if let Some(ref update) = update.visibility {
      self.set_visibility(*update);
    }
    match update.expires {
      Update::Set(ref s) => {
        self.set_expires(Some(*s));

        let timestamp = s.timestamp();

        let user = match self.author_id() {
          Some(a) => a.simple().to_string(),
          None => "anonymous".to_string(),
        };

        let job = Job::queue("ExpirePaste", timestamp, vec![
          Value::Number(timestamp.into()),
          Value::String(Store::directory().to_string_lossy().to_string()),
          Value::String(user),
          Value::String(self.id().simple().to_string()),
        ]);
        sidekiq.push(job.into())?;
      },
      Update::Remove => self.set_expires(None),
      _ => {},
    }
    diesel::update(pastes::table)
      .filter(pastes::id.eq(self.id))
      .set(&*self)
      .execute(&**conn)?;

    Ok(())
  }

  pub fn check_access<U>(&self, user: U) -> Option<(HttpStatus, ErrorKind)>
    where U: Into<Option<UserId>>,
  {
    let user = user.into();
    let is_private = self.visibility == Visibility::Private;
    if self.author_id.is_none() || !is_private || self.author_id == user {
      return None;
    }
    if is_private {
      Some((HttpStatus::NotFound, ErrorKind::MissingPaste))
    } else {
      Some((HttpStatus::Forbidden, ErrorKind::NotAllowed))
    }
  }

  pub fn directory(&self) -> PathBuf {
    let author = self.author_id().map(|x| x.simple().to_string()).unwrap_or_else(|| "anonymous".into());
    Store::directory().join(author).join(self.id().simple().to_string())
  }

  pub fn files_directory(&self) -> PathBuf {
    self.directory().join("files")
  }

  pub fn repo_dirty(&self) -> Result<bool> {
    let repo = Repository::open(self.files_directory())?;
    let dirty = repo
      .statuses(None)?
      .iter()
      .any(|x| x.status() != Status::CURRENT && x.status() != Status::IGNORED);
    Ok(dirty)
  }

  pub fn commit_if_dirty(&self, username: &str, email: &str, message: &str) -> Result<()> {
    if self.repo_dirty()? {
      return self.commit(username, email, message);
    }

    Ok(())
  }

  pub fn commit(&self, username: &str, email: &str, message: &str) -> Result<()> {
    let repo = self.repository()?;
    let mut index = repo.index()?;

    index.add_all(vec!["."], IndexAddOption::DEFAULT, None)?;
    index.write()?;

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

  pub fn repository(&self) -> Result<Repository> {
    let repo = Repository::open(self.files_directory())?;

    Ok(repo)
  }

  pub fn num_commits(&self) -> Result<usize> {
    let repo = self.repository()?;
    let head = repo.refname_to_id("HEAD")?;
    let head_commit = repo.find_commit(head)?;

    let mut count = 1;

    let mut commit = head_commit;
    while let Ok(p) = commit.parent(0) {
      commit = p;
      count += 1;
    }

    Ok(count)
  }

  pub fn create_file<S: AsRef<str>>(&self, conn: &DbConn, name: Option<S>, lang: Option<Language>, content: Content) -> Result<DbFile> {
    // generate file id
    let id = FileId(Uuid::new_v4());

    // check if content is binary for later
    let binary = content.is_binary();

    // create file on the system
    let file_path = self.files_directory().join(id.simple().to_string());
    let mut f = File::create(file_path)?;
    f.write_all(&content.into_bytes())?;

    let name = name
      .map(|s| s.as_ref().to_string()) // get a String
      .or_else(|| self.id().next_generic_name(conn).ok()) // try to get a generic name if no name specified
      .unwrap_or_else(|| id.simple().to_string()); // fall back to uuid if necessary

    // add file to the database
    let new_file = NewFile::new(id, self.id(), name, Some(binary), lang, None);
    let db_file = diesel::insert_into(files::table).values(&new_file).get_result(&**conn)?;

    Ok(db_file)
  }

  pub fn delete_file(&self, conn: &DbConn, id: FileId) -> Result<()> {
    diesel::delete(files::table.filter(files::id.eq(id))).execute(&**conn)?;
    fs::remove_file(self.files_directory().join(id.simple().to_string()))?;

    if self.id().is_empty(conn)? {
      self.delete(conn)?;
    }

    Ok(())
  }

  pub fn delete(&self, conn: &DbConn) -> Result<()> {
    // database will cascade and delete all files and deletion keys, as well
    diesel::delete(pastes::table.filter(pastes::id.eq(self.id()))).execute(&**conn)?;
    // remove from system
    fs::remove_dir_all(self.directory())?;

    Ok(())
  }
}

#[derive(Insertable)]
#[table_name = "pastes"]
pub struct NewPaste {
  id: PasteId,
  name: Option<String>,
  visibility: Visibility,
  author_id: Option<UserId>,
  description: Option<String>,
  created_at: NaiveDateTime,
  expires: Option<NaiveDateTime>,
}

impl NewPaste {
  pub fn new(
    id: PasteId,
    name: Option<String>,
    description: Option<String>,
    visibility: Visibility,
    author_id: Option<UserId>,
    created_at: Option<NaiveDateTime>,
    expires: Option<NaiveDateTime>,
  ) -> Self {
    let created_at = created_at.unwrap_or_else(|| Utc::now().naive_utc());
    NewPaste { id, name, visibility, author_id, description, created_at, expires }
  }
}
