//! # Setting up logging
//!
//! Coloured output is available by default with the `colors` feature, but can be disabled
//! when installing if using the `--no-default-features` flag.

use crate::cli::Cli;

use tracing::{trace, Level};
use tracing_subscriber::FmtSubscriber;

/// Configures the logging according to the CLI configurations. This
/// enables coloured output (if using `colors` feature), and formats the
/// output to include the file and line (if running in `debug` profile).
pub fn setup_logging(cli: &Cli) {
    let level = match cli.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_max_level(level)
        .with_target(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    trace!("Finished setting up logging configuration!");
}
