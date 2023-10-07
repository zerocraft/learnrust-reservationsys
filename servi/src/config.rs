use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::ServError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub db: DbConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbConfig {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    #[allow(dead_code)]
    pub async fn load(path: &str) -> Result<Self, ServError> {
        let config = fs::read_to_string(&path).await;
        if config.is_err() {
            return Err(ServError(rsys::error::RsysError::ConfigError(
                path.to_string(),
            )));
        }
        let config = serde_yaml::from_str(&config.unwrap());
        if config.is_err() {
            return Err(ServError(rsys::error::RsysError::ConfigError(
                path.to_string(),
            )));
        }
        Ok(config.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn load_config() {
        let _path = include_str!("../../config.yml");
        let config = Config::load("../config.yml").await;
        if config.is_err() {
            println!("{:?}", config.err());
        } else {
            println!("{:?}", config.unwrap());
        }
    }
}
