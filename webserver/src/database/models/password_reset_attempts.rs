use database::DbConn;
use errors::*;
use super::super::schema::password_reset_attempts;
use utils::BitMask;

use chrono::{DateTime, Duration, NaiveDateTime, Utc};

use diesel;
use diesel::prelude::*;

use ipnetwork::IpNetwork;

use std::net::IpAddr;

#[derive(Debug, Identifiable, AsChangeset, Queryable, Associations)]
#[primary_key(addr)]
pub struct PasswordResetAttempt {
  addr: IpNetwork,
  timestamp: NaiveDateTime,
  attempts: i32,
}

impl PasswordResetAttempt {
  pub fn addr(&self) -> IpNetwork {
    self.addr
  }

  pub fn set_addr(&mut self, addr: IpNetwork) {
    self.addr = addr;
  }

  pub fn timestamp(&self) -> &NaiveDateTime {
    &self.timestamp
  }

  pub fn set_timestamp(&mut self, timestamp: NaiveDateTime) {
    self.timestamp = timestamp;
  }

  pub fn attempts(&self) -> i32 {
    self.attempts
  }

  pub fn set_attempts(&mut self, attempts: i32) {
    self.attempts = attempts;
  }

  pub fn get_or_insert(conn: &DbConn, network: IpNetwork) -> Result<PasswordResetAttempt> {
    diesel::insert_into(password_reset_attempts::table)
      .values(&NewPasswordResetAttempt::new(network, 0))
      .on_conflict_do_nothing()
      .execute(&**conn)?;

    let attempt: PasswordResetAttempt = password_reset_attempts::table
      .filter(password_reset_attempts::addr.eq(network))
      .first(&**conn)?;

    Ok(attempt)
  }

  pub fn get(conn: &DbConn, network: IpNetwork) -> Result<Option<PasswordResetAttempt>> {
    let attempt: Option<PasswordResetAttempt> = password_reset_attempts::table
      .filter(password_reset_attempts::addr.eq(network))
      .first(&**conn)
      .optional()?;

    Ok(attempt)
  }

  fn network(ip: IpAddr) -> IpNetwork {
    let prefix = if ip.is_ipv4() { 32 } else { 64 };
    let masked_ip = ip.to_masked(prefix);
    IpNetwork::new(masked_ip, prefix).expect("bad prefix")
  }

  pub fn increment(&mut self, conn: &DbConn) -> Result<()> {
    let new_amount = self.attempts() + 1;
    self.set_attempts(new_amount);

    if self.attempts() < 3 {
      self.set_timestamp(Utc::now().naive_utc());
    }

    self.update(conn)?;

    Ok(())
  }

  pub fn find_increment(conn: &DbConn, ip: IpAddr) -> Result<Option<String>> {
    let network = PasswordResetAttempt::network(ip);
    let mut attempt = PasswordResetAttempt::get_or_insert(conn, network)?;

    attempt.increment(conn)?;
    attempt.check()
  }

  pub fn check(&self) -> Result<Option<String>> {
    let attempts = self.attempts();
    if attempts < 3 {
      return Ok(None);
    }

    let expires = DateTime::from_utc(*self.timestamp(), Utc) + Duration::hours(1);
    if expires <= Utc::now() {
      return Ok(None);
    }

    let minutes = expires.signed_duration_since(Utc::now()).num_minutes();
    if minutes != 0 {
      Ok(Some(format!(
        "Please try again in {} minute{}.",
        minutes,
        if minutes == 1 { "" } else { "s" }
      )))
    } else {
      Ok(Some("Please try again in a few seconds.".into()))
    }
  }

  pub fn find_check(conn: &DbConn, ip: IpAddr) -> Result<Option<String>> {
    let network = PasswordResetAttempt::network(ip);
    let attempt = match PasswordResetAttempt::get(conn, network)? {
      Some(a) => a,
      None => return Ok(None),
    };

    attempt.check()
  }

  pub fn update(&self, conn: &DbConn) -> Result<()> {
    diesel::update(password_reset_attempts::table)
      .filter(password_reset_attempts::addr.eq(self.addr()))
      .set(self)
      .execute(&**conn)?;

    Ok(())
  }
}

#[derive(Insertable)]
#[table_name = "password_reset_attempts"]
pub struct NewPasswordResetAttempt {
  addr: IpNetwork,
  attempts: i32,
}

impl NewPasswordResetAttempt {
  pub fn new(addr: IpNetwork, attempts: i32) -> Self {
    NewPasswordResetAttempt { addr, attempts }
  }
}
