use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct AntiCsrfToken(
  pub String,
  pub DateTime<Utc>,
);
