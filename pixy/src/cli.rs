//! # Command Line Definition
//!
//! This module of the crate includes the definition of the command line args that should
//! be accepted. It uses [`clap`](https://docs.rs/clap/latest/clap/) and the `derive`
//! feature to achieve this.
//!
//! The [`env`](https://docs.rs/clap/latest/clap/_features/index.html) feature of `clap`
//! is also enabled, so arguments can instead pull from the environment. See the `clap`
//! documentation for more information.

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "pixy")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// The Janus CLI definition
pub struct Cli {
    /// Increases logging verbosity, up to max of 3
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

    /// The config file to use.
    #[arg(short, long, global = true, default_value = "/etc/pixy/pixy.yaml")]
    pub(crate) config: String,

    #[command(subcommand)]
    pub(crate) command: Commands,
}

/// A subcommand to enable schema validation of the config file.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Validates the config file against the gateway schema.
    Validate(ValidateArgs),
    /// Emit sensor data to the configured targets, as defined in the config file.
    Emit(EmitArgs),
    /// Starts a server instance of Janus.
    Server(ServerArgs),
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    #[arg(from_global)]
    pub(crate) config: String,
}

/// A subcommand to emit sensor data to the configured targets. This emulates
/// the behavior of the Janus gateway server.
#[derive(Args, Debug)]
pub struct EmitArgs {
    /// The sensor data to emit to the configured targets. If not provided, this
    /// will be read from stdin.
    pub(crate) data: Option<String>,

    /// Whether the data is a file path to read from. If not provided, the data
    /// will be read as a raw string.
    #[arg(short, long)]
    pub(crate) file: bool,

    #[arg(from_global)]
    pub(crate) config: String,
}

/// Arguments for starting a server instance of Janus.
#[derive(Args, Debug)]
pub struct ServerArgs {
    /// The port to run the server on.
    #[arg(short, long, default_value = "8080")]
    pub(crate) port: u16,

    /// Whether to enable the `/echo` endpoint. Not recommended for production.
    #[arg(long, default_value_t = false)]
    pub(crate) enable_echo: bool,

    #[arg(from_global)]
    pub(crate) config: String,
}
