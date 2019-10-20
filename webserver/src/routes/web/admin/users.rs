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

#[get("/admin/users?<page>")]
pub fn get(page: Option<u32>, config: State<Config>, user: AdminUser, mut sess: Session, conn: DbConn, langs: AcceptLanguage) -> Result<Rst> {
  const PAGE_SIZE: i64 = 15;

  let user = user.into_inner();

  let page = i64::from(page.unwrap_or(1));
  if page <= 0 {
    return Ok(Rst::Redirect(Redirect::to(uri!(get: 1))));
  }

  let total_users: i64 = users::table
    .select(count_star())
    .first(&*conn)?;
  let max_page = total_users / PAGE_SIZE + if total_users % PAGE_SIZE != 0 { 1 } else { 0 };

  if page > max_page {
    return Ok(Rst::Redirect(Redirect::to(uri!(get: max_page as u32))));
  }

  let users: Vec<User> = users::table
    .offset(PAGE_SIZE * (page - 1))
    .limit(PAGE_SIZE)
    .load(&*conn)?;

  let mut ctx = context(&*config, Some(&user), &mut sess, langs);
  ctx["links"] = json!(super::admin_links()
    .add_value("delete", users
      .iter()
      .fold(&mut Links::default(), |l, x| l.add(
        x.id().to_simple().to_string(),
        uri!(delete: x.id()),
      )))
    .add("prev", if page > 2 {
      uri!(get: page as u32 - 1)
    } else {
      uri!(get: _)
    })
    .add("next", if page < max_page {
      uri!(get: page as u32 + 1)
    } else {
      uri!(get: page as u32)
    }));
  ctx["users"] = json!(users);
  ctx["pagination"] = json!({
    "page": page,
    "max_page": max_page,
  });

  Ok(Rst::Template(Template::render("admin/users", ctx)))
}

#[delete("/admin/users/<id>", format = "application/x-www-form-urlencoded", data = "<form>")]
pub fn delete(id: UserId, form: Form<Delete>, config: State<Config>, _user: AdminUser, mut sess: Session, conn: DbConn, sidekiq: State<SidekiqClient>, l10n: L10n) -> Result<Redirect> {
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

  if target.admin() == Admin::Super {
    sess.add_data("error", l10n.tr(("admin-users-delete", "super"))?);
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
