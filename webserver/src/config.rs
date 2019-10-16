use crate::errors::*;

use parking_lot::RwLock;

use toml;

use std::{
  fs::File,
  io::Read,
  path::PathBuf,
};

pub type Config = RwLock<AppConfig>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
  #[serde(skip_deserializing)]
  pub _path: Option<PathBuf>,
  pub general: General,
  #[serde(default)]
  pub admin: Admin,
  pub store: Store,
  #[serde(default)]
  pub pastes: Pastes,
  #[serde(default)]
  pub registration: Registration,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
  pub path: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Pastes {
  pub sign_in_to_create: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Registration {
  pub open: bool,
  pub whitelisted_emails: Vec<String>,
}

impl Default for Registration {
  fn default() -> Self {
    Self {
      open: true,
      whitelisted_emails: Vec::new(),
    }
  }
}

pub fn load_config(s: &str) -> Result<AppConfig> {
  let mut file = File::open(s)?;
  let mut content = String::new();
  file.read_to_string(&mut content)?;
  let config = toml::from_str(&content)?;
  initialise(config, s)
}

pub fn initialise(mut config: AppConfig, s: &str) -> Result<AppConfig> {
  let path = PathBuf::from(s)
    .canonicalize()
    .map_err(|e| failure::format_err!("could not canonicalise config path: {}", e))?;
  config._path = Some(path);
  std::fs::create_dir_all(&config.store.path)
    .map_err(|e| failure::format_err!(
      "could not create store at {}: {}",
      config.store.path.to_string_lossy(),
      e,
    ))?;
  let store_path = config.store.path
    .canonicalize()
    .map_err(|e| failure::format_err!("could not canonicalise store path: {}", e))?;
  config.store.path = store_path;
  Ok(config)
}
