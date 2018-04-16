use database::{DbConn, schema};
use database::schema::{pastes, files};
use database::models::pastes::{NewPaste, Paste as DbPaste};
use database::models::deletion_keys::NewDeletionKey;
use database::models::files::{NewFile, File as DbFile};
use models::paste::{Paste, Visibility, PasteId, Content};
use models::paste::update::PasteUpdate;
use models::status::{Status, ErrorKind};
use routes::{RouteResult, RequiredUser};
use store::Store;

use diesel;
use diesel::prelude::*;

use git2::{Repository, Signature};

use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

use uuid::Uuid;

use std::fs::{self, File, OpenOptions};
use std::io::Write;

type UpdateResult = ::std::result::Result<Json<PasteUpdate>, ::rocket_contrib::SerdeError>;

#[patch("/<id>", format = "application/json", data = "<info>")]
pub fn patch(id: PasteId, info: UpdateResult, user: RequiredUser, conn: DbConn) -> RouteResult<()> {
  // TODO: can this be a request guard?
  let info = match info {
    Ok(x) => x.into_inner(),
    Err(e) => {
      let message = format!("could not parse json: {}", e);
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some(message))));
    },
  };

  // verify auth
  let mut paste: DbPaste = match pastes::table.find(*id).first(&*conn).optional()? {
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
  let mut db_changed = false;
  if let Some(update) = info.metadata.name {
    paste.set_name(update);
    db_changed = true;
  }
  if let Some(update) = info.metadata.visibility {
    paste.set_visibility(update);
    db_changed = true;
  }
  if db_changed {
    diesel::update(pastes::table).set(&paste).execute(&*conn)?;
    db_changed = false;
  }

  let mut fs_changed = false;

  // update files and database if necessary
  if let Some(files) = info.files {
    let files_directory = id.files_directory();

    let mut db_files: Vec<DbFile> = DbFile::belonging_to(&paste).load(&*conn)?;
    let mut db_files_len = db_files.len();
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
              let bytes = match content {
                Content::Text(s) => s.into_bytes(),
                Content::Base64(b) | Content::Gzip(b) | Content::Xz(b) => b,
              };
              f.write_all(&bytes)?;
              fs_changed = true;
            },
            // deleting file
            Some(None) => {
              // FIXME: if all files are deleted, delete paste, too
              diesel::delete(&*db_file).execute(&*conn)?;
              fs::remove_file(files_directory.join(db_file.id().simple().to_string()))?;
              fs_changed = true;
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
          let file_id = Uuid::new_v4();

          let mut f = File::create(files_directory.join(file_id.simple().to_string()))?;
          // should be safe because of check above
          let bytes = match file.content.expect("missing content 1").expect("missing content 2") {
            Content::Text(s) => s.into_bytes(),
            Content::Base64(b) | Content::Gzip(b) | Content::Xz(b) => b,
          };
          f.write_all(&bytes)?;
          fs_changed = true;
          let file_name = file.name.unwrap_or_else(|| {
            db_files_len += 1;
            format!("pastefile{}", db_files_len)
          });

          let nf = NewFile::new(file_id, paste.id(), file_name, None);
          diesel::insert_into(files::table).values(&nf).execute(&*conn)?;
        },
      }
    }
  }

  // commit if any files were changed
  if fs_changed {
    let paste_id = PasteId(paste.id());
    let repo = Repository::open(paste_id.files_directory())?;
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let head_id = repo.refname_to_id("HEAD")?;
    let parent = repo.find_commit(head_id)?;
    let username = user.username().as_str();
    // TODO: figure out what email should be
    let sig = Signature::now(username, &format!("{}@paste.com", username))?;
    // TODO: more descriptive commit name?
    repo.commit(Some("HEAD"), &sig, &sig, "update paste", &tree, &[&parent])?;
  }

  // return status (204?)
  Ok(Status::show_success(HttpStatus::NoContent, ()))
}
