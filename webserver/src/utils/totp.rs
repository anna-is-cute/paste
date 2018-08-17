use chrono::Utc;

use oath::HashType;

pub fn totp_raw_skew(key: &[u8], digits: u32, epoch: u64, time_step: u64, hash: &HashType) -> [u64; 3] {
  let timestamp = Utc::now().timestamp() as u64;

  [
    oath::totp_raw_custom_time(key, digits, epoch, time_step, timestamp, hash),
    oath::totp_raw_custom_time(key, digits, epoch, time_step, timestamp - time_step, hash),
    oath::totp_raw_custom_time(key, digits, epoch, time_step, timestamp + time_step, hash),
  ]
}
