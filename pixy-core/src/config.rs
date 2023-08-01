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
    #[serde(default = "_default_true")]
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

    #[serde(default)]
    pub auth: Option<WebhookAuth>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebhookAuth {
    Basic { username: String, password: String },
    Bearer { token: String },
}

fn _default_retries() -> u8 {
    3
}

fn _default_timeout() -> u8 {
    10
}

fn _default_true() -> bool {
    true
}

impl std::fmt::Debug for WebhookAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookAuth::Basic {
                username: _,
                password: _,
            } => {
                write!(f, "Basic {{ username: ******, password: ****** }}")
            }
            WebhookAuth::Bearer { token: _ } => write!(f, "Bearer {{ token: ******* }}"),
        }
    }
}
