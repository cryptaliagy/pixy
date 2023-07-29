use crate::config::ConfigFile;

use std::{collections::HashMap, env};
use tracing::debug;

use minijinja::{context, Environment};

pub const GATEWAY_SCHEMA: &str = include_str!("../schemas/gateway.schema.json");

fn validate_config(config_value: &serde_json::Value) -> Result<(), String> {
    let schema_value: serde_json::Value =
        serde_json::from_str(GATEWAY_SCHEMA).map_err(|e| format!("Error parsing schema: {}", e))?;

    let schema = jsonschema::JSONSchema::compile(&schema_value)
        .map_err(|e| format!("Error compiling schema: {}", e))?;

    schema.validate(config_value).map_err(|e| {
        let errors = e
            .into_iter()
            .map(|e| {
                if e.instance_path.to_string() != "" {
                    format!("\tError validating {}: {}", e.instance_path, e)
                } else {
                    format!("\tError validating schema: {}", e)
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        format!("Validation failed:\n{}", errors)
    })
}

pub fn parse_configs(file_name: &str) -> Result<ConfigFile, String> {
    debug!("Validating file: {}", file_name);

    let config_text =
        std::fs::read_to_string(file_name).map_err(|e| format!("Error reading file: {}", e))?;

    let mut env = Environment::new();

    env.add_template("pixy", &config_text)
        .map_err(|e| format!("Error adding template to environment: {}", e))?;

    let env_vars: HashMap<String, String> = env::vars()
        .filter(|(key, _)| key.starts_with("PIXY_"))
        .map(|(key, val)| (key.replace("PIXY_", ""), val))
        .collect();

    let rendered = env
        .get_template("pixy")
        .map_err(|e| format!("Error getting template: {}", e))?
        .render(context!(env => env_vars))
        .map_err(|e| format!("Error rendering template: {}", e))?;

    let config: serde_json::Value =
        serde_yaml::from_str(&rendered).map_err(|e| format!("Error parsing YAML: {}", e))?;

    validate_config(&config)?;

    let config_file: ConfigFile = serde_json::from_value(config)
        .map_err(|e| format!("Error parsing config into target: {}", e))?;

    debug!("Deserialized config file: {:?}", &config_file);

    Ok(config_file)
}
