use crate::{
  config::Config,
  database::{
    DbConn,
    models::{pastes::Paste as DbPaste, users::User},
    schema::users,
  },
  errors::*,
  models::{
    id::PasteId,
    paste::output::{Output, OutputAuthor, OutputFile},
  },
  routes::web::{context, Rst, OptionalWebUser, Session},
};

use diesel::prelude::*;

use git2::{Repository, DiffFormat, Oid, Commit, Tree};

use rocket::{State, http::Status as HttpStatus};

use rocket_contrib::Template;

use serde_json::json;

#[get("/p/<username>/<id>/revisions")]
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

  let files: Vec<OutputFile> = id.output_files(&conn, &paste, false)?;

  let repo = Repository::open(paste.files_directory())?;
  let head = repo.refname_to_id("HEAD")?;
  let head_commit = repo.find_commit(head)?;

  let mut count = 1;

  let mut all_revisions: Vec<Vec<Revision>> = Vec::default();
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

    let mut revisions = Vec::default();

    let mut revision = Revision::default();
    let mut hunk = Hunk::default();

    diff.print(DiffFormat::Patch, |delta, _hunk, line| {
      let line_str = ::std::str::from_utf8(line.content()).unwrap();
      match line.origin() {
        '+' | '-' | ' ' => hunk.diff.push(line.origin()),
        '=' | '>' | '<' => {},
        'F' => {
          let name = delta.new_file().path()
            .or_else(|| delta.old_file().path())
            .unwrap()
            .to_string_lossy()
            .to_string();

          if revision.id.is_some() && revision.id.as_ref() != Some(&name) {
            revision.hunks.push(hunk.clone());
            hunk = Hunk::default();
            revisions.push(revision.clone());
            revision = Revision::default();
          }

          revision.id = Some(name.clone());

          let name = files
            .iter()
            .find(|x| x.id.simple().to_string() == name)
            .and_then(|x| x.name.as_ref())
            .cloned();
          revision.file_name = name;
          return true;
        },
        'H' => {
          if hunk.header.is_some() {
            revision.hunks.push(hunk.clone());
            hunk = Hunk::default();
          }
          hunk.header = Some(line_str.to_string());
          return true;
        },
        _ => return true,
      }
      hunk.diff += line_str;
      true
    })?;

    revision.hunks.push(hunk);
    revisions.push(revision);

    all_revisions.push(revisions);

    match parent {
      DiffArg::Commit(c) => commit = c,
      DiffArg::Tree(_) => break,
    }
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

  let author_name = output.author.as_ref().map(|x| x.username.to_string()).unwrap_or_else(|| "anonymous".into());

  let mut ctx = context(&*config, user.into_inner().as_ref(), &mut sess);
  ctx["paste"] = json!(output);
  ctx["num_commits"] = json!(count);
  ctx["author_name"] = json!(author_name);
  ctx["revisions"] = json!(all_revisions);

  Ok(Rst::Template(Template::render("paste/revisions", ctx)))
}

enum DiffArg<'repo> {
  Commit(Commit<'repo>),
  Tree(Tree<'repo>),
}

#[derive(Debug, Serialize, Default, Clone)]
struct Revision {
  id: Option<String>,
  file_name: Option<String>,
  hunks: Vec<Hunk>,
}

#[derive(Debug, Serialize, Default, Clone)]
struct Hunk {
  header: Option<String>,
  diff: String,
}
