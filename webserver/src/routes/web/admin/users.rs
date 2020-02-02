use crate::{
  config::Config,
  database::{
    DbConn,
    models::users::User,
    schema::users,
  },
  errors::*,
  i18n::prelude::*,
  models::{
    id::UserId,
    user::Admin,
  },
  routes::web::{context, Links, Rst, Session},
  sidekiq::Job,
  utils::AcceptLanguage,
};

use super::AdminUser;

use diesel::{
  dsl::count_star,
  prelude::*,
};

use rocket::{
  State,
  request::Form,
  response::Redirect,
};

use rocket_contrib::templates::Template;

use serde_json::json;

use sidekiq::Client as SidekiqClient;

use std::collections::HashMap;

#[get("/admin/users?<page>")]
pub fn get(page: Option<u32>, config: State<Config>, user: AdminUser, mut sess: Session, conn: DbConn, langs: AcceptLanguage) -> Result<Rst> {
  const PAGE_SIZE: i64 = 15;

  let user = user.into_inner();

  // get the page number or 1 by default
  let page = i64::from(page.unwrap_or(1));
  // redirect to page 1 if page was 0
  if page <= 0 {
    return Ok(Rst::Redirect(Redirect::to(uri!(get: _))));
  }

  // count all users
  let total_users: i64 = users::table
    .select(count_star())
    .first(&*conn)?;
  // determine the max page
  let max_page = total_users / PAGE_SIZE + if total_users % PAGE_SIZE != 0 { 1 } else { 0 };

  // redirect to the max page if requested page was too high
  if page > max_page {
    return Ok(Rst::Redirect(Redirect::to(uri!(get: max_page as u32))));
  }

  // get all users for the page ordered by admin status (users have no created_at field)
  let users: Vec<User> = users::table
    .order_by((
      users::admin.desc(),
      users::username.asc(),
    ))
    .offset(PAGE_SIZE * (page - 1))
    .limit(PAGE_SIZE)
    .load(&*conn)?;

  // create the default context
  let mut ctx = context(&*config, Some(&user), &mut sess, langs);
  // add 2fa statuses
  ctx["tfa"] = json!(users.iter()
    .map(|u| (u.id(), u.tfa_enabled()))
    .collect::<HashMap<UserId, bool>>());
  // add some links
  ctx["links"] = json!(super::admin_links()
    // add the deletion endpoints
    .add_value("delete", users
      .iter()
      .fold(&mut Links::default(), |l, x| l.add(
        x.id().to_simple().to_string(),
        uri!(delete: x.id()),
      )))
    // add the promotion endpoints
    .add_value("promote", users
      .iter()
      .fold(&mut Links::default(), |l, x| l.add(
        x.id().to_simple().to_string(),
        uri!(promote: x.id()),
      )))
    // add the demotion endpoints
    .add_value("demote", users
      .iter()
      .fold(&mut Links::default(), |l, x| l.add(
        x.id().to_simple().to_string(),
        uri!(demote: x.id()),
      )))
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
  // add the users
  ctx["users"] = json!(users);
  // add pagination info
  ctx["pagination"] = json!({
    "page": page,
    "max_page": max_page,
  });

  // render the template
  Ok(Rst::Template(Template::render("admin/users", ctx)))
}

#[delete("/admin/users/<id>", format = "application/x-www-form-urlencoded", data = "<form>")]
pub fn delete(id: UserId, form: Form<Delete>, config: State<Config>, user: AdminUser, mut sess: Session, conn: DbConn, sidekiq: State<SidekiqClient>, l10n: L10n) -> Result<Redirect> {
  // check the anti csrf token
  if !sess.check_token(&form.anti_csrf_token) {
    sess.add_data("error", l10n.tr("error-csrf")?);
    return Ok(Redirect::to("lastpage"));
  }

  // get the user from the id
  let target = match id.get(&conn)? {
    Some(u) => u,
    None => {
      sess.add_data("error", l10n.tr(("admin-users-delete", "missing"))?);
      return Ok(Redirect::to("lastpage"));
    },
  };

  // can't delete superadmins
  if target.admin() == Admin::Super {
    sess.add_data("error", l10n.tr(("admin-users-delete", "super"))?);
    return Ok(Redirect::to("lastpage"));
  }

  // admins can't delete other admins (but can delete themselves)
  if user.admin() == Admin::Normal && target.admin() == Admin::Normal && user.id() != target.id() {
    sess.add_data("error", l10n.tr(("admin-users-delete", "other-admin"))?);
    return Ok(Redirect::to("lastpage"));
  }

  // delete the user
  target.delete(&conn)?;

  // add a job to delete all their pastes
  sidekiq.push(Job::DeleteAllPastes(&*config, target.id()).into())?;

  // add a notification of success
  sess.add_data("info", l10n.tr(("admin-users-delete", "success"))?);

  // go back
  Ok(Redirect::to("lastpage"))
}

#[derive(FromForm)]
pub struct Delete {
  pub anti_csrf_token: String,
}

#[patch("/admin/user/<id>/promote", format = "application/x-www-form-urlencoded", data = "<form>")]
pub fn promote(id: UserId, form: Form<Promote>, user: AdminUser, mut sess: Session, conn: DbConn, l10n: L10n) -> Result<Redirect> {
  // check anti-csrf token
  if !sess.check_token(&form.anti_csrf_token) {
    sess.add_data("error", l10n.tr("error-csrf")?);
    return Ok(Redirect::to("lastpage"));
  }

  // ensure the user is a superadmin
  if user.admin() != Admin::Super {
    sess.add_data("error", l10n.tr(("admin-users-status", "must-be-super"))?);
    return Ok(Redirect::to("lastpage"));
  }

  // determine the level of the promotion
  let level = match &*form.level {
    "admin" => Admin::Normal,
    "super" => Admin::Super,
    _ => {
      sess.add_data("error", l10n.tr(("admin-users-status", "invalid-level"))?);
      return Ok(Redirect::to("lastpage"));
    },
  };

  // get the target user
  let mut target = match id.get(&conn)? {
    Some(u) => u,
    None => {
      sess.add_data("error", l10n.tr(("admin-users-status", "missing"))?);
      return Ok(Redirect::to("lastpage"));
    },
  };

  match target.admin() {
    // if the target is a super admin, let the user know that superadmins are untouchable
    Admin::Super => {
      sess.add_data("error", l10n.tr(("admin-users-status", "target-super"))?);
    },
    // if the target is a normal admin, tell the user
    Admin::Normal => {
      sess.add_data("error", l10n.tr(("admin-users-status", "already-admin"))?);
    },
    // if the target is not an admin, promote them
    Admin::None => {
      target.set_admin(level);
      target.update(&conn)?;
      sess.add_data("info", l10n.tr(("admin-users-status", "promoted"))?);
    },
  }

  // go back
  Ok(Redirect::to("lastpage"))
}

#[derive(FromForm)]
pub struct Promote {
  pub anti_csrf_token: String,
  pub level: String,
}

#[patch("/admin/user/<id>/demote", format = "application/x-www-form-urlencoded", data = "<form>")]
pub fn demote(id: UserId, form: Form<Demote>, user: AdminUser, mut sess: Session, conn: DbConn, l10n: L10n) -> Result<Redirect> {
  // check anti-csrf token
  if !sess.check_token(&form.anti_csrf_token) {
    sess.add_data("error", l10n.tr("error-csrf")?);
    return Ok(Redirect::to("lastpage"));
  }

  // ensure the user is a superadmin
  if user.admin() != Admin::Super {
    sess.add_data("error", l10n.tr(("admin-users-status", "must-be-super"))?);
    return Ok(Redirect::to("lastpage"));
  }

  // get the target user
  let mut target = match id.get(&conn)? {
    Some(u) => u,
    None => {
      sess.add_data("error", l10n.tr(("admin-users-status", "missing"))?);
      return Ok(Redirect::to("lastpage"));
    },
  };

  match target.admin() {
    // if the target is a superadmin, let the user know they are untouchable
    Admin::Super => {
      sess.add_data("error", l10n.tr(("admin-users-status", "target-super"))?);
    },
    // if the target is a normal admin, demote them
    Admin::Normal => {
      target.set_admin(Admin::None);
      target.update(&conn)?;
      sess.add_data("info", l10n.tr(("admin-users-status", "demoted"))?);
    },
    // if the target is not an admin, tell the user
    Admin::None => {
      sess.add_data("error", l10n.tr(("admin-users-status", "not-admin"))?);
    },
  }

  // go back
  Ok(Redirect::to("lastpage"))
}

#[derive(FromForm)]
pub struct Demote {
  pub anti_csrf_token: String,
}
