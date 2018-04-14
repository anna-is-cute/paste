use models::paste::{Paste, Content, PasteId};
use models::paste::output::{Output, OutputFile};
use models::status::{Status, ErrorKind};
use routes::RouteResult;

use rocket::http::Status as HttpStatus;

use std::fs::File;
use std::io::Read;

#[derive(Debug, Default, FromForm)]
struct Query {
  full: Option<bool>,
}

// routes separated because of https://github.com/SergioBenitez/Rocket/issues/376

#[get("/<id>")]
fn get(id: PasteId) -> RouteResult<Output> {
  _get(id, None)
}

#[get("/<id>?<query>")]
fn get_query(id: PasteId, query: Query) -> RouteResult<Output> {
  _get(id, Some(query))
}

fn _get(id: PasteId, query: Option<Query>) -> RouteResult<Output> {
  if !id.exists() {
    return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste));
  }
  let files_dir = id.files_directory();

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

  let output = Output {
    paste: Paste {
      metadata,
      files: Vec::new(),
    },
    files,
  };

  Ok(Status::show_success(HttpStatus::Ok, output))
}
