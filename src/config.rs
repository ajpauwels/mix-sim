use std::{error::Error, fmt::Display};

use config::{Config as ExternalConfig, ConfigError as ExternalConfigError};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Directory {
    pub buffer_size: Option<usize>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Client {
    pub id: String,
    pub buffer_size: Option<usize>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Server {
    pub buffer_size: Option<usize>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub server: Option<Server>,
    pub directory: Option<Directory>,
    pub clients: Option<Vec<Client>>,
}

#[derive(Debug)]
pub struct ConfigError {
    parent: ExternalConfigError,
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.parent)
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parent)
    }
}

pub fn load_config(path: &str, prefix: &str) -> Result<Config, ExternalConfigError> {
    let mut config_builder = ExternalConfig::builder();
    let paths = std::fs::read_dir(path)
        .map_err(|_| ExternalConfigError::Message(format!("Could not read directory at {path}")))?
        .filter_map(|result| result.ok())
        .map(|de| de.path());
    for path in paths {
        config_builder = match path.as_path().to_str() {
            Some(s) => {
                if s.ends_with(".yaml") {
                    config_builder.add_source(config::File::with_name(s))
                } else {
                    config_builder
                }
            }
            None => config_builder,
        };
    }
    config_builder
        .add_source(config::Environment::with_prefix(prefix).separator("_"))
        .build()?
        .try_deserialize()
}
