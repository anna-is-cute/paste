use crate::{
  config::Config,
  database::{
    DbConn,
    models::{pastes::Paste as DbPaste, users::User},
    schema::{pastes, users},
  },
  errors::*,
  models::{
    id::{PasteId, FileId},
    paste::{
      Content, Visibility,
      output::{Output, OutputFile, OutputAuthor},
    },
  },
  routes::web::{context, Rst, OptionalWebUser, Session},
  utils::{post_processing, Language},
};

use ammonia::Builder;

use comrak::{markdown_to_html, ComrakOptions};

use diesel::prelude::*;

use hashbrown::HashMap;

use rocket::{
  http::Status as HttpStatus,
  response::Redirect,
  State,
};

use rocket_contrib::templates::Template;

use serde_json::json;

lazy_static! {
  static ref OPTIONS: ComrakOptions = ComrakOptions {
    github_pre_lang: true,
    ext_strikethrough: true,
    ext_table: true,
    ext_autolink: true,
    // let's see how https://github.com/notriddle/ammonia/issues/100 turns out
    // ext_tasklist: true,
    ext_footnotes: true,
    .. Default::default()
  };

  static ref CLEANER: Builder<'static> = {
    let mut b = Builder::default();
    b.link_rel(Some("noopener noreferrer nofollow"));
    b
  };
}

#[get("/<id>", rank = 10)]
pub fn id(id: PasteId, user: OptionalWebUser, conn: DbConn) -> Result<Rst> {
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
  Ok(Rst::Redirect(Redirect::to(uri!(
    crate::routes::web::pastes::get::users_username_id:
    username,
    id,
  ))))
}

#[get("/<username>/<id>", rank = 10)]
pub fn username_id(username: String, id: PasteId) -> Redirect {
  Redirect::to(uri!(
    crate::routes::web::pastes::get::users_username_id:
    username,
    id,
  ))
}

#[get("/p/<username>/<id>")]
pub fn users_username_id(username: String, id: PasteId, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
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

  let files: Vec<OutputFile> = id.output_files(&conn, &paste, true)?;

  let mut rendered: HashMap<FileId, Option<String>> = HashMap::with_capacity(files.len());

  for file in &files {
    if let Some(ref name) = file.name {
      let lower = name.to_lowercase();
      let md_ext = file.highlight_language.is_none() && lower.ends_with(".md") || lower.ends_with(".mdown") || lower.ends_with(".markdown");
      let lang = file.highlight_language == Some(Language::Markdown.hljs());
      if !lang && !md_ext {
        rendered.insert(file.id, None);
        continue;
      }
    }
    let content = match file.content {
      Some(Content::Text(ref s)) => s,
      _ => {
        rendered.insert(file.id, None);
        continue;
      },
    };
    let md = markdown_to_html(content, &*OPTIONS);
    let cleaned = CLEANER.clean(&md).to_string();
    let processed = post_processing::process(&*config, &cleaned);
    rendered.insert(file.id, Some(processed));
  }

  let output = Output::new(
    id,
    author,
    paste.name(),
    paste.description(),
    paste.visibility(),
    paste.created_at(),
    paste.updated_at().ok(), // FIXME
    paste.expires(),
    None,
    files,
  );

  let is_owner = paste.author_id().is_some() && user.as_ref().map(|x| x.id()) == paste.author_id();

  let author_name = output.author.as_ref().map(|x| x.username.to_string()).unwrap_or_else(|| "anonymous".into());

  let mut ctx = context(&*config, user.as_ref(), &mut sess);
  ctx["paste"] = json!(output);
  ctx["num_commits"] = json!(paste.num_commits()?);
  ctx["rendered"] = json!(rendered);
  ctx["user"] = json!(*user);
  ctx["deletion_key"] = json!(sess.data.remove("deletion_key"));
  ctx["is_owner"] = json!(is_owner);
  ctx["author_name"] = json!(author_name);

  Ok(Rst::Template(Template::render("paste/index", ctx)))
}

#[get("/p/<username>/<id>/edit")]
pub fn edit(username: String, id: PasteId, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
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

  let files: Vec<OutputFile> = id.output_files(&conn, &paste, true)?;

  let output = Output::new(
    id,
    author,
    paste.name(),
    paste.description(),
    paste.visibility(),
    paste.created_at(),
    paste.updated_at().ok(), // FIXME
    paste.expires(),
    None,
    files,
  );

  let is_owner = paste.author_id().is_some() && Some(user.id()) == paste.author_id();

  let author_name = output.author.as_ref().map(|x| x.username.to_string()).unwrap_or_else(|| "anonymous".into());

  let mut ctx = context(&*config, Some(&user), &mut sess);
  ctx["paste"] = json!(output);
  ctx["languages"] = json!(Language::context());
  ctx["num_commits"] = json!(paste.num_commits()?);
  ctx["is_owner"] = json!(is_owner);
  ctx["author_name"] = json!(author_name);

  Ok(Rst::Template(Template::render("paste/edit", ctx)))
}
