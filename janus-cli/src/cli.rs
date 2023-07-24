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
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
/// The Janus CLI definition
pub struct Cli {
    /// Increases logging verbosity, up to max of 3
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

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
}

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// The file to validate. Defaults to `janus.yaml` in the current directory.
    pub(crate) file: Option<String>,
}

/// A subcommand to emit sensor data to the configured targets. This emulates
/// the behavior of the Janus gateway server.
#[derive(Args, Debug)]
pub struct EmitArgs {
    /// The config file to use. Defaults to `janus.yaml` in the current directory.
    #[arg(short, long)]
    pub(crate) config: Option<String>,

    /// The sensor data to emit to the configured targets. If not provided, this
    /// will be read from stdin.
    pub(crate) data: Option<String>,

    /// Whether the data is a file path to read from. If not provided, the data
    /// will be read as a raw string.
    #[arg(short, long)]
    pub(crate) file: bool,
}
