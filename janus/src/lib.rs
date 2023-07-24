pub(crate) mod clients;
pub mod handlers;

use std::sync::Arc;

use janus_common::{ConfigFile, TargetProperties};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

/// A model describing the payload of the Enviro Pico board.
#[derive(Debug, Serialize, Deserialize)]
pub struct SensorMessage {
    /// The readings from the sensor.
    readings: Readings,

    /// The nickname of the specific controller board.
    nickname: String,

    /// The timestamp of the reading.
    timestamp: String,

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
    color_temperature: Option<usize>,

    /// The gas resistance in Ohms.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    gas_resistance: Option<usize>,

    /// The IAQ (Indoor Air Quality) score.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    aqi: Option<f32>,

    /// The luminance in lux.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    luminance: Option<usize>,
}

#[async_trait]
pub trait SensorHandler: Send + Sync + std::fmt::Debug {
    /// Publishes the given reading to the target.
    async fn handle_reading(&self, reading: Arc<SensorMessage>) -> Result<(), String>;

    /// Returns the name of the handler.
    fn get_name(&self) -> &str;

    /// Returns whether or not the handler is enabled.
    fn is_enabled(&self) -> bool;
}

#[async_trait]
pub trait Gateway: Send + Sync + std::fmt::Debug {
    async fn handle_reading(&self, reading: Arc<SensorMessage>);
}

#[derive(Debug, Clone)]
pub struct SensorGateway {
    handlers: Vec<Arc<dyn SensorHandler>>,
}

impl From<ConfigFile> for SensorGateway {
    fn from(config: ConfigFile) -> Self {
        let mut handlers: Vec<Arc<dyn SensorHandler>> = Vec::new();

        let client = clients::get_default_webhook_client();

        for target in config.targets {
            match target.properties {
                TargetProperties::Webhook(_) => {
                    handlers.push(Arc::new(handlers::WebhookHandler::new(
                        target,
                        client.clone(),
                    )));
                }
                TargetProperties::Unknown => {
                    tracing::warn!("Unknown target properties for target {}", target.name);
                }
            }
        }

        Self { handlers }
    }
}

#[async_trait]
impl Gateway for SensorGateway {
    #[instrument]
    async fn handle_reading(&self, reading: Arc<SensorMessage>) {
        debug!("Handling reading: {:?}", &reading);

        let mut join_handles = Vec::new();
        for handler in &self.handlers {
            let handler = handler.clone();
            let reading = reading.clone();
            if handler.is_enabled() {
                join_handles.push(tokio::spawn(async move {
                    handler.handle_reading(reading).await
                }));
            }
        }

        let mut results = Vec::new();

        for join_handle in join_handles {
            results.push(join_handle.await.unwrap());
        }

        for result in results {
            if let Err(e) = result {
                tracing::error!("Error handling reading: {}", e);
            }
        }
    }
}
