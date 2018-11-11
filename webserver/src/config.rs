use crate::errors::*;

use toml;

use std::{
  fs::File,
  io::Read,
  path::PathBuf,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
  #[serde(skip_deserializing)]
  pub _path: Option<PathBuf>,
  pub general: General,
  #[serde(default)]
  pub admin: Admin,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct General {
  pub site_name: String,
  pub site_domain: String,
  #[serde(default)]
  pub about_file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Admin {
  #[serde(default)]
  pub key: Option<String>,
}

pub fn load_config(s: &str) -> Result<Config> {
  let mut file = File::open(s)?;
  let mut content = String::new();
  file.read_to_string(&mut content)?;
  let config = toml::from_str(&content)?;

  Ok(config)
}
