use errors::*;
use models::paste::{Paste, Content, PasteId};
use models::paste::output::{Output, OutputFile};

use rocket_contrib::Json;

use std::fs::File;
use std::io::Read;

#[derive(Debug, Default, FromForm)]
struct Query {
  full: Option<bool>,
}

// routes separated because of https://github.com/SergioBenitez/Rocket/issues/376

#[get("/<id>")]
fn get(id: PasteId) -> Result<Json<Output>> {
  _get(id, None)
}

#[get("/<id>?<query>")]
fn get_query(id: PasteId, query: Query) -> Result<Json<Output>> {
  _get(id, Some(query))
}

fn _get(id: PasteId, query: Option<Query>) -> Result<Json<Output>> {
  let files_dir = id.files_directory();
  // FIXME: check if dir exists and return 404 instead of failing below and returning 500

  let metadata = id.metadata()?;
  let internal = id.internal()?;

  let query = query.unwrap_or_default();

  let mut files = Vec::with_capacity(internal.names.len());
  for (uuid, name) in &*internal.names {
    let file_path = files_dir.join(uuid.simple().to_string());
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    // TODO: store if the file is text or binary instead of attempting to parse
    let content = match query.full {
      Some(true) => Some(String::from_utf8(data.clone())
        .map(Content::Text)
        .unwrap_or_else(|_| Content::Base64(data))),
      _ => None,
    };

    let pf = OutputFile::new(uuid, Some(name.clone()), content);
    files.push(pf);
  }

  Ok(Json(Output {
    paste: Paste {
      metadata,
      files: Vec::new(),
    },
    files,
  }))
}
