use config::{Config, ConfigError, Environment};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfiguration {
    pub port: u16,
    pub log_level: String,
    pub config_file: String,
}

impl ServerConfiguration {
    pub fn build() -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(Environment::with_prefix("JANUS"))
            .set_default("port", 8080)?
            .set_default("log_level", "info")?
            .set_default("config_file", "janus.yaml")?
            .build()?
            .try_deserialize()
    }
}
