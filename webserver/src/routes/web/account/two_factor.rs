use crate::{
  config::Config,
  database::{DbConn, models::users::User},
  errors::*,
  models::id::UserId,
  redis_store::Redis,
  routes::web::{context, AddCsp, Rst, OptionalWebUser, Session},
  utils::totp::totp_raw_skew,
};

use base32::Alphabet;

use failure::bail;

use image::{Luma, Pixel, png::PNGEncoder};

use redis::Commands;

use oath::HashType;

use qrcode::QrCode;

use rocket::{
  request::Form,
  response::Redirect,
  State,
};

use rocket_contrib::Template;

use serde_json::json;

use sodiumoxide::randombytes;

use url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET, QUERY_ENCODE_SET};

#[get("/account/2fa")]
fn get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Result<Rst> {
  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  let backups = sess.data.remove("backup_codes");

  let mut ctx = context(&*config, Some(&user), &mut sess);
  ctx["tfa_enabled"] = json!(user.tfa_enabled());
  ctx["backups"] = json!(backups);

  Ok(Rst::Template(Template::render("account/2fa/index", ctx)))
}

#[get("/account/2fa/enable")]
fn enable_get(config: State<Config>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<AddCsp<Rst>> {
  let mut user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(AddCsp::new(Rst::Redirect(Redirect::to("/login")), vec!["img-src data:"])),
  };

  if user.tfa_enabled() {
    sess.add_data("error", "2FA is already enabled on your account.");
    return Ok(AddCsp::new(Rst::Redirect(Redirect::to("lastpage")), vec!["img-src data:"]));
  }

  if user.shared_secret().is_none() {
    generate_secret(&conn, &mut user)?;
  }

  let shared_secret = base32::encode(Alphabet::RFC4648 { padding: false }, user.shared_secret().expect("missing secret"));

  // create the segments of the uri
  let unsafe_label = format!("{} - {} ({})", config.general.site_name, user.name(), user.username());
  let label = utf8_percent_encode(&unsafe_label, PATH_SEGMENT_ENCODE_SET);
  let issuer = utf8_percent_encode(&config.general.site_name, QUERY_ENCODE_SET);

  // create the uri
  let otpauth = format!("otpauth://totp/{}?secret={}&issuer={}", label, shared_secret, issuer);

  // make a qr code out of the uri
  let qr = match QrCode::new(otpauth.as_bytes()) {
    Ok(q) => q,
    Err(e) => bail!("could not create qr code: {}", e),
  };
  let img = qr
    .render::<Luma<u8>>()
    .min_dimensions(512, 512)
    .max_dimensions(512, 512)
    .build();
  let mut png: Vec<u8> = Vec::with_capacity(8192);
  PNGEncoder::new(&mut png).encode(&*img, img.width(), img.height(), Luma::<u8>::color_type())?;

  // make a data uri for the qr code
  let qr_img = format!("data:image/png;base64,{}", base64::encode(&png));

  let mut ctx = context(&*config, Some(&user), &mut sess);
  ctx["shared_secret_segments"] = json!(secret_segments(&shared_secret));
  ctx["qr_code"] = json!(qr_img);

  Ok(AddCsp::new(
    Rst::Template(Template::render("account/2fa/enable", ctx)),
    vec!["img-src data:"],
  ))
}

#[post("/account/2fa/new_secret", format = "application/x-www-form-urlencoded", data = "<form>")]
fn new_secret(form: Form<NewSecret>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  if !sess.check_token(&form.into_inner().anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("lastpage"));
  }

  let mut user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  if user.tfa_enabled() {
    sess.add_data("error", "2FA is already enabled on your account.");
    return Ok(Redirect::to("lastpage"));
  }

  generate_secret(&conn, &mut user)?;

  Ok(Redirect::to("lastpage"))
}

