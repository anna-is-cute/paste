use crate::{
  errors::*,
  i18n::prelude::*,
  routes::web::Session,
};

use rocket::{
  request::Form,
  response::Redirect,
};

#[post("/logout", data = "<data>")]
pub fn post(data: Form<Logout>, mut sess: Session, l10n: L10n) -> Result<Redirect> {
  let data = data.into_inner();
  if !sess.check_token(&data.anti_csrf_token) {
    sess.add_data("error", l10n.tr("error-csrf")?);
    return Ok(Redirect::to("lastpage"));
  }

  sess.user_id = None;

  Ok(Redirect::to("lastpage"))
}

#[derive(FromForm)]
pub struct Logout {
  anti_csrf_token: String,
}
