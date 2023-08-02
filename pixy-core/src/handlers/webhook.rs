use crate::{clients, SensorHandler, SensorMessage};

use std::time::Duration;

use async_trait::async_trait;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use tracing::{debug, error, info, instrument};

use minijinja::{value::Value, Environment};

use crate::config::{Target, TargetProperties::Webhook, WebhookAuth, WebhookTargetProperties};

#[derive(Debug)]
pub struct WebhookHandler {
    name: String,
    enabled: bool,
    config: WebhookTargetProperties,
    client: ClientWithMiddleware,
}

impl WebhookHandler {
    /// Creates a new WebhookHandler given a target configuration and a reqwest client.
    ///
    /// ## Arguments
    ///
    /// * `target_config` - The target configuration.
    /// * `client` - The reqwest client to use for requests.
    ///
    /// ## Examples
    /// ```
    /// use pixy_core::config::{Target, TargetProperties::Webhook, WebhookTargetProperties};
    /// use pixy_core::handlers::WebhookHandler;
    ///
    /// let target = Target {
    ///    name: "test".to_string(),
    ///    enabled: true,
    ///    properties: Webhook(WebhookTargetProperties {
    ///       url: "https://example.com".to_string(),
    ///       retries: 3,
    ///       timeout: 5,
    ///       auth: None,
    ///   }),
    /// };
    ///
    /// let client = reqwest::Client::new();
    ///
    /// let handler = WebhookHandler::new(target, client);
    /// ```
    ///
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
    /// Creates a new WebhookHandler given a target configuration.
    ///
    /// ## Arguments
    ///
    /// * `target_config` - The target configuration.
    ///
    /// ## Examples
    /// ```
    /// use pixy_core::config::{Target, TargetProperties::Webhook, WebhookTargetProperties};
    /// use pixy_core::handlers::WebhookHandler;
    ///
    /// let target = Target {
    ///    name: "test".to_string(),
    ///    enabled: true,
    ///    properties: Webhook(WebhookTargetProperties {
    ///       url: "https://example.com".to_string(),
    ///       retries: 3,
    ///       timeout: 5,
    ///       auth: None,
    ///   }),
    /// };
    ///
    /// let handler = WebhookHandler::from(target);
    /// ```
    ///
    fn from(target_config: Target) -> Self {
        let client = clients::get_default_webhook_client();

        Self::new(target_config, client)
    }
}

