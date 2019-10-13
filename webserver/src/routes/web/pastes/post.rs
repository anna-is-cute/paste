use crate::{
  backend::{errors::BackendError, pastes::*},
  config::Config,
  database::DbConn,
  errors::*,
  models::paste::{Visibility, Content},
  routes::web::{AntiSpam, OptionalWebUser, Session},
  utils::{FormDate, Language},
};

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
pub fn post(paste: Form<PasteUpload>, user: OptionalWebUser, mut sess: Session, antispam: AntiSpam, conn: DbConn, sidekiq: State<SidekiqClient>, config: State<Config>) -> Result<Redirect> {
  let paste = paste.into_inner();
  sess.set_form(&paste);

  if !sess.check_token(&paste.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to(uri!(crate::routes::web::index::get)));
  }

  if !paste.honeypot.is_empty() {
    sess.add_data("error", "An error occurred. Please try again.");
    return Ok(Redirect::to(uri!(crate::routes::web::index::get)));
  }

  if paste.js_check != format!("{}{}", antispam.js.0, antispam.js.1) && paste.no_js_check != antispam.no_js.2.to_string() {
    sess.add_data("error", "An error occurred. Please try again.");
    return Ok(Redirect::to(uri!(crate::routes::web::index::get)));
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
        return Ok(Redirect::to(uri!(crate::routes::web::index::get)));
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

  let CreateSuccess { paste, deletion_key, .. } = match pp.create(&*config, &conn, &*sidekiq) {
    Ok(s) => s,
    Err(e) => {
      let msg = e.into_web_message()?;
      sess.add_data("error", msg);
      return Ok(Redirect::to(uri!(crate::routes::web::index::get)));
    },
  };

  if let Some(dk) = deletion_key {
    sess.add_data(
      format!("deletion_key_{}", paste.id().to_simple()),
      dk.key().to_simple().to_string()
    );
  }

  match user {
    Some(ref u) => paste.commit(&*config, u.name(), u.email(), "create paste via web")?,
    None => paste.commit(&*config, "Anonymous", "none", "create paste via web")?,
  }

  let username = match user {
    Some(ref u) => u.username(),
    None => "anonymous",
  };

  sess.take_form();

  Ok(Redirect::to(uri!(
    crate::routes::web::pastes::get::users_username_id:
    username,
    paste.id(),
  )))
}

#[derive(Debug, FromForm, Serialize)]
pub struct PasteUpload {
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
  #[serde(skip)]
  #[form(field = "email")]
  honeypot: String,
  #[serde(skip)]
  #[form(field = "js-check")]
  js_check: String,
  #[serde(skip)]
  #[form(field = "no-js-check")]
  no_js_check: String,
}

#[derive(Debug, Deserialize)]
struct MultiFile {
  name: String,
  language: Option<Language>,
  content: String,
}
