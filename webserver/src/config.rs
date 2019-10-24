use crate::{
  errors::*,
  filter::Filter,
};

use parking_lot::RwLock;

use std::path::PathBuf;

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
  #[serde(default, rename = "filter")]
  pub filters: Vec<Filter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct General {
  pub site_name: String,
  pub site_domain: String,
  #[serde(default)]
  pub about_file: Option<String>,
  #[serde(default)]
  pub convert_avatars: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Admin {
  #[serde(default)]
  pub key: Option<String>,
  #[serde(default)]
  pub admins_can_edit_config: bool,
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
  // read in the given path to a string
  let config_str = std::fs::read_to_string(s)?;
  // deserialise the config as toml
  let config = toml::from_str(&config_str)?;
  // initialise the config
  initialise(config, s)
}

pub fn initialise(mut config: AppConfig, s: &str) -> Result<AppConfig> {
  // unset the admin key if it's the empty string
  if config.admin.key.as_deref().map(str::is_empty).unwrap_or(false) {
    config.admin.key = None;
  }

  // canonicalise the config path and set it on the config
  let path = PathBuf::from(s)
    .canonicalize()
    .map_err(|e| anyhow::anyhow!("could not canonicalise config path: {}", e))?;
  config._path = Some(path);

  // create the config store path
  std::fs::create_dir_all(&config.store.path)
    .map_err(|e| anyhow::anyhow!(
      "could not create store at {}: {}",
      config.store.path.to_string_lossy(),
      e,
    ))?;

  // canonicalise the store path and set it on the config
  let store_path = config.store.path
    .canonicalize()
    .map_err(|e| anyhow::anyhow!("could not canonicalise store path: {}", e))?;
  config.store.path = store_path;

  Ok(config)
}
