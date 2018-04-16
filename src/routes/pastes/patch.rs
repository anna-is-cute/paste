use database::DbConn;
use database::schema::{pastes, files};
use database::models::pastes::Paste as DbPaste;
use database::models::files::File as DbFile;
use models::paste::{Visibility, PasteId};
use models::paste::update::PasteUpdate;
use models::status::{Status, ErrorKind};
use routes::{RouteResult, RequiredUser};

use diesel;
use diesel::prelude::*;

use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

use uuid::Uuid;

use std::fs::OpenOptions;
use std::io::Write;

type UpdateResult = ::std::result::Result<Json<PasteUpdate>, ::rocket_contrib::SerdeError>;

#[patch("/<paste_id>", format = "application/json", data = "<info>")]
pub fn patch(paste_id: PasteId, info: UpdateResult, user: RequiredUser, conn: DbConn) -> RouteResult<()> {
  // TODO: can this be a request guard?
  let info = match info {
    Ok(x) => x.into_inner(),
    Err(e) => {
      let message = format!("could not parse json: {}", e);
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some(message))));
    },
  };

  // verify auth
  let mut paste: DbPaste = match pastes::table.find(*paste_id).first(&*conn).optional()? {
    Some(p) => p,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };
  if *paste.author_id() != Some(user.id()) {
    return Ok(if paste.visibility() == Visibility::Private {
      Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)
    } else {
      Status::show_error(HttpStatus::Forbidden, ErrorKind::NotAllowed)
    });
  }

  // update paste and database if necessary
  paste.update(&conn, &info)?;

  let mut db_changed = false;

  // update files and database if necessary
  if let Some(files) = info.files {
    let files_directory = paste_id.files_directory();

    let mut db_files: Vec<DbFile> = DbFile::belonging_to(&paste).load(&*conn)?;
    {
      let db_files_ids: Vec<Uuid> = db_files.iter().map(|x| x.id()).collect();
      let db_files_names: Vec<&String> = db_files.iter().map(|x| x.name()).collect();
      // verify all files before making changes
      if files.iter().filter_map(|x| x.id).any(|x| !db_files_ids.contains(&x)) {
        return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::MissingFile));
      }
      if files.iter().filter_map(|x| x.name.as_ref()).any(|x| db_files_names.contains(&x)) {
        return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::InvalidFile(Some("duplicate file name".into()))));
      }
      if files.iter().any(|x| x.id.is_none() && (x.content.is_none() || x.content.as_ref().map(|z| z.is_none()) == Some(true))) {
        return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::InvalidFile(Some("new files must have content".into()))));
      }
    }

    for file in files {
      match file.id {
        // updating existing file
        Some(id) => {
          // file should be present due to check above
          let mut db_file = db_files.iter_mut().find(|x| x.id() == id).expect("missing file");
          if let Some(name) = file.name {
            db_file.set_name(name);
            db_changed = true;
          }
          match file.content {
            // replacing contents
            Some(Some(content)) => {
              let mut f = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(files_directory.join(db_file.id().simple().to_string()))?;
              f.write_all(&content.into_bytes())?;
            },
            // deleting file
            Some(None) => {
              paste_id.delete_file(&conn, db_file.id())?;
              db_changed = false;
              // do not update file in database
              continue;
            },
            // doing nothing
            None => {},
          }

          if db_changed {
            diesel::update(files::table).set(&*db_file).execute(&*conn)?;
            db_changed = false;
          }
        },
        // adding file
        None => {
          let content = file.content.expect("missing content 1").expect("missing content 2");
          paste_id.create_file(&conn, file.name, content)?;
        },
      }
    }
  }

  // commit if any files were changed
  let username = user.username().as_str();
  // TODO: more descriptive commit message
  paste_id.commit_if_dirty(&username, &format!("{}@paste.com", username), "update paste")?;

  // return status (204?)
  Ok(Status::show_success(HttpStatus::NoContent, ()))
}
