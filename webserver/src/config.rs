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
  pub spam: Spam,
  pub store: Store,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Spam {
  #[serde(default)]
  pub akismet: Akismet,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Akismet {
  #[serde(default)]
  pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct General {
  pub site_name: String,
  pub site_domain: String,
  #[serde(default)]
  pub about_file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
  pub path: PathBuf,
}

pub fn load_config(s: &str) -> Result<Config> {
  let mut file = File::open(s)?;
  let mut content = String::new();
  file.read_to_string(&mut content)?;
  let config = toml::from_str(&content)?;

  Ok(config)
}
