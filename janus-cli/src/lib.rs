//! # Rust Command Line App
//!
//! This crate defines a binary application with sensible defaults for logging
//! and a basic verbosity option. It is meant to act as a starting point
//! for CLI applications to offer a batteries-mostly-included approach to CI/CD,
//! logging, and documentation.
//!
//! Please see the top level [README.md](https://github.com/taliamax/cli-rs) file
//! for instructions on how to set up this template repository for new projects
//!
//! ## Default Features
//! - **colors**: Causes the logging level of a log message to be coloured depending on its level
//! - **env**: Enables the `clap` feature `env` for pulling values from the environment
//! - **wrap_help**: Enables the `clap` feature `wrap_help` for wrapping help messages for terminal size
//!

pub mod cli;
pub mod logging;

use janus::{Gateway, SensorGateway, SensorReading};
use janus_common::ConfigFile;
use jsonschema::JSONSchema;
use std::{fs::File, sync::Arc};
use tracing::debug;

const GATEWAY_SCHEMA: &str = include_str!("../../schemas/gateway.schema.json");

pub async fn run(cli: cli::Cli) {
    logging::setup_logging(&cli);

    debug!("CLI config: {:?}", &cli);

    let result = match cli.command {
        cli::Commands::Validate(args) => run_validate(args),
        cli::Commands::Emit(args) => run_emit(args).await,
    };

    if let Err(e) = result {
        println!("{}", e);
    }
}

fn validate_config(config_value: &serde_json::Value) -> Result<(), String> {
    let schema_value: serde_json::Value =
        serde_json::from_str(GATEWAY_SCHEMA).map_err(|e| format!("Error parsing schema: {}", e))?;

    let schema =
        JSONSchema::compile(&schema_value).map_err(|e| format!("Error compiling schema: {}", e))?;

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

fn parse_configs(file_name: &str) -> Result<ConfigFile, String> {
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

fn run_validate(args: cli::ValidateArgs) -> Result<(), String> {
    debug!("Validation args: {:?}", &args);

    let file = args.file.unwrap_or_else(|| "janus.yaml".to_string());
    parse_configs(&file)?;

    println!("Validation succeeded!");

    Ok(())
}

async fn run_emit(args: cli::EmitArgs) -> Result<(), String> {
    let config_file = args.config.unwrap_or_else(|| "janus.yaml".to_string());

    let config = parse_configs(&config_file)?;

    let gateway = SensorGateway::from(config);

    debug!("Gateway: {:?}", &gateway);

    let is_file = args.file;
    let data = args.data.unwrap_or_default();

    let data = if is_file {
        std::fs::read_to_string(data).map_err(|e| format!("Error reading file: {}", e))?
    } else if data.is_empty() {
        // Read from stdin
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .map_err(|e| format!("Error reading from stdin: {}", e))?;

        buffer
    } else {
        data
    };

    let reading = SensorReading::new(data);

    gateway.handle_reading(Arc::new(reading)).await;

    Ok(())
}
