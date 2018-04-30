use database::DbConn;
use database::models::deletion_keys::{NewDeletionKey, DeletionKey};
use database::models::pastes::{NewPaste, Paste as DbPaste};
use database::models::users::User;
use database::schema::{users, pastes, deletion_keys};
use errors::*;
use models::id::PasteId;
use models::paste::{Visibility, Content};
use routes::web::{Rst, OptionalWebUser, Session};
use store::Store;

use diesel;
use diesel::prelude::*;

use rocket::http::Status as HttpStatus;
use rocket::request::Form;
use rocket::response::Redirect;

use serde_json;

use unicode_segmentation::UnicodeSegmentation;

use uuid::Uuid;

use std::result;
use std::str::FromStr;

fn handle_non_js(upload: &PasteUpload) -> Vec<MultiFile> {
  vec![
    MultiFile {
      name: upload.file_name.clone(),
      content: upload.file_content.clone(),
    },
  ]
}

fn handle_js(input: &str) -> Result<Vec<MultiFile>> {
  let files: Vec<MultiFile> = serde_json::from_str(input)?;

  Ok(files)
}

fn check_paste(paste: &PasteUpload, files: &[MultiFile]) -> result::Result<(), String> {
  const MAX_SIZE: usize = 25 * 1024;

  if files.is_empty() {
    return Err("You must upload at least one file.".into());
  }

  if paste.name.len() > MAX_SIZE {
    return Err("Paste name must be less than 25 KiB.".into());
  }

  if paste.name.graphemes(true).count() > 255 {
    return Err("Paste name must be less than or equal to 255 graphemes.".into());
  }

  if paste.description.graphemes(true).count() > 255 {
    return Err("Paste description must be less than or equal to 255 graphemes.".into());
  }

  if paste.name.len() > MAX_SIZE {
    return Err("Paste description must be less than 25 KiB.".into());
  }

  if files.iter().any(|x| x.content.is_empty()) {
    return Err("File content must not be empty.".into());
  }

  if files.iter().any(|x| x.name.len() > MAX_SIZE) {
    return Err("File names must be less than 25 KiB.".into());
  }

  if files.iter().any(|x| x.name.graphemes(true).count() > 255) {
    return Err("File names must be less than or equal to 255 graphemes.".into());
  }

  Ok(())
}

#[post("/pastes", format = "application/x-www-form-urlencoded", data = "<paste>")]
fn post(paste: Form<PasteUpload>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let paste = paste.into_inner();

  let anonymous = paste.anonymous.is_some();

  let user = if anonymous {
    None
  } else {
    user.into_inner()
  };

  if anonymous && paste.visibility == Visibility::Private {
    sess.data.insert("error".into(), "Cannot make anonymous private pastes.".into());
    return Ok(Redirect::to("lastpage"));
  }

  let files = match paste.upload_json {
    Some(ref json) => match handle_js(json) {
      Ok(f) => f,
      Err(_) => {
        sess.data.insert("error".into(), "Invalid JSON. Did you tamper with the form?".into());
        return Ok(Redirect::to("lastpage"));
      },
    },
    None => handle_non_js(&paste),
  };

  if files.is_empty() {
    sess.data.insert("error".into(), "You must upload at least one file.".into());
    return Ok(Redirect::to("lastpage"));
  }

  if let Err(e) = check_paste(&paste, &files) {
    sess.data.insert("error".into(), e);
    return Ok(Redirect::to("lastpage"));
  }

  let id = Store::new_paste()?;

  let name = if paste.name.is_empty() {
    None
  } else {
    Some(paste.name)
  };

  let description = if paste.description.is_empty() {
    None
  } else {
    Some(paste.description)
  };

  // TODO: refactor
  let np = NewPaste::new(
    *id,
    name,
    description,
    paste.visibility,
    user.as_ref().map(|x| x.id()),
    None,
  );
  diesel::insert_into(pastes::table)
    .values(&np)
    .execute(&*conn)?;

  if user.is_none() {
    let key = NewDeletionKey::generate(*id);
    diesel::insert_into(deletion_keys::table)
      .values(&key)
      .execute(&*conn)?;
    sess.data.insert("deletion_key".into(), key.key().simple().to_string());
  }

  for file in files {
    let file_name = if file.name.is_empty() {
      None
    } else {
      Some(file.name)
    };

    id.create_file(&conn, file_name, Content::Text(file.content))?;
  }

  match user {
    Some(ref u) => id.commit(u.name(), u.email(), "create paste via web")?,
    None => id.commit("Anonymous", "none", "create paste via web")?,
  }

  let username = match user {
    Some(ref u) => u.username().clone(),
    None => "anonymous".into(),
  };

  Ok(Redirect::to(&format!("/users/{}/{}", username, id.simple())))
}

#[derive(Debug, FromForm)]
struct PasteUpload {
  name: String,
  visibility: Visibility,
  description: String,
  file_name: String,
  file_content: String,
  upload_json: Option<String>,
  anonymous: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MultiFile {
  name: String,
  content: String,
}

#[post("/users/<username>/<id>/delete", format = "application/x-www-form-urlencoded", data = "<deletion>")]
fn delete(deletion: Form<PasteDeletion>, username: String, id: PasteId, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  let paste: DbPaste = match id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Rst::Status(HttpStatus::NotFound)),
  };

  let expected_username: String = match paste.author_id() {
    Some(author) => {
      let user: User = users::table.find(author).first(&*conn)?;
      user.username().to_string()
    },
    None => "anonymous".into(),
  };

  if username != expected_username {
    return Ok(Rst::Status(HttpStatus::NotFound));
  }

  if let Some((status, _)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Rst::Status(status));
  }

  match paste.author_id() {
    Some(author) => if Some(*author) != user.as_ref().map(|x| x.id()) {
      if paste.visibility() == Visibility::Private {
        return Ok(Rst::Status(HttpStatus::NotFound));
      } else {
        return Ok(Rst::Status(HttpStatus::Forbidden));
      }
    },
    None => {
      let key = match deletion.into_inner().key {
        Some(k) => k,
        None => {
          sess.data.insert("error".into(), "Anonymous pastes require a deletion key to delete.".into());
          return Ok(Rst::Redirect(Redirect::to("lastpage")));
        },
      };

      let key = match Uuid::from_str(&key) {
        Ok(k) => k,
        Err(_) => {
          sess.data.insert("error".into(), "Invalid deletion key.".into());
          return Ok(Rst::Redirect(Redirect::to("lastpage")));
        },
      };

      let db_key: DeletionKey = match deletion_keys::table.find(&key).first(&*conn).optional()? {
        Some(k) => k,
        None => {
          sess.data.insert("error".into(), "Invalid deletion key.".into());
          return Ok(Rst::Redirect(Redirect::to("lastpage")));
        },
      };

      if db_key.paste_id() != paste.id() {
        sess.data.insert("error".into(), "Invalid deletion key.".into());
        return Ok(Rst::Redirect(Redirect::to("lastpage")));
      }
    },
  }

  // should be authed beyond this point

  id.delete(&conn)?;

  Ok(Rst::Redirect(Redirect::to("/")))
}

#[derive(Debug, FromForm)]
struct PasteDeletion {
  key: Option<String>,
}
