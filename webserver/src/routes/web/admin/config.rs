use crate::{
  config::{AppConfig, Config, initialise},
  errors::*,
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
  let user = user.into_inner();
  let config_str = std::fs::read_to_string(config.read()._path.as_ref().unwrap())?;
  let mut ctx = context(&*config, Some(&user), &mut sess, langs);
  ctx["links"] = json!(super::admin_links()
    .add_value("upload", uri!(crate::routes::web::admin::config::post).to_string()));
  ctx["config_text"] = json!(config_str);

  Ok(Rst::Template(Template::render("admin/config", ctx)))
}

#[post("/admin/config", format = "application/x-www-form-urlencoded", data = "<update>")]
pub fn post(update: Form<ConfigUpdate>, config: State<Config>, _user: AdminUser, mut sess: Session) -> Result<Redirect> {
  if !sess.check_token(&update.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to(uri!(get)));
  }

  let path = config.read()._path.clone().unwrap();
  let new_config: AppConfig = match toml::from_str(&update.config) {
    Ok(c) => c,
    Err(e) => {
      sess.add_data("error", format!("Could not parse config as valid TOML: {}.", e));
      return Ok(Redirect::to(uri!(get)));
    },
  };
  *config.write() = match initialise(new_config, &path.to_string_lossy()) {
    Ok(c) => c,
    Err(e) => {
      sess.add_data("error", format!("Could not intiailise config: {}.", e));
      return Ok(Redirect::to(uri!(get)));
    },
  };

  let mut f = File::create(config.read()._path.as_ref().unwrap())?;
  f.write_all(&update.config.as_bytes())?;

  Ok(Redirect::to(uri!(get)))
}

#[derive(FromForm)]
pub struct ConfigUpdate {
  anti_csrf_token: String,
  config: String,
}
