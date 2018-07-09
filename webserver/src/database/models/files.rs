#![cfg_attr(feature = "cargo-clippy", allow(option_option))]

use errors::*;
use models::id::{FileId, PasteId};
use models::paste::Content;
use models::paste::output::OutputFile;
use utils::Language;

use super::pastes::Paste;
use super::super::schema::files;

use chrono::{NaiveDateTime, Utc};

use std::fs::File as FsFile;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Identifiable, AsChangeset, Queryable, Associations)]
#[changeset_options(treat_none_as_null = "true")]
#[belongs_to(Paste)]
pub struct File {
  id: FileId,
  paste_id: PasteId,
  name: String,
  is_binary: Option<bool>,
  created_at: NaiveDateTime,
  highlight_language: Option<Language>,
}

impl File {
  pub fn id(&self) -> FileId {
    self.id
  }

  pub fn paste_id(&self) -> PasteId {
    self.paste_id
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn set_name(&mut self, name: String) {
    self.name = name;
  }

  pub fn highlight_language(&self) -> Option<Language> {
    self.highlight_language
  }

  pub fn set_highlight_language(&mut self, lang: Option<Language>) {
    self.highlight_language = lang;
  }

  pub fn is_binary(&self) -> Option<bool> {
    self.is_binary
  }

  pub fn created_at(&self) -> &NaiveDateTime {
    &self.created_at
  }

  pub fn as_output_file(&self, with_content: bool, paste: &Paste) -> Result<OutputFile> {
    let content = if with_content {
      Some(self.read_content(paste)?)
    } else {
      None
    };

    Ok(OutputFile::new(self.id(), Some(self.name()), self.highlight_language(), content))
  }

  pub fn path(&self, paste: &Paste) -> PathBuf {
    paste.files_directory().join(self.id().simple().to_string())
  }

  pub fn read_content(&self, paste: &Paste) -> Result<Content> {
    let mut file = FsFile::open(self.path(paste))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if self.is_binary() == Some(true) {
      Ok(Content::Base64(data))
    } else {
      // FIXME: fall back to base64? this error shouldn't really be possible except for FS
      //        corruption
      Ok(Content::Text(String::from_utf8(data)?))
    }
  }
}

#[derive(Insertable)]
#[table_name = "files"]
pub struct NewFile {
  id: FileId,
  paste_id: PasteId,
  name: String,
  is_binary: Option<bool>,
  created_at: NaiveDateTime,
  highlight_language: Option<Language>,
}

impl NewFile {
  pub fn new(id: FileId, paste_id: PasteId, name: String, is_binary: Option<bool>, highlight_language: Option<Language>, created_at: Option<NaiveDateTime>) -> Self {
    let created_at = created_at.unwrap_or_else(|| Utc::now().naive_utc());
    NewFile { id, paste_id, name, is_binary, created_at, highlight_language }
  }
}
