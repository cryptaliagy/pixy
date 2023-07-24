pub(crate) mod clients;
pub mod handlers;

use std::sync::Arc;

use janus_common::{ConfigFile, TargetProperties};

use async_trait::async_trait;
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct SensorReading {
    raw_reading: String,
}

impl SensorReading {
    pub fn new(raw_reading: String) -> Self {
        Self { raw_reading }
    }

    pub fn get_raw_reading(&self) -> &str {
        &self.raw_reading
    }
}

#[async_trait]
pub trait SensorHandler: Send + Sync + std::fmt::Debug {
    /// Publishes the given reading to the target.
    async fn handle_reading(&self, reading: Arc<SensorReading>) -> Result<(), String>;

    /// Returns the name of the handler.
    fn get_name(&self) -> &str;

    /// Returns whether or not the handler is enabled.
    fn is_enabled(&self) -> bool;
}

#[async_trait]
pub trait Gateway {
    async fn handle_reading(&self, reading: Arc<SensorReading>);
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
    async fn handle_reading(&self, reading: Arc<SensorReading>) {
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
