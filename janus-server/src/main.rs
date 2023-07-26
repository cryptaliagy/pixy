use janus_server::{config::ServerConfiguration, run_server_with};
use tracing::Level;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() {
    let server_configs = ServerConfiguration::build().unwrap();
    let log_level = match server_configs.log_level.as_str() {
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    fmt().compact().with_max_level(log_level).init();

    run_server_with(server_configs).await;
}
