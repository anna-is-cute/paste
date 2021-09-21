use crate::{
  database::DbConn,
  errors::*,
  i18n::L10n,
  utils::BitMask,
};

use super::super::schema::login_attempts;

use chrono::{DateTime, Duration, NaiveDateTime, Utc};

use diesel::prelude::*;

use ipnetwork::IpNetwork;

use std::net::IpAddr;

#[derive(Debug, Identifiable, AsChangeset, Queryable, Associations)]
#[primary_key(addr)]
pub struct LoginAttempt {
  addr: IpNetwork,
  timestamp: NaiveDateTime,
  attempts: i32,
}

impl LoginAttempt {
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

  pub fn get_or_insert(conn: &DbConn, network: IpNetwork) -> Result<LoginAttempt> {
    diesel::insert_into(login_attempts::table)
      .values(&NewLoginAttempt::new(network, 0))
      .on_conflict_do_nothing()
      .execute(&**conn)?;

    let attempt: LoginAttempt = login_attempts::table
      .filter(login_attempts::addr.eq(network))
      .first(&**conn)?;

    Ok(attempt)
  }

  pub fn get(conn: &DbConn, network: IpNetwork) -> Result<Option<LoginAttempt>> {
    let attempt: Option<LoginAttempt> = login_attempts::table
      .filter(login_attempts::addr.eq(network))
      .first(&**conn)
      .optional()?;

    Ok(attempt)
  }

  fn network(ip: IpAddr) -> Option<IpNetwork> {
    if !ip.is_global() {
      return None;
    }
    let prefix = if ip.is_ipv4() { 32 } else { 64 };
    let masked_ip = ip.to_masked(prefix);
    IpNetwork::new(masked_ip, prefix).map(Some).expect("bad prefix")
  }

  pub fn increment(&mut self, conn: &DbConn) -> Result<()> {
    let new_amount = self.attempts() + 1;
    self.set_attempts(new_amount);

    if self.attempts() < 5 {
      self.set_timestamp(Utc::now().naive_utc());
    }

    self.update(conn)?;

    Ok(())
  }

  pub fn find_increment(conn: &DbConn, l10n: &L10n, ip: IpAddr) -> Result<Option<String>> {
    let network = match LoginAttempt::network(ip) {
      Some(n) => n,
      None => return Ok(None),
    };
    let mut attempt = LoginAttempt::get_or_insert(conn, network)?;

    attempt.increment(conn)?;
    attempt.check(l10n)
  }

  pub fn check(&self, l10n: &L10n) -> Result<Option<String>> {
    let attempts = self.attempts();
    if attempts < 5 {
      return Ok(None);
    }

    let expires = DateTime::<Utc>::from_utc(*self.timestamp(), Utc) + Duration::minutes(30);
    if expires <= Utc::now() {
      return Ok(None);
    }

    let minutes = expires.signed_duration_since(Utc::now()).num_minutes();
    if minutes != 0 {
      Ok(Some(l10n.tr_ex(
        ("login-error", "rate-limit"),
        |req| req.arg("minutes", minutes),
      )?))
    } else {
      Ok(Some(l10n.tr(("login-error", "rate-limit-soon"))?))
    }
  }

  pub fn find_check(conn: &DbConn, l10n: &L10n, ip: IpAddr) -> Result<Option<String>> {
    let network = match LoginAttempt::network(ip) {
      Some(n) => n,
      None => return Ok(None),
    };
    let attempt = match LoginAttempt::get(conn, network)? {
      Some(a) => a,
      None => return Ok(None),
    };

    attempt.check(l10n)
  }

  pub fn update(&self, conn: &DbConn) -> Result<()> {
    diesel::update(login_attempts::table)
      .filter(login_attempts::addr.eq(self.addr()))
      .set(self)
      .execute(&**conn)?;

    Ok(())
  }
}

#[derive(Insertable)]
#[table_name = "login_attempts"]
pub struct NewLoginAttempt {
  addr: IpNetwork,
  attempts: i32,
}

impl NewLoginAttempt {
  pub fn new(addr: IpNetwork, attempts: i32) -> Self {
    NewLoginAttempt { addr, attempts }
  }
}
