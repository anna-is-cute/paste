use crate::{
  database::{
    DbConn,
    schema::files,
  },
  models::{
    id::{FileId, PasteId},
    paste::update::{PasteFileUpdate, Update},
    status::{Status, ErrorKind},
  },
  routes::{RouteResult, RequiredUser},
};

use diesel::prelude::*;

use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

use std::{fs::OpenOptions, io::Write};

type UpdateResult = ::std::result::Result<Json<Vec<PasteFileUpdate>>, ::rocket_contrib::SerdeError>;

#[patch("/<paste_id>/files", format = "application/json", data = "<info>")]
pub fn patch(paste_id: PasteId, info: UpdateResult, user: RequiredUser, conn: DbConn) -> RouteResult<()> {
  // TODO: can this be a request guard?
  let mut info = match info {
    Ok(x) => x.into_inner(),
    Err(e) => {
      let message = format!("could not parse json: {}", e);
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some(message))));
    },
  };

  if info.is_empty() {
    return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some("array cannot be empty".into()))));
  }

  // sort the updates by content, which will put file removals last
  info.sort_by(|a, b| a.content.cmp(&b.content));

  // verify auth
  let paste = match paste_id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };
  if let Some((status, kind)) = paste.check_access(Some(user.id())) {
    return Ok(Status::show_error(status, kind));
  }

  let mut db_changed = false;
  // TODO: this needs much refactor love
  // update files and database if necessary
  let files_directory = paste.files_directory();

  let mut db_files = paste_id.files(&conn)?;
  {
    let db_files_ids: Vec<FileId> = db_files.iter().map(|x| x.id()).collect();
    let db_files_names: Vec<&str> = db_files.iter().map(|x| x.name()).collect();
    // verify all files before making changes
    if info.iter().filter_map(|x| x.id).any(|x| !db_files_ids.contains(&x)) {
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::MissingFile));
    }
    if info.iter().filter_map(|x| x.name.as_ref()).any(|x| db_files_names.contains(&x.as_str())) {
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::InvalidFile(Some("duplicate file name".into()))));
    }
    if info.iter().any(|x| x.id.is_none() && !x.content.is_set()) {
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::InvalidFile(Some("new files must have content".into()))));
    }
  }

  for file in info {
    match file.id {
      // updating existing file
      Some(id) => {
        // file should be present due to check above
        let db_file = db_files.iter_mut().find(|x| x.id() == id).expect("missing file");
        if let Some(name) = file.name {
          db_file.set_name(name);
          db_changed = true;
        }
        match file.content {
          // replacing contents
          Update::Set(content) => {
            let mut f = OpenOptions::new()
              .write(true)
              .truncate(true)
              .open(files_directory.join(db_file.id().to_simple().to_string()))?;
            f.write_all(&content.into_bytes())?;
            // FIXME: set is_binary field
          },
          // deleting file
          Update::Remove => {
            paste.delete_file(&conn, db_file.id())?;
            // do not update file in database
            db_changed = false;
            continue;
          },
          // doing nothing
          Update::Ignore => {},
        }

        match file.highlight_language {
          Update::Set(lang) => db_file.set_highlight_language(Some(lang)),
          Update::Remove => db_file.set_highlight_language(None),
          Update::Ignore => {},
        }

        if file.highlight_language.is_set() || file.highlight_language.is_remove() {
          db_changed = true;
        }

        if db_changed {
          diesel::update(files::table)
            .filter(files::id.eq(db_file.id()))
            .set(&*db_file)
            .execute(&*conn)?;
          db_changed = false;
        }
      },
      // adding file
      None => {
        let content = file.content.unwrap_set();
        paste.create_file(&conn, file.name, file.highlight_language.set(), content)?;
      },
    }
  }

  // commit if any files were changed
  // TODO: more descriptive commit message
  paste.commit_if_dirty(user.name(), user.email(), "update paste")?;

  Ok(Status::show_success(HttpStatus::NoContent, ()))
}
