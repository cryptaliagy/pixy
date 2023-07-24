mod config;

use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::post,
};

use crate::config::ServerConfiguration;
use janus::{Gateway, SensorGateway, SensorMessage};
use janus_common::parse_configs;

#[tokio::main]
async fn main() {
    let server_configs = ServerConfiguration::build().unwrap();

    let janus_configs = parse_configs(&server_configs.config_file).unwrap();

    let gateway: Arc<dyn Gateway> = Arc::new(SensorGateway::from(janus_configs));

    let app = axum::Router::new()
        .route("/data", post(handler))
        .with_state(gateway);

    let bind_address = format!("0.0.0.0:{}", server_configs.port);

    axum::Server::bind(&bind_address.as_str().parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(
    State(gateway): State<Arc<dyn Gateway>>,
    Json(reading): Json<SensorMessage>,
) -> StatusCode {
    let reading = Arc::new(reading);

    let gateway = gateway.clone();

    tokio::spawn(async move {
        gateway.handle_reading(reading).await;
    });

    StatusCode::ACCEPTED
}
