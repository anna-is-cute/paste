use config::Config;
use database::DbConn;
use errors::*;
use models::id::{UserId, EmailVerificationId};
use sidekiq_::Job;
use super::users::User;
use super::super::schema::email_verifications;
use utils::HashedPassword;

use base64;

use chrono::{Utc, DateTime, NaiveDateTime, Duration};

use diesel;
use diesel::prelude::*;

use sodiumoxide::crypto::pwhash::{pwhash_verify, HashedPassword as PwhashPassword};
use sodiumoxide::randombytes;

use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, AsChangeset, Associations)]
#[primary_key(id)]
#[belongs_to(User)]
pub struct EmailVerification {
  id: EmailVerificationId,
  email: String,
  user_id: UserId,
  key: String,
  last_sent: Option<NaiveDateTime>,
}

impl EmailVerification {
  pub fn job(&self, config: &Config, user: &User, secret: &[u8]) -> Result<Job> {
    Job::email(
      "verify.html.tera",
      json!({
        "config": config,
        "user": user,
        "verify_url": format!(
          "https://{}/account/verify?id={}&key={}",
          config.general.site_domain,
          self.id.simple(),
          base64::encode_config(secret, base64::URL_SAFE),
        ),
      }),
      config._path.as_ref().unwrap(),
      self.email.as_str(),
      "Verify your email",
    )
  }

  pub fn can_send_again(&self) -> bool {
    let last_sent = match self.last_sent {
      Some(l) => DateTime::from_utc(l, Utc),
      None => return true,
    };

    last_sent + Duration::minutes(15) < Utc::now()
  }

  pub fn update_last_sent(&mut self, conn: &DbConn, sent: NaiveDateTime) -> Result<()> {
    diesel::update(email_verifications::table)
      .filter(email_verifications::id.eq(*self.id))
      .set(email_verifications::last_sent.eq(Some(sent)))
      .execute(&**conn)?;

    self.last_sent = Some(sent);

    Ok(())
  }

  pub fn set_key(&mut self, key: String) {
    self.key = key;
  }

  pub fn update(&self, conn: &DbConn) -> Result<()> {
    diesel::update(email_verifications::table)
      .filter(email_verifications::id.eq(*self.id))
      .set(self)
      .execute(&**conn)?;

    Ok(())
  }

  pub fn check(&self, bytes: &[u8]) -> bool {
    let mut secret = self.key.as_bytes().to_vec();
    secret.push(0x00);

    let pw = match PwhashPassword::from_slice(&secret) {
      Some(p) => p,
      None => return false,
    };

    pwhash_verify(&pw, bytes)
  }
}

#[derive(Insertable)]
#[table_name = "email_verifications"]
pub struct NewEmailVerification {
  pub id: EmailVerificationId,
  pub email: String,
  pub user_id: UserId,
  pub key: String,
  pub last_sent: Option<NaiveDateTime>,
}

impl NewEmailVerification {
  pub fn new<S>(email: S, user_id: UserId, last_sent: Option<NaiveDateTime>) -> (NewEmailVerification, Vec<u8>)
    where S: Into<String>,
  {
    let secret = randombytes::randombytes(32);
    let key = HashedPassword::from(&secret).into_string();

    let nev = NewEmailVerification {
      key,
      id: EmailVerificationId(Uuid::new_v4()),
      user_id,
      last_sent,
      email: email.into(),
    };

    (nev, secret)
  }
}
