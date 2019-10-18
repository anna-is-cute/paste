use crate::{
  config::Config,
  database::{
    DbConn,
    models::{
      deletion_keys::DeletionKey,
      pastes::Paste as DbPaste,
      users::User,
    },
    schema::{users, deletion_keys},
  },
  errors::*,
  i18n::prelude::*,
  models::{
    id::{DeletionKeyId, PasteId},
    paste::Visibility,
  },
  routes::web::{Rst, OptionalWebUser, Session},
};

use diesel::prelude::*;

use rocket::{
  http::Status as HttpStatus,
  request::{Form, State},
  response::Redirect,
};

use uuid::Uuid;

use std::str::FromStr;

#[delete("/p/<username>/<id>", format = "application/x-www-form-urlencoded", data = "<deletion>", rank = 1)]
pub fn delete(deletion: Form<PasteDeletion>, username: String, id: PasteId, config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn, l10n: L10n) -> Result<Rst> {
  let deletion = deletion.into_inner();

  if !sess.check_token(&deletion.anti_csrf_token) {
    sess.add_data("error", l10n.tr("error-csrf")?);
    return Ok(Rst::Redirect(Redirect::to("lastpage")));
  }

  let paste: DbPaste = match id.get(&conn)? {
    Some(p) => p,
    None => return Ok(Rst::Status(HttpStatus::NotFound)),
  };

  let expected_username: String = match paste.author_id() {
    Some(author) => {
      let user: User = users::table.find(author).first(&*conn)?;
      user.username().to_string()
    },
    None => "anonymous".into(),
  };

  if username != expected_username {
    return Ok(Rst::Status(HttpStatus::NotFound));
  }

  if let Some((status, _)) = paste.check_access(user.as_ref().map(|x| x.id())) {
    return Ok(Rst::Status(status));
  }

  match paste.author_id() {
    Some(author) => if Some(author) != user.as_ref().map(|x| x.id()) {
      if paste.visibility() == Visibility::Private {
        return Ok(Rst::Status(HttpStatus::NotFound));
      } else {
        return Ok(Rst::Status(HttpStatus::Forbidden));
      }
    },
    None => {
      let key = match deletion.key {
        Some(k) => k,
        None => {
          sess.add_data("error", "Anonymous pastes require a deletion key to delete.");
          return Ok(Rst::Redirect(Redirect::to("lastpage")));
        },
      };

      let key = match Uuid::from_str(&key) {
        Ok(k) => DeletionKeyId(k),
        Err(_) => {
          sess.add_data("error", "Invalid deletion key.");
          return Ok(Rst::Redirect(Redirect::to("lastpage")));
        },
      };

      let db_key: DeletionKey = match deletion_keys::table.find(&key).first(&*conn).optional()? {
        Some(k) => k,
        None => {
          sess.add_data("error", "Invalid deletion key.");
          return Ok(Rst::Redirect(Redirect::to("lastpage")));
        },
      };

      if db_key.paste_id() != paste.id() {
        sess.add_data("error", "Invalid deletion key.");
        return Ok(Rst::Redirect(Redirect::to("lastpage")));
      }
    },
  }

  // should be authed beyond this point

  paste.delete(&*config, &conn)?;

  sess.add_data("info", "Paste deleted.");
  Ok(Rst::Redirect(Redirect::to("/")))
}

#[derive(Debug, FromForm)]
pub struct PasteDeletion {
  key: Option<String>,
  anti_csrf_token: String,
}

#[delete("/p/<username>/ids", format = "application/x-www-form-urlencoded", data = "<deletion>", rank = 2)]
pub fn ids(deletion: Form<MultiPasteDeletion>, username: String, user: OptionalWebUser, mut sess: Session, conn: DbConn, config: State<Config>, l10n: L10n) -> Result<Rst> {
  let deletion = deletion.into_inner();

  if !sess.check_token(&deletion.anti_csrf_token) {
    sess.add_data("error", l10n.tr("error-csrf")?);
    return Ok(Rst::Redirect(Redirect::to("lastpage")));
  }

  let user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  if username != user.username() {
    sess.add_data("error", "You cannot delete pastes for other users.");
    return Ok(Rst::Redirect(Redirect::to("lastpage")));
  }

  let ids: Vec<PasteId> = serde_json::from_str(&deletion.ids)?;

  if ids.len() > 15 {
    sess.add_data("error", "Up to 15 pastes can be deleted at a time.");
    return Ok(Rst::Redirect(Redirect::to("lastpage")));
  }

  let err = "No pastes were deleted because an invalid paste was specified. Perhaps it was deleted already?";
  let mut pastes = Vec::with_capacity(ids.len());
  for id in ids {
    let paste: DbPaste = match id.get(&conn)? {
      Some(p) => p,
      None => {
        sess.add_data("error", err);
        return Ok(Rst::Redirect(Redirect::to("lastpage")));
      },
    };
    if paste.author_id() != Some(user.id()) {
      // no special error for this because it could lead to leaking private pastes
      sess.add_data("error", err);
      return Ok(Rst::Redirect(Redirect::to("lastpage")));
    }
    pastes.push(paste);
  }

  for paste in &pastes {
    paste.delete(&*config, &conn)?;
  }

  sess.add_data("info", format!("{} paste{} deleted.", pastes.len(), if pastes.len() == 1 { "" } else { "s" }));
  Ok(Rst::Redirect(Redirect::to("lastpage")))
}

#[derive(Debug, FromForm)]
pub struct MultiPasteDeletion {
  anti_csrf_token: String,
  ids: String,
}
