use crate::{
  database::{
    DbConn,
    schema::files,
  },
  models::{
    id::{PasteId, FileId},
    paste::update::{PasteFileUpdate, Update},
    status::{Status, ErrorKind},
  },
  routes::{RouteResult, RequiredUser},
};

use diesel::prelude::*;

use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

use std::{fs::OpenOptions, io::Write};

type UpdateResult = ::std::result::Result<Json<PasteFileUpdate>, ::rocket_contrib::SerdeError>;

#[patch("/<paste_id>/files/<file_id>", format = "application/json", data = "<file>")]
pub fn patch(paste_id: PasteId, file_id: FileId, file: UpdateResult, user: RequiredUser, conn: DbConn) -> RouteResult<()> {
  // TODO: can this be a request guard?
  let file = match file {
    Ok(x) => x.into_inner(),
    Err(e) => {
      let message = format!("could not parse json: {}", e);
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some(message))));
    },
  };
  // verify auth
  let paste = match paste_id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };
  if let Some((status, kind)) = paste.check_access(Some(user.id())) {
    return Ok(Status::show_error(status, kind));
  }

  if let Some(ref id) = file.id {
    if *id != file_id {
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::InvalidFile(Some("IDs must match".into()))));
    }
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
    if_chain! {
      if let Some(ref id) = file.id;
      if !db_files_ids.contains(id);
      then {
        return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::MissingFile));
      }
    }
    if_chain! {
      if let Some(ref name) = file.name;
      if db_files_names.contains(&name.as_str());
      then {
        return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::InvalidFile(Some("duplicate file name".into()))));
      }
    }
  }

    // file should be present due to check above
    let db_file = db_files.iter_mut().find(|x| x.id() == file_id).expect("missing file");
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
          .open(files_directory.join(db_file.id().simple().to_string()))?;
        f.write_all(&content.into_bytes())?;
        // FIXME: set is_binary field
      },
      // deleting file
      Update::Remove => {
        paste.delete_file(&conn, db_file.id())?;
        // do not update file in database
        db_changed = false;
      },
      // doing nothing
      Update::Ignore => {},
    }

    if db_changed {
      diesel::update(files::table)
        .filter(files::id.eq(*file_id))
        .set(&*db_file)
        .execute(&*conn)?;
    }

  // commit if any files were changed
  // TODO: more descriptive commit message
  paste.commit_if_dirty(user.name(), user.email(), "update paste")?;

  Ok(Status::show_success(HttpStatus::NoContent, ()))
}
