pub mod bird;
pub mod challenge;
pub mod config;
pub mod ipalloc;
pub mod registry;
pub mod wireguard;

use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    let app = Router::new().route("/", get(root));

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
