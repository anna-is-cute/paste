use crate::{
  config::{AppConfig, Config, initialise},
  errors::*,
  i18n::prelude::*,
  models::user::Admin,
  routes::web::{context, Rst, Session},
  utils::AcceptLanguage,
};

use super::AdminUser;

use rocket::{
  State,
  request::Form,
  response::Redirect,
};

use rocket_contrib::templates::Template;

use serde_json::json;

use std::{
  fs::File,
  io::Write,
};

#[get("/admin/config")]
pub fn get(config: State<Config>, user: AdminUser, mut sess: Session, langs: AcceptLanguage) -> Result<Rst> {
  // check if normal admins are allowed to edit config and redirect to admin overview if not
  if user.admin() == Admin::Normal && !config.read().admin.admins_can_edit_config {
    return Ok(Rst::Redirect(Redirect::to(uri!(super::index::get))));
  }

  let user = user.into_inner();

  // read in the config file
  let config_str = std::fs::read_to_string(config.read()._path.as_ref().unwrap())?;
  // create the default context
  let mut ctx = context(&*config, Some(&user), &mut sess, langs);
  // add admin links and the upload endpoint
  ctx["links"] = json!(super::admin_links()
    .add_value("upload", uri!(post).to_string()));
  // add the config file text
  ctx["config_text"] = json!(config_str);

  // render the template
  Ok(Rst::Template(Template::render("admin/config", ctx)))
}

#[post("/admin/config", format = "application/x-www-form-urlencoded", data = "<update>")]
pub fn post(update: Form<ConfigUpdate>, config: State<Config>, user: AdminUser, mut sess: Session, l10n: L10n) -> Result<Redirect> {
  // check the anti-csrf token
  if !sess.check_token(&update.anti_csrf_token) {
    sess.add_data("error", l10n.tr("error-csrf")?);
    return Ok(Redirect::to(uri!(get)));
  }

  // check if admins are allowed to edit the config and redirect to admin overview if not
  if user.admin() == Admin::Normal && !config.read().admin.admins_can_edit_config {
    return Ok(Redirect::to(uri!(super::index::get)));
  }

  // grab the path to the config
  let path = config.read()._path.clone().unwrap();
  // deserialise the new config
  let new_config: AppConfig = match toml::from_str(&update.config) {
    Ok(c) => c,
    Err(e) => {
      sess.add_data("error", format!("Could not parse config as valid TOML: {}.", e));
      return Ok(Redirect::to(uri!(get)));
    },
  };
  // update the config
  *config.write() = match initialise(new_config, &path.to_string_lossy()) {
    Ok(c) => c,
    Err(e) => {
      sess.add_data("error", format!("Could not initialise config: {}.", e));
      return Ok(Redirect::to(uri!(get)));
    },
  };

  // write the config to disk
  let mut f = File::create(config.read()._path.as_ref().unwrap())?;
  f.write_all(&update.config.as_bytes())?;

  // redirect back
  Ok(Redirect::to(uri!(get)))
}

#[derive(FromForm)]
pub struct ConfigUpdate {
  anti_csrf_token: String,
  config: String,
}
