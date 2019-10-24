use crate::{
  config::Config,
  database::{
    DbConn,
    models::{
      pastes::Paste as DbPaste,
      users::User,
    },
    schema::{files, pastes, users},
  },
  errors::*,
  i18n::prelude::*,
  models::{
    paste::output::{Output, OutputAuthor},
    id::PasteId,
  },
  routes::web::{context, Links, Rst, Session},
  utils::AcceptLanguage,
};

use super::AdminUser;

use diesel::{
  dsl::count_star,
  prelude::*,
};

use rocket::{
  request::Form,
  response::Redirect,
  State,
};

use rocket_contrib::templates::Template;

use serde_json::json;

use uuid::Uuid;

#[get("/admin/pastes?<page>")]
pub fn get(page: Option<u32>, config: State<Config>, user: AdminUser, mut sess: Session, conn: DbConn, langs: AcceptLanguage) -> Result<Rst> {
  const PAGE_SIZE: i64 = 15;

  let user = user.into_inner();

  // get the page number or 1 by default
  let page = i64::from(page.unwrap_or(1));
  // redirect to first page if given page 0
  if page <= 0 {
    return Ok(Rst::Redirect(Redirect::to(uri!(get: _))));
  }

  // get the total number of pastes
  let total_pastes: i64 = pastes::table
    .select(count_star())
    .first(&*conn)?;
  // determine the max page
  let max_page = total_pastes / PAGE_SIZE + if total_pastes % PAGE_SIZE != 0 { 1 } else { 0 };

  // if requested a page greater than the max page, redirect to the max page
  if page > max_page {
    return Ok(Rst::Redirect(Redirect::to(uri!(get: max_page as u32))));
  }

  // get all pastes for that page with their respective authors
  let pastes: Vec<(DbPaste, i64, Option<User>)> = pastes::table
    .left_join(users::table)
    .inner_join(files::table)
    .select((
      pastes::table::all_columns(),
      diesel::dsl::sql::<diesel::sql_types::BigInt>("count(files.id)"),
      users::table::all_columns().nullable(),
    ))
    .group_by((
      pastes::table::all_columns(),
      users::table::all_columns(),
    ))
    .order_by(pastes::created_at.desc())
    .offset(PAGE_SIZE * (page - 1))
    .limit(PAGE_SIZE)
    .load(&*conn)?;

  // convert pastes into paste outputs
  let outputs: Vec<(Output, i64)> = pastes
    .into_iter()
    .map(|(paste, file_count, user)| (
      Output::new(
        paste.id(),
        user.map(|user| OutputAuthor::new(user.id(), user.username(), user.name())),
        paste.name(),
        paste.description(),
        paste.visibility(),
        paste.created_at(),
        paste.updated_at(&*config).ok(),
        paste.expires(),
        None,
        Vec::new(),
      ),
      file_count,
    ))
    .collect();

  // create default context
  let mut ctx = context(&*config, Some(&user), &mut sess, langs);
  // add links to context
  ctx["links"] = json!(super::admin_links()
    // add links to each paste
    .add_value("paste_links", outputs
      .iter()
      .fold(&mut Links::default(), |l, (x, _)| l.add(
        x.id.to_simple().to_string(),
        uri!(
          crate::routes::web::pastes::get::users_username_id:
          x.author.as_ref().map(|x| x.username.as_str()).unwrap_or("anonymous"),
          x.id,
        ),
      )))
    // add deletion links for each paste
    .add_value("delete", outputs
      .iter()
      .fold(&mut Links::default(), |l, (x, _)| l.add(
        x.id.to_simple().to_string(),
        uri!(delete: x.id),
      )))
    // add the batch delete endpoint
    .add("batch_delete", uri!(batch_delete))
    // add the previous page link
    .add("prev", if page > 2 {
      uri!(get: page as u32 - 1)
    } else {
      uri!(get: _)
    })
    // add the next page link
    .add("next", if page < max_page {
      uri!(get: page as u32 + 1)
    } else {
      uri!(get: page as u32)
    }));
  // add the pastes
  ctx["pastes"] = json!(outputs);
  // add pagination info
  ctx["pagination"] = json!({
    "page": page,
    "max_page": max_page,
  });

  // render the template
  Ok(Rst::Template(Template::render("admin/pastes", ctx)))
}

