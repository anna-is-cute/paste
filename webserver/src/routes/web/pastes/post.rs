use database::DbConn;
use database::models::deletion_keys::NewDeletionKey;
use database::models::pastes::{Paste, NewPaste};
use database::schema::{pastes, deletion_keys};
use errors::*;
use models::paste::{Visibility, Content};
use routes::web::{OptionalWebUser, Session};
use store::Store;

use diesel;
use diesel::prelude::*;

use percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};

use rocket::request::Form;
use rocket::response::Redirect;

use serde_json;

use unicode_segmentation::UnicodeSegmentation;

use std::borrow::Cow;
use std::result;

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

  if files.len() > 1 {
    let mut names: Vec<Cow<str>> = files.iter()
      .enumerate()
      .map(|(i, x)| if x.name.is_empty() {
        Cow::Owned(format!("pastefile{}", i + 1))
      } else {
        Cow::Borrowed(x.name.as_str())
      })
      .collect();
    let len = names.len();
    names.sort();
    names.dedup();
    if len != names.len() {
      return Err("Duplicate file names are not allowed.".into());
    }
  }

  if paste.name.len() > MAX_SIZE {
    return Err("Paste name must be less than 25 KiB.".into());
  }

  if paste.name.graphemes(true).count() > 255 {
    return Err("Paste name must be less than or equal to 255 graphemes.".into());
  }

  if paste.description.len() > MAX_SIZE {
    return Err("Paste description must be less than 25 KiB.".into());
  }

  if paste.description.graphemes(true).count() > 255 {
    return Err("Paste description must be less than or equal to 255 graphemes.".into());
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
  sess.set_form(&paste);

  if !sess.check_token(&paste.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("/"));
  }

  let anonymous = paste.anonymous.is_some() || user.is_none();

  let user = if anonymous {
    None
  } else {
    user.into_inner()
  };

  if anonymous && paste.visibility == Visibility::Private {
    sess.add_data("error", "Cannot make anonymous private pastes.");
    return Ok(Redirect::to("/"));
  }

  let files = match paste.upload_json {
    Some(ref json) => match handle_js(json) {
      Ok(f) => f,
      Err(_) => {
        sess.add_data("error", "Invalid JSON. Did you tamper with the form?");
        return Ok(Redirect::to("/"));
      },
    },
    None => handle_non_js(&paste),
  };

  if files.is_empty() {
    sess.add_data("error", "You must upload at least one file.");
    return Ok(Redirect::to("/"));
  }

  if let Err(e) = check_paste(&paste, &files) {
    sess.add_data("error", e);
    return Ok(Redirect::to("/"));
  }

  let id = Store::new_paste(user.as_ref().map(|x| x.id()))?;

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
    id,
    name,
    description,
    paste.visibility,
    user.as_ref().map(|x| x.id()),
    None,
  );
  let paste: Paste = diesel::insert_into(pastes::table)
    .values(&np)
    .get_result(&*conn)?;

  if user.is_none() {
    let key = NewDeletionKey::generate(id);
    diesel::insert_into(deletion_keys::table)
      .values(&key)
      .execute(&*conn)?;
    sess.add_data("deletion_key", key.key().simple().to_string());
  }

  for file in files {
    let file_name = if file.name.is_empty() {
      None
    } else {
      Some(file.name)
    };

    paste.create_file(&conn, file_name, Content::Text(file.content))?;
  }

  match user {
    Some(ref u) => paste.commit(u.name(), u.email(), "create paste via web")?,
    None => paste.commit("Anonymous", "none", "create paste via web")?,
  }

  let username = match user {
    Some(ref u) => u.username(),
    None => "anonymous",
  };

  sess.take_form();

  let username = utf8_percent_encode(username, PATH_SEGMENT_ENCODE_SET);
  Ok(Redirect::to(&format!("/pastes/{}/{}", username, id.simple())))
}

#[derive(Debug, FromForm, Serialize)]
struct PasteUpload {
  name: String,
  visibility: Visibility,
  description: String,
  file_name: String,
  file_content: String,
  upload_json: Option<String>,
  #[serde(skip)]
  anonymous: Option<String>,
  #[serde(skip)]
  anti_csrf_token: String,
}

#[derive(Debug, Deserialize)]
struct MultiFile {
  name: String,
  content: String,
}
