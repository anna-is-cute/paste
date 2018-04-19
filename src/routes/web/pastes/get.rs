use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::schema::{pastes, users};
use errors::*;
use models::id::PasteId;
use models::paste::output::{Output, OutputFile};

use diesel::prelude::*;

use rocket::response::Redirect;

use rocket_contrib::Template;

use std::result;

#[get("/<id>")]
fn id(id: PasteId, conn: DbConn) -> Result<Redirect> {
  // FIXME: respect visibility rules
  let owner: Option<String> = users::table
    .inner_join(pastes::table)
    .filter(pastes::id.eq(*id))
    .select(users::username)
    .first(&*conn)
    .optional()?;
  let owner = owner.unwrap_or_else(|| "anonymous".into());
  Ok(Redirect::to(&format!("/{}/{}", owner, id)))
}

#[get("/<username>/<id>")]
fn username_id(username: String, id: PasteId, conn: DbConn) -> Result<Template> {
  // FIXME: respect visibility rules
  let paste: DbPaste = id.get(&conn)?.unwrap();

  // FIXME: check username

  // if let Some((status, kind)) = paste.check_access(user.as_ref().map(|x| x.id())) {
  //   return Ok(Status::show_error(status, kind));
  // }

  let files: Vec<OutputFile> = id.files(&conn)?
    .iter()
    .map(|x| x.as_output_file(true))
    .collect::<result::Result<_, _>>()?;

  let output = Output::new(
    *id,
    paste.name().clone(),
    paste.description().clone(),
    paste.visibility(),
    None,
    files,
  );

  let ctx = Context { paste: output };

  Ok(Template::render("paste/index", ctx))
}

#[derive(Serialize)]
struct Context {
  paste: Output,
}
