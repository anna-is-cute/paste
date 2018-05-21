use config::Config;
use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::models::users::User;
use database::schema::{pastes, users};
use errors::*;
use models::id::PasteId;
use models::paste::Visibility;
use models::paste::output::{Output, OutputFile, OutputAuthor};
use routes::web::{context, Rst, OptionalWebUser, Session};

use diesel::prelude::*;

use percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};

use rocket::http::Status as HttpStatus;
use rocket::response::Redirect;
use rocket::State;

use rocket_contrib::Template;

use std::result;

#[get("/<id>", rank = 10)]
fn id(id: PasteId, user: OptionalWebUser, conn: DbConn) -> Result<Rst> {
  let result: Option<(Option<String>, DbPaste)> = pastes::table
    .left_join(users::table)
    .select((users::username.nullable(), pastes::all_columns))
    .filter(pastes::id.eq(*id))
    .first(&*conn)
    .optional()?;

  let (owner, paste) = match result {
    Some(x) => x,
    None => return Ok(Rst::Status(HttpStatus::NotFound)),
  };

  if let Some((status, _)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Rst::Status(status));
  }

  let username = owner.unwrap_or_else(|| "anonymous".into());
  let owner = utf8_percent_encode(
    &username,
    PATH_SEGMENT_ENCODE_SET,
  );
  Ok(Rst::Redirect(Redirect::to(&format!("/pastes/{}/{}", owner, id))))
}

#[get("/<username>/<id>", rank = 10)]
fn username_id(username: String, id: PasteId) -> Redirect {
  let username = utf8_percent_encode(&username, PATH_SEGMENT_ENCODE_SET);
  Redirect::to(&format!("/pastes/{}/{}", username, id))
}

#[get("/pastes/<username>/<id>")]
fn users_username_id(username: String, id: PasteId, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  let paste: DbPaste = match id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Rst::Status(HttpStatus::NotFound)),
  };

  let (expected_username, author): (String, Option<OutputAuthor>) = match paste.author_id() {
    Some(author) => {
      let user: User = users::table.find(author).first(&*conn)?;
      (user.username().to_string(), Some(OutputAuthor::new(author, user.username(), user.name())))
    },
    None => ("anonymous".into(), None),
  };

  if username != expected_username {
    return Ok(Rst::Status(HttpStatus::NotFound));
  }

  if let Some((status, _)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Rst::Status(status));
  }

  let files: Vec<OutputFile> = id.files(&conn)?
    .iter()
    .map(|x| x.as_output_file(true, &paste))
    .collect::<result::Result<_, _>>()?;

  let output = Output::new(
    id,
    author,
    paste.name(),
    paste.description(),
    paste.visibility(),
    None,
    files,
  );

  let is_owner = paste.author_id().is_some() && user.as_ref().map(|x| x.id()) == paste.author_id();

  let author_name = output.author.as_ref().map(|x| x.username.to_string()).unwrap_or_else(|| "anonymous".into());

  let mut ctx = context(&*config, user.as_ref(), &mut sess);
  ctx["paste"] = json!(output);
  ctx["user"] = json!(*user);
  ctx["deletion_key"] = json!(sess.data.remove("deletion_key"));
  ctx["is_owner"] = json!(is_owner);
  ctx["author_name"] = json!(author_name);

  Ok(Rst::Template(Template::render("paste/index", ctx)))
}

#[get("/pastes/<username>/<id>/edit")]
fn edit(username: String, id: PasteId, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  let paste: DbPaste = match id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Rst::Status(HttpStatus::NotFound)),
  };

  let (expected_username, author): (String, Option<OutputAuthor>) = match paste.author_id() {
    Some(author) => {
      let user: User = users::table.find(author).first(&*conn)?;
      (user.username().to_string(), Some(OutputAuthor::new(author, user.username(), user.name())))
    },
    None => ("anonymous".into(), None),
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

  // should be authed beyond this point

  let files: Vec<OutputFile> = id.files(&conn)?
    .iter()
    .map(|x| x.as_output_file(true, &paste))
    .collect::<result::Result<_, _>>()?;

  let output = Output::new(
    id,
    author,
    paste.name(),
    paste.description(),
    paste.visibility(),
    None,
    files,
  );

  let is_owner = paste.author_id().is_some() && Some(user.id()) == paste.author_id();

  let author_name = output.author.as_ref().map(|x| x.username.to_string()).unwrap_or_else(|| "anonymous".into());

  let mut ctx = context(&*config, Some(&user), &mut sess);
  ctx["paste"] = json!(output);
  ctx["is_owner"] = json!(is_owner);
  ctx["author_name"] = json!(author_name);

  Ok(Rst::Template(Template::render("paste/edit", ctx)))
}
