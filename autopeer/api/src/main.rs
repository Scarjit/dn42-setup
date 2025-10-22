pub mod api;
pub mod bird;
pub mod challenge;
pub mod config;
pub mod ipalloc;
pub mod jwt;
pub mod middleware;
pub mod registry;
pub mod validation;
pub mod wireguard;

use axum::{routing::{get, post}, Router};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Load application config
    let app_config = Arc::new(
        config::AppConfig::from_env().expect("Failed to load configuration")
    );

    let app = Router::new()
        .route("/", get(root))
        .route("/peering/init", post(api::init_peering))
        .route("/peering/verify", post(api::verify_peering))
        .route("/peering/deploy", post(api::deploy_peering))
        .route("/peering/config/:asn", get(api::get_config))
        .with_state(app_config);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!(
        "AutoPeer API listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "AutoPeer API v0.1.0"
}
