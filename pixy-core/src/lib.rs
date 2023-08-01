pub(crate) mod clients;
pub mod config;
pub mod handlers;
pub mod validation;

use std::collections::HashMap;

use crate::config::{ConfigFile, TargetProperties};

use async_trait::async_trait;
use minijinja::{context, value::Value};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

/// A model describing the payload of the Enviro Pico board.
#[derive(Debug, Serialize, Deserialize)]
pub struct SensorMessage {
    /// The readings from the sensor.
    readings: Readings,

    /// The timestamp of the reading.
    timestamp: String,

    /// The metadata of the sensor.
    #[serde(flatten)]
    metadata: SensorMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorMetadata {
    /// The nickname of the specific controller board.
    nickname: String,

    /// The model of the controller board.
    model: String,

    /// The unique identifier of the controller board.
    uid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Readings {
    // Sensors in every board
    /// The temperature in degrees Celsius.
    temperature: f32,

    /// The pressure in hPa.
    pressure: f32,

    /// The humidity in percentage.
    humidity: f32,

    /// The color temperature in Kelvin.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    color_temperature: Option<u64>,

    /// The gas resistance in Ohms.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    gas_resistance: Option<u64>,

    /// The IAQ (Indoor Air Quality) score.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    aqi: Option<f32>,

    /// The luminance in lux.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    luminance: Option<u64>,
}

#[async_trait]
pub trait SensorHandler: Send + Sync + std::fmt::Debug {
    /// Publishes the given reading to the target.
    async fn handle_reading(&self, reading: &SensorMessage, context: &Value) -> Result<(), String>;

    /// Returns the name of the handler.
    fn get_name(&self) -> &str;

    /// Returns whether or not the handler is enabled.
    fn is_enabled(&self) -> bool;
}

#[async_trait]
pub trait Gateway: Send + Sync + std::fmt::Debug {
    async fn handle_reading(&self, reading: SensorMessage);
}

#[derive(Debug)]
pub struct SensorGateway {
    handlers: Vec<Box<dyn SensorHandler>>,
    env_vars: HashMap<String, String>,
}

impl From<ConfigFile> for SensorGateway {
    fn from(config: ConfigFile) -> Self {
        let mut handlers: Vec<Box<dyn SensorHandler>> = Vec::new();

        let client = clients::get_default_webhook_client();

        let env_vars = std::env::vars()
            .filter(|(key, _)| key.starts_with("PIXY_"))
            .map(|(key, value)| (key.replace("PIXY_", ""), value))
            .collect();

        for target in config.targets {
            match target.properties {
                TargetProperties::Webhook(_) => {
                    handlers.push(Box::new(handlers::WebhookHandler::new(
                        target,
                        client.clone(),
                    )));
                }
                TargetProperties::Unknown => {
                    tracing::warn!("Unknown target properties for target {}", target.name);
                }
            }
        }

        Self { handlers, env_vars }
    }
}

#[async_trait]
impl Gateway for SensorGateway {
    #[instrument]
    async fn handle_reading(&self, reading: SensorMessage) {
        debug!("Handling reading: {:?}", &reading);

        let ctx = context!(env => self.env_vars, reading => reading);

        for handler in &self.handlers {
            let _ = handler.handle_reading(&reading, &ctx).await.map_err(|_| {
                tracing::error!(?handler, "Handler produced error");
            });
        }
    }
}
