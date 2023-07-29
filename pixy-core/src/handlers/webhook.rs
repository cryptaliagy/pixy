use crate::{clients, SensorHandler, SensorMessage};

use std::time::Duration;

use async_trait::async_trait;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use tracing::{debug, error, info, instrument};

use crate::config::{Target, TargetProperties::Webhook, WebhookTargetProperties};

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
    async fn handle_reading(&self, reading: &SensorMessage) -> Result<(), String> {
        info!(config = ?self.config, "Sending reading data to {}", &self.config.url);
        let response = self
            .client
            .post(&self.config.url)
            .timeout(Duration::from_secs(self.config.timeout as u64))
            .json(reading)
            .send()
            .await;

        response
            .and_then(|r| match r.error_for_status() {
                Ok(res) => Ok(res),
                Err(e) => Err(reqwest_middleware::Error::from(e)),
            })
            .map(|r| {
                debug!(response = ?r, "Successfully sent reading data");
                info!(
                    response_status = r.status().as_u16(),
                    target_url = %self.config.url,
                );
            })
            .map_err(|e| {
                error!(error = ?e, "Failed to send reading data");
                e.to_string()
            })
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
