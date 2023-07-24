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
    #[serde(default = "_default_retries")]
    pub retries: u8,

    #[serde(default = "_default_timeout")]
    pub timeout: u8,
}

fn _default_retries() -> u8 {
    3
}

fn _default_timeout() -> u8 {
    10
}
