use errors::*;
use models::paste::{Paste, PasteFile, Content, PasteId};

use rocket_contrib::Json;

use std::fs::File;
use std::io::Read;

#[get("/<id>")]
fn get(id: PasteId) -> Result<Json<Paste>> {
  let files_dir = id.files_directory();
  // FIXME: check if dir exists and return 404 instead of failing below and returning 500

  let metadata = id.metadata()?;
  let internal = id.internal()?;

  let mut files = Vec::with_capacity(internal.names.len());
  for (uuid, name) in &*internal.names {
    let file_path = files_dir.join(uuid.simple().to_string());
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let content = String::from_utf8(data.clone())
      .map(Content::Text)
      .unwrap_or_else(|_| Content::Base64(data));

    let pf = PasteFile {
      name: Some(name.clone()),
      content,
    };
    files.push(pf);
  }

  Ok(Json(Paste {
    metadata,
    files,
  }))
}
