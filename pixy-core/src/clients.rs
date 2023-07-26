use reqwest::{Client, ClientBuilder};
use std::time::Duration;

pub(crate) fn get_default_webhook_client() -> Client {
    ClientBuilder::new()
        .timeout(Duration::from_secs(60))
        .build()
        .expect("Failed to build reqwest client")
}
