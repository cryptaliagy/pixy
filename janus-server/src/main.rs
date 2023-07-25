mod config;

use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
};
use tracing::{debug, info, instrument, Level};
use tracing_subscriber::fmt;

use crate::config::ServerConfiguration;
use janus::{Gateway, SensorGateway, SensorMessage};
use janus_common::parse_configs;

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

    let janus_configs = parse_configs(&server_configs.config_file).unwrap();

    let gateway: Arc<dyn Gateway> = Arc::new(SensorGateway::from(janus_configs));

    let app = axum::Router::new()
        .route("/data", post(handler))
        .route("/healthz", get(|| async { StatusCode::OK }))
        .route("/echo", post(echo))
        .with_state(gateway);

    let bind_address = format!("0.0.0.0:{}", server_configs.port);

    info!("Starting server on {}", &bind_address);
    axum::Server::bind(&bind_address.as_str().parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[instrument]
async fn handler(
    State(gateway): State<Arc<dyn Gateway>>,
    Json(reading): Json<SensorMessage>,
) -> StatusCode {
    debug!("Received reading: {:?}", &reading);
    let reading = Arc::new(reading);

    let gateway = gateway.clone();

    tokio::spawn(async move {
        gateway.handle_reading(reading).await;
    });

    StatusCode::ACCEPTED
}

#[instrument]
async fn echo(Json(data): Json<SensorMessage>) -> Json<SensorMessage> {
    info!("Received data: {:?}", &data);
    Json(data)
}
