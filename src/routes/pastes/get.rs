use database::DbConn;
use models::paste::{Paste, Content, Metadata, PasteId};
use models::paste::output::{Output, OutputFile};
use models::status::{Status, ErrorKind};
use routes::{RouteResult, OptionalUser};

use rocket::http::Status as HttpStatus;

use std::fs::File;
use std::io::Read;

#[derive(Debug, Default, FromForm)]
struct Query {
  full: Option<bool>,
}

// routes separated because of https://github.com/SergioBenitez/Rocket/issues/376

#[get("/<id>")]
fn get(id: PasteId, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  _get(id, None, user, conn)
}

#[get("/<id>?<query>")]
fn get_query(id: PasteId, query: Query, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  _get(id, Some(query), user, conn)
}

fn _get(id: PasteId, query: Option<Query>, user: OptionalUser, conn: DbConn) -> RouteResult<Output> {
  let paste = match id.get(&conn)? {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Status::show_error(status, kind));
  }

  let db_files = id.files(&conn)?;

  let files_dir = id.files_directory();

  let query = query.unwrap_or_default();

  let mut files = Vec::with_capacity(db_files.len());
  for db_file in db_files {
    let file_path = files_dir.join(db_file.id().simple().to_string());
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    // TODO: store if the file is text or binary instead of attempting to parse
    let content = match query.full {
      Some(true) => {
        if *db_file.is_binary() == Some(true) {
          Some(Content::Base64(data))
        } else {
          // FIXME: fall back to base64? this error shouldn't really be possible except for FS
          //        corruption
          Some(Content::Text(String::from_utf8(data)?))
        }
      },
      _ => None,
    };

    let pf = OutputFile::new(&db_file.id(), Some(db_file.name().clone()), content);
    files.push(pf);
  }

  let output = Output {
    paste: Paste {
      metadata: Metadata {
        name: paste.name().clone(),
        visibility: paste.visibility(),
      },
      files: Vec::new(),
    },
    files,
  };

  Ok(Status::show_success(HttpStatus::Ok, output))
}