#[post("/admin/batch_delete", format = "application/x-www-form-urlencoded", data = "<ids>")]
pub fn batch_delete(ids: Form<BatchDelete>, config: State<Config>, _user: AdminUser, mut sess: Session, conn: DbConn, l10n: L10n) -> Result<Redirect> {
  // set the form in the session for restoring on error
  sess.set_form(&*ids);

  // check the anti csrf token
  if !sess.check_token(&ids.anti_csrf_token) {
    sess.add_data("error", l10n.tr("error-csrf")?);
    return Ok(Redirect::to("lastpage"));
  }

  // trim each line, take the last segment of a '/' split, parse it as a uuid, then map it to a PasteId
  let ids: Result<Vec<PasteId>> = ids.ids
    .lines()
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .flat_map(|i| i.split('/').last())
    .map(Uuid::parse_str)
    .map(|u| u.map(PasteId).map_err(Into::into))
    .collect();

  // let the user know why a uuid couldn't be parsed, if any
  let mut ids = match ids {
    Ok(i) => i,
    Err(e) => {
      sess.add_data("error", format!("Invalid ID: {}.", e));
      return Ok(Redirect::to("lastpage"));
    },
  };
  // sort the ids
  ids.sort_unstable();
  // remove any duplicate ids
  ids.dedup();

  fn delete_paste(id: PasteId, config: &Config, conn: &DbConn, l10n: &L10n) -> Result<()> {
    let paste = match id.get(&*conn)? {
      Some(p) => p,
      None => anyhow::bail!(l10n.tr("admin-batch-delete-missing")?),
    };
    paste.delete(config, conn)
  }

  // collect a list of errors encountered while deleting
  let errors: Vec<String> = ids
    .iter()
    // map each id to a tuple of id to deletion result
    .map(|&id| (id, delete_paste(id, &config, &conn, &l10n)))
    // only keep the errors
    .flat_map(|(id, res)| res.err().map(|e| (id, e)))
    // format the error
    .map(|(id, e)| l10n.tr_ex(
      ("admin-batch-delete", "error"),
      |req| req
        .arg_str("id", id)
        .arg_str("error", e),
      ))
    .collect::<Result<_>>()?;

  // determine how many pastes were deleted
  let deleted = ids.len() - errors.len();
  // add a notification if any were deleted
  if errors.is_empty() || deleted > 0 {
    sess.add_data(
      "info",
      l10n.tr_ex(
        ("admin-batch-delete", "success"),
        |req| req.arg_num("pastes", deleted),
      )?,
    );
  }
  // remove the form from the session if all were successful
  if errors.is_empty() {
    sess.take_form();
  } else {
    // otherwise add a HTML error message
    sess.add_data("error_safe", format!("<p>{}</p>", errors.join("</p><p>")));
  }

  // redirect back to the pastes page
  Ok(Redirect::to("lastpage"))
}

#[derive(FromForm, Serialize)]
pub struct BatchDelete {
  #[serde(skip)]
  pub anti_csrf_token: String,
  pub ids: String,
}

#[delete("/admin/pastes/<id>", format = "application/x-www-form-urlencoded", data = "<form>")]
pub fn delete(id: PasteId, form: Form<Delete>, config: State<Config>, _user: AdminUser, mut sess: Session, conn: DbConn, l10n: L10n) -> Result<Redirect> {
  // check the anti csrf token
  if !sess.check_token(&form.anti_csrf_token) {
    sess.add_data("error", l10n.tr("error-csrf")?);
    return Ok(Redirect::to("lastpage"));
  }

  // get the paste from the id
  let paste = match id.get(&conn)? {
    Some(p) => p,
    None => {
      sess.add_data("error", l10n.tr(("admin-paste-delete", "missing"))?);
      return Ok(Redirect::to("lastpage"));
    },
  };

  // delete the paste
  paste.delete(&config, &conn)?;

  // add notification
  sess.add_data("info", l10n.tr(("admin-paste-delete", "success"))?);

  // redirect back
  Ok(Redirect::to("lastpage"))
}

#[derive(FromForm)]
pub struct Delete {
  pub anti_csrf_token: String,
}
