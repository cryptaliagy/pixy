use config::{Config, ConfigError, Environment};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfiguration {
    pub port: u16,
    pub log_level: String,
    pub config_file: String,
    pub enable_echo: bool,
}

impl ServerConfiguration {
    pub fn build() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(Environment::with_prefix("PIXY"))
            .set_default("port", 9147)?
            .set_default("log_level", "info")?
            .set_default("config_file", "/pixy.yaml")?
            .set_default("enable_echo", false)?
            .build()?
            .try_deserialize()
    }
}
