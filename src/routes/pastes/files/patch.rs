use database::DbConn;
use database::schema::files;
use models::id::PasteId;
use models::paste::update::PasteFileUpdate;
use models::status::{Status, ErrorKind};
use routes::{RouteResult, RequiredUser};

use diesel;
use diesel::prelude::*;

use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

use uuid::Uuid;

use std::fs::OpenOptions;
use std::io::Write;

type UpdateResult = ::std::result::Result<Json<Vec<PasteFileUpdate>>, ::rocket_contrib::SerdeError>;

#[patch("/<paste_id>/files", format = "application/json", data = "<info>")]
pub fn patch(paste_id: PasteId, info: UpdateResult, user: RequiredUser, conn: DbConn) -> RouteResult<()> {
  // TODO: can this be a request guard?
  let info = match info {
    Ok(x) => x.into_inner(),
    Err(e) => {
      let message = format!("could not parse json: {}", e);
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some(message))));
    },
  };

  if info.is_empty() {
    return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some("array cannot be empty".into()))));
  }

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
  let files_directory = paste_id.files_directory();

  let mut db_files = paste_id.files(&conn)?;
  {
    let db_files_ids: Vec<Uuid> = db_files.iter().map(|x| x.id()).collect();
    let db_files_names: Vec<&String> = db_files.iter().map(|x| x.name()).collect();
    // verify all files before making changes
    if info.iter().filter_map(|x| x.id).any(|x| !db_files_ids.contains(&x)) {
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::MissingFile));
    }
    if info.iter().filter_map(|x| x.name.as_ref()).any(|x| db_files_names.contains(&x)) {
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::InvalidFile(Some("duplicate file name".into()))));
    }
    if info.iter().any(|x| x.id.is_none() && (x.content.is_none() || x.content.as_ref().map(|z| z.is_none()) == Some(true))) {
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::InvalidFile(Some("new files must have content".into()))));
    }
  }

  for file in info {
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
            // FIXME: set is_binary field
          },
          // deleting file
          Some(None) => {
            paste_id.delete_file(&conn, db_file.id())?;
            // do not update file in database
            db_changed = false;
            continue;
          },
          // doing nothing
          None => {},
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
        let content = file.content.expect("missing content 1").expect("missing content 2");
        paste_id.create_file(&conn, file.name, content)?;
      },
    }
  }

  // commit if any files were changed
  let username = user.username().as_str();
  // TODO: more descriptive commit message
  paste_id.commit_if_dirty(&username, &format!("{}@paste.com", username), "update paste")?;

  Ok(Status::show_success(HttpStatus::NoContent, ()))
}