#[async_trait]
impl SensorHandler for WebhookHandler {
    #[instrument]
    async fn handle_reading(&self, reading: &SensorMessage, context: &Value) -> Result<(), String> {
        info!(config = ?self.config, "Sending reading data to {}", &self.config.url);
        let request = self
            .client
            .post(&self.config.url)
            .timeout(Duration::from_secs(self.config.timeout as u64))
            .json(reading);

        let env = Environment::new();

        let request = if let Some(auth) = &self.config.auth {
            match auth {
                WebhookAuth::Basic { username, password } => {
                    let username = env.render_str(username, context).map_err(|e| {
                        tracing::error!("Error rendering username: {}", e);
                        e.to_string()
                    })?;
                    let password = env.render_str(password, context).map_err(|e| {
                        tracing::error!("Error rendering password: {}", e);
                        e.to_string()
                    })?;
                    request.basic_auth(username, Some(password))
                }
                WebhookAuth::Bearer { token } => {
                    request.bearer_auth(env.render_str(token, context).map_err(|e| {
                        tracing::error!("Error rendering token: {}", e);
                        e.to_string()
                    })?)
                }
            }
        } else {
            request
        };

        let response = request
            .send()
            .await
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
            });

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

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{Method::POST, MockServer};
    use minijinja::context;

    const TEST_MESSAGE: &str = include_str!("../../../example-configs/test-sensor.json");

    fn default_properties() -> WebhookTargetProperties {
        WebhookTargetProperties {
            url: "http://localhost:8080".to_string(),
            retries: 3,
            timeout: 10,
            auth: None,
        }
    }

    #[test]
    fn test_webhook_handler_from_target() {
        let target = Target {
            name: "test".to_string(),
            enabled: true,
            properties: Webhook(default_properties()),
        };

        let handler = WebhookHandler::from(target);

        assert_eq!(handler.name, "test");
        assert_eq!(handler.config.url, "http://localhost:8080");
        assert_eq!(handler.config.retries, 3);
        assert_eq!(handler.config.timeout, 10);

        assert!(handler.enabled);
        assert!(handler.config.auth.is_none());
    }

    #[tokio::test]
    async fn test_webhook_on_simple_target() {
        let server = MockServer::start_async().await;

        let message: SensorMessage = serde_json::from_str(TEST_MESSAGE).unwrap();

        let body_contents = serde_json::to_string(&message).unwrap();

        let mock: httpmock::Mock<'_> = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("Content-Type", "application/json")
                    .body(&body_contents);
                then.status(200);
            })
            .await;

        server
            .mock_async(|when, then| {
                when.any_request();
                then.status(400);
            })
            .await;

        let mut properties = default_properties();

        properties.url = server.url("/");

        let target = Target {
            name: "test".to_string(),
            enabled: true,
            properties: Webhook(properties),
        };

        let handler = WebhookHandler::from(target);

        let result = handler.handle_reading(&message, &context!()).await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_webhook_with_basic_auth() {
        let server = MockServer::start_async().await;

        let message: SensorMessage = serde_json::from_str(TEST_MESSAGE).unwrap();

        let body_contents = serde_json::to_string(&message).unwrap();

        let mock: httpmock::Mock<'_> = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("Content-Type", "application/json")
                    .header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
                    .body(&body_contents);
                then.status(200);
            })
            .await;

        server
            .mock_async(|when, then| {
                when.any_request();
                then.status(400);
            })
            .await;

        let mut properties = default_properties();

        properties.url = server.url("/");
        properties.auth = Some(WebhookAuth::Basic {
            username: "username".to_string(),
            password: "password".to_string(),
        });

        let target = Target {
            name: "test".to_string(),
            enabled: true,
            properties: Webhook(properties),
        };

        let handler = WebhookHandler::from(target);

        let result = handler.handle_reading(&message, &context!()).await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_webhook_with_bearer_auth() {
        let server = MockServer::start_async().await;

        let message: SensorMessage = serde_json::from_str(TEST_MESSAGE).unwrap();

        let body_contents = serde_json::to_string(&message).unwrap();

        let mock: httpmock::Mock<'_> = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("Content-Type", "application/json")
                    .header("Authorization", "Bearer 123456789")
                    .body(&body_contents);
                then.status(200);
            })
            .await;

        server
            .mock_async(|when, then| {
                when.any_request();
                then.status(400);
            })
            .await;

        let mut properties = default_properties();

        properties.url = server.url("/");
        properties.auth = Some(WebhookAuth::Bearer {
            token: "123456789".to_string(),
        });

        let target = Target {
            name: "test".to_string(),
            enabled: true,
            properties: Webhook(properties),
        };

        let handler = WebhookHandler::from(target);

        let result = handler.handle_reading(&message, &context!()).await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_webhook_client_retries() {
        let server = MockServer::start_async().await;

        let message: SensorMessage = serde_json::from_str(TEST_MESSAGE).unwrap();

        let body_contents = serde_json::to_string(&message).unwrap();

        let mock: httpmock::Mock<'_> = server
            .mock_async(|when, then| {
                when.method(POST)
                    .path("/")
                    .header("Content-Type", "application/json")
                    .body(&body_contents);
                then.status(500);
            })
            .await;

        let mut properties = default_properties();

        properties.url = server.url("/");
        properties.retries = 1;
        properties.timeout = 1;

        let target = Target {
            name: "test".to_string(),
            enabled: true,
            properties: Webhook(properties),
        };

        let handler = WebhookHandler::from(target);

        let result = handler.handle_reading(&message, &context!()).await;

        assert!(result.is_err());
        mock.assert_hits_async(2).await;
    }
}
