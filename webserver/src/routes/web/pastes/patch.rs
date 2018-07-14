use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::models::users::User;
use database::schema::{users, files};
use errors::*;
use models::id::{PasteId, FileId};
use models::paste::{Visibility, Content};
use models::paste::update::{MetadataUpdate, Update};
use routes::web::{OptionalWebUser, Rst, Session};
use utils::{FormDate, Language};

use diesel;
use diesel::prelude::*;

use percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};

use rocket::http::Status as HttpStatus;
use rocket::request::LenientForm;
use rocket::response::Redirect;
use rocket::State;

use serde_json;

use sidekiq::Client as SidekiqClient;

use unicode_segmentation::UnicodeSegmentation;

use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Write;
use std::result;

fn handle_js(input: &str) -> Result<Vec<MultiFile>> {
  let files: Vec<MultiFile> = serde_json::from_str(input)?;

  Ok(files)
}

fn check_paste(paste: &PasteUpdate, files: &[MultiFile]) -> result::Result<(), String> {
  const MAX_SIZE: usize = 25 * 1024;

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

#[patch("/p/<username>/<paste_id>", format = "application/x-www-form-urlencoded", data = "<update>")]
fn patch(update: LenientForm<PasteUpdate>, username: String, paste_id: PasteId, user: OptionalWebUser, mut sess: Session, conn: DbConn, sidekiq: State<SidekiqClient>) -> Result<Rst> {
  let update = update.into_inner();
  sess.set_form(&update);

  if !sess.check_token(&update.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Rst::Redirect(Redirect::to("lastpage")));
  }

  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  let mut paste: DbPaste = match paste_id.get(&conn)? {
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

  if let Some((status, _)) = paste.check_access(user.id()) {
    return Ok(Rst::Status(status));
  }

  match paste.author_id() {
    Some(author) => if author != user.id() {
      if paste.visibility() == Visibility::Private {
        return Ok(Rst::Status(HttpStatus::NotFound));
      } else {
        return Ok(Rst::Status(HttpStatus::Forbidden));
      }
    },
    None => {
      sess.add_data("error", "Cannot edit anonymous pastes.");
      return Ok(Rst::Redirect(Redirect::to("lastpage")));
    },
  }

  let files = match update.upload_json {
    Some(ref json) => match handle_js(json) {
      Ok(f) => f,
      Err(_) => {
        sess.add_data("error", "Invalid JSON. Did you tamper with the form?");
        return Ok(Rst::Redirect(Redirect::to("lastpage")));
      },
    },
    None => Default::default(),
  };

  if let Err(e) = check_paste(&update, &files) {
    sess.add_data("error", e);
    return Ok(Rst::Redirect(Redirect::to("lastpage")));
  }

  let expires = match update.expires {
    Some(ref f) if Some(**f) == paste.expires() => Update::Ignore,
    Some(f) => Update::Set(f.into_inner()),
    None => Update::Remove,
  };

  let metadata = MetadataUpdate {
    name: into_update(update.name, paste.name()),
    description: into_update(update.description, paste.description()),
    expires,
    visibility: if update.visibility == paste.visibility() {
      None
    } else {
      Some(update.visibility)
    },
  };

  paste.update(&conn, &*sidekiq, &metadata)?;

  let mut db_changed = false;
  // TODO: this needs much refactor love
  // update files and database if necessary
  let files_directory = paste.files_directory();

  let mut db_files = paste_id.files(&conn)?;
  {
    let db_files_ids: Vec<FileId> = db_files.iter().map(|x| x.id()).collect();
    // verify all files before making changes
    if files.iter().filter_map(|x| x.id).any(|x| !db_files_ids.contains(&x)) {
      sess.add_data("error", "An invalid file ID was provided.");
      return Ok(Rst::Redirect(Redirect::to("lastpage")));
    }
  }

  // filter out IDs that are in the updated files to find the removed files
  let removed: Vec<FileId> = db_files
    .iter()
    .filter(|x| !files.iter().any(|f| f.id == Some(x.id())))
    .map(|x| x.id())
    .collect();

  {
    let mut names: Vec<Cow<str>> = files
      .iter()
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
      sess.add_data("error", "Duplicate file names are not allowed.");
      return Ok(Rst::Redirect(Redirect::to("lastpage")));
    }
  }

  for file in files {
    match file.id {
      // updating existing file
      Some(id) => {
        // file should be present due to check above
        let mut db_file = db_files.iter_mut().find(|x| x.id() == id).expect("missing file");
        if !file.name.is_empty() && file.name != db_file.name() {
          db_file.set_name(file.name);
          db_changed = true;
        }
        let mut f = OpenOptions::new()
          .write(true)
          .truncate(true)
          .open(files_directory.join(db_file.id().simple().to_string()))?;
        f.write_all(&file.content.into_bytes())?;
        // FIXME: set is_binary field

        if file.language != db_file.highlight_language() {
          db_changed = true;
        }
        db_file.set_highlight_language(file.language);

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
        // should be content due to checks we did before this
        let name = if file.name.is_empty() {
          None
        } else {
          Some(file.name)
        };
        let content = Content::Text(file.content);
        paste.create_file(&conn, name, file.language, content)?;
      },
    }
  }

  for file in removed {
    paste.delete_file(&conn, file)?;
  }

  // commit if any files were changed
  // TODO: more descriptive commit message
  paste.commit_if_dirty(user.name(), user.email(), "update paste via web")?;

  sess.add_data("info", "Paste updated.");

  sess.take_form();

  let username = utf8_percent_encode(&username, PATH_SEGMENT_ENCODE_SET);
  Ok(Rst::Redirect(Redirect::to(&format!("/p/{}/{}", username, paste_id.simple()))))
}

#[derive(Debug, FromForm, Serialize)]
struct PasteUpdate {
  name: String,
  visibility: Visibility,
  description: String,
  expires: Option<FormDate>,
  #[serde(skip)]
  upload_json: Option<String>,
  #[serde(skip)]
  anti_csrf_token: String,
}

#[derive(Debug, Deserialize)]
struct MultiFile {
  #[serde(default)]
  id: Option<FileId>,
  name: String,
  language: Option<Language>,
  content: String,
}

fn into_update<N, O, S>(new: N, old: Option<O>) -> Update<S>
  where N: Into<String>,
        O: AsRef<str>,
        S: From<String>,
{
  let new = new.into();
  let old = old.as_ref().map(|x| x.as_ref());
  if new.is_empty() && old.is_some() {
    Update::Remove
  } else if Some(new.as_str()) == old {
    Update::Ignore
  } else {
    Update::Set(new.into())
  }
}
