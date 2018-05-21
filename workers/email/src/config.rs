use toml;

use std::fs;
use std::io;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub email: Email,
}

#[derive(Debug, Deserialize)]
pub struct Email {
  pub sender: Sender,
  pub smtp: Smtp,
}

#[derive(Debug, Deserialize)]
pub struct Sender {
  pub name: String,
  pub from: String,
}

#[derive(Debug, Deserialize)]
pub struct Smtp {
  pub address: String,
  pub domain: String,
  pub port: u16,
  pub username: String,
  pub password: String,
}

#[derive(Debug)]
pub enum ConfigError {
  Io(io::Error),
  Toml(toml::de::Error),
}

pub fn config(path: &str) -> Result<Config, ConfigError> {
  let config = fs::read_to_string(path).map_err(ConfigError::Io)?;
  toml::from_str(&config).map_err(ConfigError::Toml)
}
