use database::{DbConn, schema};
use database::schema::pastes;
use database::models::pastes::{NewPaste, Paste as DbPaste};
use database::models::deletion_keys::NewDeletionKey;
use database::models::files::NewFile;
use models::paste::{Paste, Visibility, PasteId, Content};
use models::paste::update::PasteUpdate;
use models::status::{Status, ErrorKind};
use routes::{RouteResult, RequiredUser};
use store::Store;

use diesel;
use diesel::prelude::*;

use git2::{Repository, Signature};

use rocket::http::Status as HttpStatus;

use rocket_contrib::Json;

use uuid::Uuid;

use std::fs::File;
use std::io::Write;

type UpdateResult = ::std::result::Result<Json<PasteUpdate>, ::rocket_contrib::SerdeError>;

#[patch("/<id>", format = "application/json", data = "<info>")]
pub fn patch(id: PasteId, info: UpdateResult, user: RequiredUser, conn: DbConn) -> RouteResult<()> {
  // TODO: can this be a request guard?
  let info = match info {
    Ok(x) => x.into_inner(),
    Err(e) => {
      let message = format!("could not parse json: {}", e);
      return Ok(Status::show_error(HttpStatus::BadRequest, ErrorKind::BadJson(Some(message))));
    },
  };

  // verify auth
  let mut paste: DbPaste = match pastes::table.find(*id).first(&*conn).optional()? {
    Some(p) => p,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };
  if *paste.author_id() != Some(user.id()) {
    return Ok(if paste.visibility() == Visibility::Private {
      Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)
    } else {
      Status::show_error(HttpStatus::Forbidden, ErrorKind::NotAllowed)
    });
  }

  // update paste and database if necessary
  let mut changed = false;
  if let Some(update) = info.metadata.name {
    paste.set_name(update);
    changed = true;
  }
  if let Some(update) = info.metadata.visibility {
    paste.set_visibility(update);
    changed = true;
  }
  if changed {
    diesel::update(pastes::table).set(&paste).execute(&*conn)?;
    changed = false;
  }

  // update files and database if necessary
  if let Some(files) = info.files {
    // TODO:
    // delete all old files (but not files dir, since repo is there)
    // update database
    // write new files
    // update database
  }

  // commit if any files were changed
  if changed {
    let paste_id = PasteId(paste.id());
    let repo = Repository::open(paste_id.files_directory())?;
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let head_id = repo.refname_to_id("HEAD")?;
    let parent = repo.find_commit(head_id)?;
    let username = user.username().as_str();
    // TODO: figure out what email should be
    let sig = Signature::now(username, &format!("{}@paste.com", username))?;
    // TODO: more descriptive commit name?
    repo.commit(Some("HEAD"), &sig, &sig, "update paste", &tree, &[&parent])?;
  }

  // return status (204?)
  Ok(Status::show_success(HttpStatus::NoContent, ()))
}

/* commit in C (https://stackoverflow.com/questions/27672722/libgit2-commit-example)
git_oid tree_id, parent_id, commit_id;
git_tree *tree;
git_commit *parent;
git_index *index;

/* Get the index and write it to a tree */
git_repository_index(&index, repo);
git_index_write_tree(tree, index);

/* Get HEAD as a commit object to use as the parent of the commit */
git_reference_name_to_id(parent_id, repo, "HEAD");
git_commit_lookup(&parent, repo, parent_id);

/* Do the commit */
git_commit_create_v(
    commit_id,
    repo,
    "HEAD",     /* The commit will update the position of HEAD */
    author,
    committer,
    NULL,       /* UTF-8 encoding */
    message,
    tree,       /* The tree from the index */
    1,          /* Only one parent */
    parent      /* No need to make a list with create_v */
);
*/
