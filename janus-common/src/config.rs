use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFile {
    pub targets: Vec<Target>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub name: String,
    pub enabled: bool,
    #[serde(flatten)]
    pub properties: TargetProperties,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TargetProperties {
    Webhook(WebhookTargetProperties),
    Unknown,
}

impl Default for TargetProperties {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookTargetProperties {
    pub url: String,
    pub retries: Retries,
    pub timeout: Timeout,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Retries(u8);

impl Default for Retries {
    fn default() -> Self {
        Self(3)
    }
}

impl From<u8> for Retries {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<&Retries> for u32 {
    fn from(value: &Retries) -> Self {
        value.0 as u32
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timeout(u8);

impl Default for Timeout {
    fn default() -> Self {
        Self(1)
    }
}

impl From<u8> for Timeout {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Timeout> for u32 {
    fn from(value: Timeout) -> Self {
        value.0 as u32
    }
}

impl From<&Timeout> for Duration {
    fn from(value: &Timeout) -> Self {
        Duration::from_secs(value.0 as u64)
    }
}
