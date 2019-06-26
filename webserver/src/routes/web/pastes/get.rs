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
  utils::{
    csv::csv_to_table,
    post_processing,
    Language,
  },
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

use rouge::{Rouge, HighlightKind};

use serde_json::json;

lazy_static! {
  static ref OPTIONS: ComrakOptions = ComrakOptions {
    github_pre_lang: true,
    ext_strikethrough: true,
    ext_table: true,
    ext_autolink: true,
    ext_tasklist: true,
    ext_footnotes: true,
    ext_highlight: true,
    // allows html and bad links: ammonia + our post-processor cleans the output, not comrak
    unsafe_: true,
    .. Default::default()
  };

  static ref CLEANER: Builder<'static> = {
    let mut b = Builder::default();
    b
      .link_rel(Some("noopener noreferrer nofollow"))
      .add_tags(std::iter::once("input"))
      .add_tag_attribute_values("input", "checked", vec!["", "checked"])
      .add_tag_attribute_values("input", "disabled", vec!["", "disabled"])
      .add_tag_attribute_values("input", "type", std::iter::once("checkbox"))
      .add_allowed_classes("span", vec![
        "hll", "c", "err", "k", "l", "n", "o", "p", "cm", "cp", "c1", "cs", "gd", "ge", "gh", "gi",
        "gp", "gs", "gu", "kc", "kd", "kn", "kp", "kr", "kt", "ld", "m", "s", "na", "nb", "nc",
        "no", "nd", "ni", "ne", "nf", "nl", "nn", "nx", "py", "nt", "nv", "ow", "w", "mf", "mh",
        "mi", "mo", "sb", "sc", "sd", "s2", "se", "sh", "si", "sx", "sr", "s1", "ss", "bp", "vc",
        "vg", "vi", "il",
      ]);
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

  let mut files: Vec<OutputFile> = id.output_files(&*config, &conn, &paste, true)?;

  let mut lines: HashMap<FileId, Vec<String>> = HashMap::with_capacity(files.len());
  let mut rendered: HashMap<FileId, String> = HashMap::with_capacity(files.len());
  let mut notices: HashMap<FileId, String> = HashMap::new();

  for file in &mut files {
    if let Some(ref name) = file.name {
      let lower = name.to_lowercase();

      let md_ext = file.highlight_language.is_none() && lower.ends_with(".md") || lower.ends_with(".mdown") || lower.ends_with(".markdown");
      let md_lang = file.highlight_language == Some(Language::Markdown.hljs());
      let is_md = md_ext || md_lang;

      let is_csv = file.highlight_language.is_none() && lower.ends_with(".csv");

      let content = match file.content {
        Some(Content::Text(ref s)) => s,
        _ => continue,
      };

      let processed = if is_md {
        let md = markdown_to_html(content, &*OPTIONS);
        let cleaned = CLEANER.clean(&md).to_string();
        Some(post_processing::process(&*config, &cleaned))
      } else if is_csv {
        match csv_to_table(content) {
          Ok(h) => Some(h),
          Err(e) => {
            notices.insert(file.id, e);
            None
          },
        }
      } else {
        None
      };

      let highlighted = Rouge::highlight_lines(HighlightKind::File, &name, &content)?;
      lines.insert(file.id, highlighted);

      if let Some(processed) = processed {
        rendered.insert(file.id, processed);
      }
    }
  }

  let output = Output::new(
    id,
    author,
    paste.name(),
    paste.description(),
    paste.visibility(),
    paste.created_at(),
    paste.updated_at(&*config).ok(), // FIXME
    paste.expires(),
    None,
    files,
  );

  let is_owner = paste.author_id().is_some() && user.as_ref().map(|x| x.id()) == paste.author_id();

  let author_name = output.author.as_ref().map(|x| x.username.to_string()).unwrap_or_else(|| "anonymous".into());

  let mut links = super::paste_links(paste.id(), paste.author_id(), &author_name, user.as_ref());
  links.add_value(
    "raw_files",
    output
      .files
      .iter()
      .fold(&mut crate::routes::web::Links::default(), |acc, x| acc.add(
        x.id.to_simple().to_string(),
        uri!(crate::routes::web::pastes::files::raw::get: &author_name, paste.id(), x.id),
      )),
  );

  let mut ctx = context(&*config, user.as_ref(), &mut sess);
  ctx["paste"] = json!(output);
  ctx["num_commits"] = json!(paste.num_commits(&*config)?);
  ctx["rendered"] = json!(rendered);
  ctx["lines"] = json!(lines);
  ctx["notices"] = json!(notices);
  ctx["user"] = json!(*user);
  ctx["deletion_key"] = json!(sess.data.remove(&format!("deletion_key_{}", paste.id().to_simple())));
  ctx["is_owner"] = json!(is_owner);
  ctx["author_name"] = json!(author_name);
  ctx["links"] = json!(links);

  Ok(Rst::Template(Template::render("paste/index", ctx)))
}

#[get("/p/<username>/<id>/edit")]
pub fn edit(username: String, id: PasteId, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Rst::Redirect(Redirect::to(uri!(crate::routes::web::auth::login::get)))),
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

  let files: Vec<OutputFile> = id.output_files(&*config, &conn, &paste, true)?;

  let output = Output::new(
    id,
    author,
    paste.name(),
    paste.description(),
    paste.visibility(),
    paste.created_at(),
    paste.updated_at(&*config).ok(), // FIXME
    paste.expires(),
    None,
    files,
  );

  let is_owner = paste.author_id().is_some() && Some(user.id()) == paste.author_id();

  let author_name = output.author.as_ref().map(|x| x.username.to_string()).unwrap_or_else(|| "anonymous".into());

  let mut ctx = context(&*config, Some(&user), &mut sess);
  ctx["paste"] = json!(output);
  ctx["languages"] = json!(Language::context());
  ctx["num_commits"] = json!(paste.num_commits(&*config)?);
  ctx["is_owner"] = json!(is_owner);
  ctx["author_name"] = json!(author_name);
  ctx["links"] = json!(
    super::paste_links(paste.id(), paste.author_id(), &author_name, Some(&user))
      .add(
        "patch",
        uri!(crate::routes::web::pastes::patch::patch: &author_name, paste.id()),
      )
  );

  Ok(Rst::Template(Template::render("paste/edit", ctx)))
}
