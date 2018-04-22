use database::DbConn;
use database::models::deletion_keys::NewDeletionKey;
use database::models::pastes::NewPaste;
use database::schema;
use errors::*;
use models::paste::{Visibility, Content};
use routes::web::OptionalWebUser;
use store::Store;

use diesel;
use diesel::prelude::*;

use rocket::http::{Cookies, Cookie};
use rocket::request::Form;
use rocket::response::Redirect;

#[post("/pastes", format = "application/x-www-form-urlencoded", data = "<paste>")]
fn post(paste: Form<PasteUpload>, user: OptionalWebUser, mut cookies: Cookies, conn: DbConn) -> Result<Redirect> {
  let paste = paste.into_inner();

  if paste.file_content.is_empty() {
    cookies.add(Cookie::new("error", "File content must not be empty."));
    return Ok(Redirect::to("lastpage"));
  }

  let anonymous = paste.anonymous.is_some();

  let user = if anonymous {
    None
  } else {
    user.into_inner()
  };

  if anonymous && paste.visibility == Visibility::Private {
    cookies.add(Cookie::new("error", "Cannot make anonymous private pastes."));
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
  diesel::insert_into(schema::pastes::table)
    .values(&np)
    .execute(&*conn)?;

  // TODO: show this to user
  let _deletion_key = if user.is_none() {
    let key = NewDeletionKey::generate(*id);
    diesel::insert_into(schema::deletion_keys::table)
      .values(&key)
      .execute(&*conn)?;
    Some(key.key())
  } else {
    None
  };

  let file_name = if paste.file_name.is_empty() {
    None
  } else {
    Some(paste.file_name)
  };

  id.create_file(&conn, file_name, Content::Text(paste.file_content))?;

  match user {
    Some(ref u) => id.commit(u.name(), u.email(), "create paste via web")?,
    None => id.commit("Anonymous", "none", "create paste via web")?,
  }

  Ok(Redirect::to(&format!("/{}", id.simple())))
}

#[derive(Debug, FromForm)]
struct PasteUpload {
  name: String,
  visibility: Visibility,
  description: String,
  file_name: String,
  file_content: String,
  anonymous: Option<String>,
}
