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

pub async fn run_server_with(server_configs: ServerConfiguration) {
    let pixy_configs = parse_configs(&server_configs.config_file).unwrap();

    let gateway: Arc<dyn Gateway> = Arc::new(SensorGateway::from(pixy_configs));

    let app = axum::Router::new()
        .route("/data", post(handler))
        .route("/healthz", get(|| async { StatusCode::OK }))
        .with_state(gateway);

    let app = if server_configs.enable_echo {
        app.route("/echo", post(echo))
    } else {
        app
    };

    let bind_address = format!("0.0.0.0:{}", server_configs.port);

    println!(
        r#"
          _____                    _____                    _____                    _____                    _____          
         /\    \                  /\    \                  /\    \                  /\    \                  /\    \         
        /::\    \                /::\    \                /::\____\                /::\____\                /::\    \        
        \:::\    \              /::::\    \              /::::|   |               /:::/    /               /::::\    \       
         \:::\    \            /::::::\    \            /:::::|   |              /:::/    /               /::::::\    \      
          \:::\    \          /:::/\:::\    \          /::::::|   |             /:::/    /               /:::/\:::\    \     
           \:::\    \        /:::/__\:::\    \        /:::/|::|   |            /:::/    /               /:::/__\:::\    \    
           /::::\    \      /::::\   \:::\    \      /:::/ |::|   |           /:::/    /                \:::\   \:::\    \   
  _____   /::::::\    \    /::::::\   \:::\    \    /:::/  |::|   | _____    /:::/    /      _____    ___\:::\   \:::\    \  
 /\    \ /:::/\:::\    \  /:::/\:::\   \:::\    \  /:::/   |::|   |/\    \  /:::/____/      /\    \  /\   \:::\   \:::\    \ 
/::\    /:::/  \:::\____\/:::/  \:::\   \:::\____\/:: /    |::|   /::\____\|:::|    /      /::\____\/::\   \:::\   \:::\____\
\:::\  /:::/    \::/    /\::/    \:::\  /:::/    /\::/    /|::|  /:::/    /|:::|____\     /:::/    /\:::\   \:::\   \::/    /
 \:::\/:::/    / \/____/  \/____/ \:::\/:::/    /  \/____/ |::| /:::/    /  \:::\    \   /:::/    /  \:::\   \:::\   \/____/ 
  \::::::/    /                    \::::::/    /           |::|/:::/    /    \:::\    \ /:::/    /    \:::\   \:::\    \     
   \::::/    /                      \::::/    /            |::::::/    /      \:::\    /:::/    /      \:::\   \:::\____\    
    \::/    /                       /:::/    /             |:::::/    /        \:::\__/:::/    /        \:::\  /:::/    /    
     \/____/                       /:::/    /              |::::/    /          \::::::::/    /          \:::\/:::/    /     
                                  /:::/    /               /:::/    /            \::::::/    /            \::::::/    /      
                                 /:::/    /               /:::/    /              \::::/    /              \::::/    /       
                                 \::/    /                \::/    /                \::/____/                \::/    /        
                                  \/____/                  \/____/                  ~~                       \/____/         


    "#
    );

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
