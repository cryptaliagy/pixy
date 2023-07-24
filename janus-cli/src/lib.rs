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

use janus::{Gateway, SensorGateway, SensorMessage};
use janus_common::parse_configs;
use std::sync::Arc;
use tracing::debug;

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

    let reading: SensorMessage =
        serde_json::from_str(&data).map_err(|e| format!("Error parsing sensor data: {}", e))?;

    gateway.handle_reading(Arc::new(reading)).await;

    Ok(())
}
