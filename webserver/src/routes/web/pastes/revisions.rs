use config::Config;
use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::models::users::User;
use database::schema::users;
use errors::*;
use models::id::PasteId;
use models::paste::output::{Output, OutputAuthor, OutputFile};
use routes::web::{context, Rst, OptionalWebUser, Session};

use diesel::prelude::*;

use git2::{Repository, DiffFormat, Oid, Commit, Tree};

use rocket::http::Status as HttpStatus;
use rocket::State;

use rocket_contrib::Template;

use std::result;

#[get("/pastes/<username>/<id>/revisions")]
fn get(username: String, id: PasteId, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Rst> {
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

  let repo = Repository::open(paste.files_directory())?;
  let head = repo.refname_to_id("HEAD")?;
  let head_commit = repo.find_commit(head)?;

  let mut count = 1;

  let mut diffs = Vec::new();
  let mut commit = head_commit;
  loop {
    let parent = match commit.parent(0) {
      Ok(p) => {
        count += 1;
        DiffArg::Commit(p)
      },
      Err(_) => DiffArg::Tree(
        // FIXME: this is using the sha1 of the empty tree, which all repos have, but there must be
        //        a better way
        repo.find_tree(Oid::from_str("4b825dc642cb6eb9a060e54bf8d69288fbee4904")?)?,
      ),
    };

    let parent_tree = match parent {
      DiffArg::Commit(ref c) => c.tree()?,
      DiffArg::Tree(ref t) => t.clone(),
    };

    let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit.tree()?), None)?;
    let mut diff_str = String::new();
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
      match line.origin() {
        '+' | '-' | ' ' => diff_str.push(line.origin()),
        _ => {}
      }
      diff_str += ::std::str::from_utf8(line.content()).unwrap();
      true
    })?;

    diffs.push(diff_str);

    match parent {
      DiffArg::Commit(c) => commit = c,
      DiffArg::Tree(_) => break,
    }
  }

  let files: Vec<OutputFile> = id.files(&conn)?
    .iter()
    .map(|x| x.as_output_file(false, &paste))
    .collect::<result::Result<_, _>>()?;

  let output = Output::new(
    id,
    author,
    paste.name(),
    paste.description(),
    paste.visibility(),
    paste.created_at(),
    None,
    files,
  );

  let author_name = output.author.as_ref().map(|x| x.username.to_string()).unwrap_or_else(|| "anonymous".into());

  let mut ctx = context(&*config, user.into_inner().as_ref(), &mut sess);
  ctx["paste"] = json!(output);
  ctx["num_commits"] = json!(count);
  ctx["author_name"] = json!(author_name);
  ctx["diffs"] = json!(diffs);

  Ok(Rst::Template(Template::render("paste/revisions", ctx)))
}

enum DiffArg<'repo> {
  Commit(Commit<'repo>),
  Tree(Tree<'repo>),
}
