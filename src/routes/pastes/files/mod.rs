use database::models::files::File as DbFile;
use errors::*;
use models::id::PasteId;
use models::paste::Content;
use models::paste::output::OutputFile;

use std::fs::File;
use std::io::Read;

pub mod get;
pub mod file;

pub fn make_output_file(db_file: &DbFile, with_content: bool) -> Result<OutputFile> {
  let file_path = PasteId(db_file.paste_id()).files_directory().join(db_file.id().simple().to_string());

  let mut file = File::open(file_path)?;
  let mut data = Vec::new();
  file.read_to_end(&mut data)?;

  let content = match with_content {
    true => {
      if *db_file.is_binary() == Some(true) {
        Some(Content::Base64(data))
      } else {
        // FIXME: fall back to base64? this error shouldn't really be possible except for FS
        //        corruption
        Some(Content::Text(String::from_utf8(data)?))
      }
    },
    false => None,
  };

  Ok(OutputFile::new(&db_file.id(), Some(db_file.name().clone()), content))
}
