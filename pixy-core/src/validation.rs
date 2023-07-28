use crate::config::ConfigFile;

use std::fs::File;
use tracing::debug;

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

    let file_handler = File::open(file_name).map_err(|e| format!("Error opening file: {}", e))?;

    let config: serde_json::Value =
        serde_yaml::from_reader(file_handler).map_err(|e| format!("Error parsing YAML: {}", e))?;

    validate_config(&config)?;

    let config_file: ConfigFile = serde_json::from_value(config)
        .map_err(|e| format!("Error parsing config into target: {}", e))?;

    debug!("Deserialized config file: {:?}", &config_file);

    Ok(config_file)
}
