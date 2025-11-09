use axum::{routing::{get, post, delete}, Router};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

use tsi_backend::{routes, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    // Create shared application state
    let state = AppState::new();

    let app = Router::new()
        // Health check
        .route("/health", get(routes::health))
        
        // Dataset management
        .route("/api/v1/datasets/upload/csv", post(routes::upload_csv))
        .route("/api/v1/datasets/upload/json", post(routes::upload_json))
        .route("/api/v1/datasets/sample", post(routes::load_sample))
        .route("/api/v1/datasets/current/metadata", get(routes::get_current_metadata))
        .route("/api/v1/datasets/current", get(routes::get_current_dataset))
        .route("/api/v1/datasets/current", delete(routes::clear_dataset))
        
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    tracing::info!(%addr, "TSI backend listening");
    tracing::info!("Phase 1 complete - JSON upload & preprocessing ready");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
