use config::Config;
use database::DbConn;
use errors::*;
use models::id::{UserId, EmailVerificationId, EmailVerificationKey};
use sidekiq_::Job;
use super::users::User;
use super::super::schema::email_verifications;

use chrono::{Utc, DateTime, NaiveDateTime, Duration};

use diesel;
use diesel::prelude::*;

use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable, AsChangeset, Associations)]
#[primary_key(id)]
#[belongs_to(User)]
pub struct EmailVerification {
  id: EmailVerificationId,
  email: String,
  user_id: UserId,
  key: EmailVerificationKey,
  last_sent: Option<NaiveDateTime>,
}

impl EmailVerification {
  pub fn job(&self, config: &Config, user: &User) -> Result<Job> {
    Job::email(
      "verify.html.tera",
      json!({
        "config": config,
        "user": user,
        "verify_url": format!(
          "https://{}/account/verify?id={}&key={}",
          config.general.site_domain,
          self.id.simple(),
          self.key.simple(),
        ),
      }),
      config._path.as_ref().unwrap(),
      self.email.as_str(),
      user.name(),
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
}

#[derive(Insertable)]
#[table_name = "email_verifications"]
pub struct NewEmailVerification {
  pub id: EmailVerificationId,
  pub email: String,
  pub user_id: UserId,
  pub key: EmailVerificationKey,
  pub last_sent: Option<NaiveDateTime>,
}

impl NewEmailVerification {
  pub fn new<S>(email: S, user_id: UserId, last_sent: Option<NaiveDateTime>) -> NewEmailVerification
    where S: Into<String>,
  {
    NewEmailVerification {
      id: EmailVerificationId(Uuid::new_v4()),
      user_id,
      last_sent,
      email: email.into(),
      key: EmailVerificationKey(Uuid::new_v4()),
    }
  }
}
