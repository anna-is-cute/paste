use crate::{
  backend::{errors::BackendError, pastes::*},
  database::DbConn,
  errors::*,
  models::paste::{Visibility, Content},
  routes::web::{OptionalWebUser, Session},
  utils::{FormDate, Language},
};

use percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};

use rocket::{
  request::Form,
  response::Redirect,
  State,
};

use sidekiq::Client as SidekiqClient;

fn handle_non_js(upload: &PasteUpload) -> Vec<MultiFile> {
  vec![
    MultiFile {
      name: upload.file_name.clone(),
      language: upload.file_language,
      content: upload.file_content.clone(),
    },
  ]
}

fn handle_js(input: &str) -> Result<Vec<MultiFile>> {
  let files: Vec<MultiFile> = serde_json::from_str(input)?;

  Ok(files)
}

#[post("/pastes", format = "application/x-www-form-urlencoded", data = "<paste>")]
fn post(paste: Form<PasteUpload>, user: OptionalWebUser, mut sess: Session, conn: DbConn, sidekiq: State<SidekiqClient>) -> Result<Redirect> {
  let paste = paste.into_inner();
  sess.set_form(&paste);

  if !sess.check_token(&paste.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("/"));
  }

  let user = if paste.anonymous.is_some() || user.is_none() {
    None
  } else {
    user.into_inner()
  };

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

  let files = files
    .into_iter()
    .map(|f| FilePayload {
      name: if f.name.is_empty() { None } else { Some(f.name) },
      highlight_language: f.language,
      content: Content::Text(f.content),
    })
    .collect();

  let pp = PastePayload {
    name,
    description,
    visibility: paste.visibility,
    expires: paste.expires.map(|x| x.into_inner()),
    author: user.as_ref(),
    files,
  };

  let CreateSuccess { paste, deletion_key, .. } = match pp.create(&conn, &*sidekiq) {
    Ok(s) => s,
    Err(e) => {
      let msg = e.into_web_message()?;
      sess.add_data("error", msg);
      return Ok(Redirect::to("/"));
    },
  };

  if let Some(dk) = deletion_key {
    sess.add_data("deletion_key", dk.key().simple().to_string());
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
  Ok(Redirect::to(&format!("/p/{}/{}", username, paste.id().simple())))
}

#[derive(Debug, FromForm, Serialize)]
struct PasteUpload {
  name: String,
  visibility: Visibility,
  description: String,
  expires: Option<FormDate>,
  #[serde(skip)]
  file_name: String,
  #[serde(skip)]
  file_language: Option<Language>,
  #[serde(skip)]
  file_content: String,
  #[serde(skip)]
  upload_json: Option<String>,
  #[serde(skip)]
  anonymous: Option<String>,
  #[serde(skip)]
  anti_csrf_token: String,
}

#[derive(Debug, Deserialize)]
struct MultiFile {
  name: String,
  language: Option<Language>,
  content: String,
}
