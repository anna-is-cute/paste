use chrono::Utc;
use sha1::Sha1;
use one_time::totp::TotpOptions;

pub fn totp_raw_skew(key: &[u8]) -> Option<[u64; 3]> {
  let timestamp = Utc::now().timestamp() as u64;
  let mut options = TotpOptions {
    key,
    digits: 6,
    step: 30,
    epoch: 0,
    timestamp,
  };

  let cur = one_time::totp::<Sha1>(&options).ok()?;

  options.timestamp -= 30;
  let prev = one_time::totp::<Sha1>(&options).ok()?;

  options.timestamp += 60;
  let next = one_time::totp::<Sha1>(&options).ok()?;

  Some([cur, prev, next])
}
