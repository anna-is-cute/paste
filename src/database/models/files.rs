use errors::*;
use models::id::PasteId;
use models::paste::Content;
use models::paste::output::OutputFile;

use super::pastes::Paste;
use super::super::schema::files;

use std::fs::File as FsFile;
use std::io::Read;

use uuid::Uuid;

#[derive(Identifiable, AsChangeset, Queryable, Associations)]
#[belongs_to(Paste)]
pub struct File {
  id: Uuid,
  paste_id: Uuid,
  name: String,
  is_binary: Option<bool>,
}

impl File {
  pub fn id(&self) -> Uuid {
    self.id
  }

  pub fn paste_id(&self) -> Uuid {
    self.paste_id
  }

  pub fn name(&self) -> &String {
    &self.name
  }

  pub fn set_name(&mut self, name: String) {
    self.name = name;
  }

  pub fn is_binary(&self) -> &Option<bool> {
    &self.is_binary
  }

  pub fn as_output_file(&self, with_content: bool) -> Result<OutputFile> {
    let file_path = PasteId(self.paste_id()).files_directory().join(self.id().simple().to_string());

    let mut file = FsFile::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let content = match with_content {
      true => {
        if *self.is_binary() == Some(true) {
          Some(Content::Base64(data))
        } else {
          // FIXME: fall back to base64? this error shouldn't really be possible except for FS
          //        corruption
          Some(Content::Text(String::from_utf8(data)?))
        }
      },
      false => None,
    };

    Ok(OutputFile::new(&self.id(), Some(self.name().clone()), content))
  }
}

#[derive(Insertable)]
#[table_name = "files"]
pub struct NewFile {
  id: Uuid,
  paste_id: Uuid,
  name: String,
  is_binary: Option<bool>,
}

impl NewFile {
  pub fn new(id: Uuid, paste_id: Uuid, name: String, is_binary: Option<bool>) -> Self {
    NewFile { id, paste_id, name, is_binary }
  }
}
