pub mod config;

use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
};
use tracing::{debug, info, instrument};

use crate::config::ServerConfiguration;
use pixy_core::validation::parse_configs;
use pixy_core::{Gateway, SensorGateway, SensorMessage};

fn create_app(gateway: Arc<dyn Gateway>, server_configs: &ServerConfiguration) -> axum::Router {
    let app = axum::Router::new()
        .route("/data", post(handler))
        .route("/healthz", get(|| async { StatusCode::OK }))
        .with_state(gateway);

    if server_configs.enable_echo {
        app.route("/echo", post(echo))
    } else {
        app
    }
}

pub async fn run_server_with_gateway(
    gateway: Arc<dyn Gateway>,
    server_configs: ServerConfiguration,
) {
    let app = create_app(gateway, &server_configs);

    let bind_address = format!("0.0.0.0:{}", server_configs.port);

    println!(
        r#"
     ___                     ___                 
    /  /\      ___          /__/|          ___   
   /  /::\    /  /\        |  |:|         /__/|  
  /  /:/\:\  /  /:/        |  |:|        |  |:|  
 /  /:/~/:/ /__/::\      __|__|:|        |  |:|  
/__/:/ /:/  \__\/\:\__  /__/::::\____  __|__|:|  
\  \:\/:/      \  \:\/\    ~\~~\::::/ /__/::::\  
 \  \::/        \__\::/     |~~|:|~~     ~\~~\:\ 
  \  \:\        /__/:/      |  |:|         \  \:\
   \  \:\       \__\/       |  |:|          \__\/
    \__\/                   |__|/                
    "#
    );

    info!("Starting server on {}", &bind_address);
    axum::Server::bind(&bind_address.as_str().parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

pub async fn run_server_with(server_configs: ServerConfiguration) {
    let pixy_configs = parse_configs(&server_configs.config_file).unwrap();

    let gateway: Arc<dyn Gateway> = Arc::new(SensorGateway::from(pixy_configs));

    run_server_with_gateway(gateway, server_configs).await;
}

#[instrument]
async fn handler(
    State(gateway): State<Arc<dyn Gateway>>,
    Json(reading): Json<SensorMessage>,
) -> StatusCode {
    debug!("Received reading: {:?}", &reading);

    tokio::spawn(async move {
        gateway.handle_reading(reading).await;
    });

    StatusCode::ACCEPTED
}

#[instrument]
async fn echo(data: String) -> String {
    info!("Received data: {:?}", &data);
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::http::{self, Request};
    use tower::ServiceExt;

    #[derive(Debug)]
    struct MockGateway {}

    #[async_trait]
    impl Gateway for MockGateway {
        async fn handle_reading(&self, _reading: SensorMessage) {}
    }

    fn default_config() -> ServerConfiguration {
        ServerConfiguration {
            config_file: String::new(),
            port: 9147,
            log_level: String::from("info"),
            enable_echo: false,
        }
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let gateway: Arc<dyn Gateway> = Arc::new(MockGateway {});

        let app = create_app(gateway, &default_config());

        let res = app
            .oneshot(Request::get("/healthz").body("".into()).unwrap())
            .await
            .unwrap();

        assert_eq!(res.status(), http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_echo_enabled() {
        let mut configs = default_config();

        configs.enable_echo = true;

        let gateway: Arc<dyn Gateway> = Arc::new(MockGateway {});

        let app = create_app(gateway, &configs);

        let res = app
            .oneshot(Request::post("/echo").body("hello".into()).unwrap())
            .await
            .unwrap();

        assert_eq!(res.status(), http::StatusCode::OK);

        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();

        assert_eq!(&body[..], b"hello");
    }

    #[tokio::test]
    async fn test_echo_disable() {
        let gateway: Arc<dyn Gateway> = Arc::new(MockGateway {});

        let app = create_app(gateway, &default_config());

        let res = app
            .oneshot(Request::post("/echo").body("hello".into()).unwrap())
            .await
            .unwrap();

        assert_eq!(res.status(), http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_example_sensor_works() {
        let gateway: Arc<dyn Gateway> = Arc::new(MockGateway {});

        let app = create_app(gateway, &default_config());

        let example_sensor: SensorMessage =
            serde_json::from_str(include_str!("../../example-configs/test-sensor.json")).unwrap();

        let res = app
            .oneshot(
                Request::post("/data")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&example_sensor).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), http::StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_fails_if_wrong_content_type() {
        let gateway: Arc<dyn Gateway> = Arc::new(MockGateway {});

        let app = create_app(gateway, &default_config());

        let example_sensor: SensorMessage =
            serde_json::from_str(include_str!("../../example-configs/test-sensor.json")).unwrap();

        let res = app
            .oneshot(
                Request::post("/data")
                    .body(serde_json::to_string(&example_sensor).unwrap().into())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), http::StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn test_malformed_sensor_fails() {
        let gateway: Arc<dyn Gateway> = Arc::new(MockGateway {});

        let app = create_app(gateway, &default_config());

        let example_sensor: SensorMessage =
            serde_json::from_str(include_str!("../../example-configs/test-sensor.json")).unwrap();

        let res = app
            .oneshot(
                Request::post("/data")
                    .header("Content-Type", "application/json")
                    .body(
                        serde_json::to_string(&example_sensor)
                            .unwrap()
                            .replace("temperature", "hot")
                            .into(),
                    )
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), http::StatusCode::UNPROCESSABLE_ENTITY);
    }
}