#[post("/account/2fa/validate", format = "application/x-www-form-urlencoded", data = "<form>")]
fn validate(form: Form<Validate>, user: OptionalWebUser, mut sess: Session, conn: DbConn, redis: Redis) -> Result<Redirect> {
  let form = form.into_inner();

  if !sess.check_token(&form.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("lastpage"));
  }

  let mut user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  if user.tfa_enabled() {
    sess.add_data("error", "2FA is already enabled on your account.");
    return Ok(Redirect::to("lastpage"));
  }

  {
    let ss = match user.shared_secret() {
      Some(s) => s,
      None => {
        sess.add_data("error", "No shared secret has been generated yet.");
        return Ok(Redirect::to("/account/2fa"));
      },
    };

    if totp_raw_skew(ss, 6, 0, 30, &HashType::SHA1).iter().all(|&x| x != form.tfa_code) {
      sess.add_data("error", "Invalid authentication code.");
      return Ok(Redirect::to("lastpage"));
    }
  }

  redis.set_ex(format!("otp:{},{}", user.id(), form.tfa_code), "", 120)?;

  user.set_tfa_enabled(true);
  user.update(&conn)?;

  let backups = generate_backup_codes(&conn, user.id())?;
  sess.add_data("backup_codes", backups.join("\n"));

  Ok(Redirect::to("/account/2fa"))
}

#[derive(Debug, FromForm)]
struct Validate {
  anti_csrf_token: String,
  tfa_code: u64,
}

#[get("/account/2fa/disable")]
fn disable_get(config: State<Config>, user: OptionalWebUser, mut sess: Session) -> Result<Rst> {
  let user = match *user {
    Some(ref u) => u,
    None => return Ok(Rst::Redirect(Redirect::to("/login"))),
  };

  if !user.tfa_enabled() {
    sess.add_data("error", "Your account does not have 2FA enabled.");
    return Ok(Rst::Redirect(Redirect::to("lastpage")));
  }

  let ctx = context(&*config, Some(&user), &mut sess);
  Ok(Rst::Template(Template::render("account/2fa/disable", ctx)))
}

#[post("/account/2fa/disable", format = "application/x-www-form-urlencoded", data = "<form>")]
fn disable_post(form: Form<Disable>, user: OptionalWebUser, mut sess: Session, conn: DbConn) -> Result<Redirect> {
  let form = form.into_inner();

  if !sess.check_token(&form.anti_csrf_token) {
    sess.add_data("error", "Invalid anti-CSRF token.");
    return Ok(Redirect::to("lastpage"));
  }

  let mut user = match user.into_inner() {
    Some(u) => u,
    None => return Ok(Redirect::to("/login")),
  };

  if !user.tfa_enabled() {
    sess.add_data("error", "Your account does not have 2FA enabled.");
    return Ok(Redirect::to("lastpage"));
  }

  if !user.check_password(&form.password) {
    sess.add_data("error", "Invalid password.");
    return Ok(Redirect::to("/account/2fa/disable"));
  }

  user.set_tfa_enabled(false);
  user.set_shared_secret(None);
  user.update(&conn)?;

  delete_backup_codes(&conn, user.id())?;

  Ok(Redirect::to("/account/2fa"))
}

#[derive(Debug, FromForm)]
struct Disable {
  anti_csrf_token: String,
  password: String,
}

fn generate_secret(conn: &DbConn, user: &mut User) -> Result<()> {
  // make the shared secret and base32 encode it
  let raw_key = randombytes::randombytes(32);

  // update the shared secret on the user
  user.set_shared_secret(Some(raw_key));
  user.update(conn)?;

  Ok(())
}

#[derive(Debug, FromForm)]
struct NewSecret {
  anti_csrf_token: String,
}

fn secret_segments(s: &str) -> Vec<&str> {
  vec![
    &s[..6],
    &s[6..12],
    &s[12..18],
    &s[18..24],
    &s[24..28],
    &s[28..34],
    &s[34..40],
    &s[40..46],
    &s[46..],
  ]
}

fn delete_backup_codes(conn: &DbConn, user: UserId) -> Result<()> {
  use crate::database::schema::backup_codes;
  use diesel::prelude::*;

  // delete any existing backup codes
  diesel::delete(backup_codes::table)
    .filter(backup_codes::user_id.eq(user))
    .execute(&**conn)?;

  Ok(())
}

fn generate_backup_codes(conn: &DbConn, user: UserId) -> Result<Vec<String>> {
  use crate::database::schema::backup_codes;
  use crate::database::models::backup_codes::NewBackupCode;
  use diesel::prelude::*;

  // delete any existing backup codes
  delete_backup_codes(conn, user)?;

  let codes: Vec<String> = (0..10)
    .map(|_| randombytes::randombytes(6))
    .map(|x| hex::encode(x))
    .collect();

  let nbcs: Vec<NewBackupCode> = codes
    .iter()
    .map(|x| NewBackupCode::new(user, x.clone()))
    .collect();

  diesel::insert_into(backup_codes::table)
    .values(&nbcs)
    .execute(&**conn)?;

  Ok(codes)
}
