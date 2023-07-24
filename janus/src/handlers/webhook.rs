use crate::{clients, SensorHandler, SensorMessage};

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use tracing::instrument;

use janus_common::{Target, TargetProperties::Webhook, WebhookTargetProperties};

#[derive(Debug)]
pub struct WebhookHandler {
    name: String,
    enabled: bool,
    config: WebhookTargetProperties,
    client: ClientWithMiddleware,
}

impl WebhookHandler {
    #[instrument]
    pub fn new(target_config: Target, client: reqwest::Client) -> Self {
        let Webhook(properties) = target_config.properties else {
            panic!("Invalid target properties for WebhookHandler");
        };

        let retries: u32 = properties.retries as u32;

        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(retries);

        let middleware_client = ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Self {
            name: target_config.name,
            enabled: target_config.enabled,
            config: properties,
            client: middleware_client,
        }
    }
}

impl From<Target> for WebhookHandler {
    fn from(target_config: Target) -> Self {
        let client = clients::get_default_webhook_client();

        Self::new(target_config, client)
    }
}

#[async_trait]
impl SensorHandler for WebhookHandler {
    #[instrument]
    async fn handle_reading(&self, reading: Arc<SensorMessage>) -> Result<(), String> {
        tracing::info!("Sending reading data to {}", &self.config.url);
        let response = self
            .client
            .post(&self.config.url)
            .timeout(Duration::from_secs(self.config.timeout as u64))
            .json(reading.as_ref())
            .send()
            .await;

        match response {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
