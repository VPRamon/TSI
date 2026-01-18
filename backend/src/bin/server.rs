//! TSI HTTP Server Binary
//!
//! This is the main entry point for the TSI REST API server.
//! It initializes the repository, sets up the HTTP router, and starts serving requests.
//!
//! # Usage
//!
//! ```bash
//! # Run with local (in-memory) repository (default)
//! cargo run --bin tsi-server --features "local-repo,http-server"
//!
//! # Run with PostgreSQL repository
//! DATABASE_URL=postgres://user:pass@localhost/tsi \
//!   cargo run --bin tsi-server --features "postgres-repo,http-server"
//! ```
//!
//! # Environment Variables
//!
//! - `HOST`: Server host (default: 0.0.0.0)
//! - `PORT`: Server port (default: 8080)
//! - `DATABASE_URL`: PostgreSQL connection string (required for postgres-repo feature)
//! - `RUST_LOG`: Log level (default: info)

use std::env;
use std::net::SocketAddr;

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use tsi_rust::db;
use tsi_rust::http::{create_router, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(
            env::var("RUST_LOG")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(Level::INFO),
        )
        .with_target(true)
        .with_thread_ids(true)
        .init();

    info!("Starting TSI HTTP Server");

    // Initialize global repository once and reuse it across the app
    db::init_repository().map_err(|e| anyhow::anyhow!(e))?;
    let repository = std::sync::Arc::clone(db::get_repository()?);
    info!("Repository initialized successfully");

    // Create application state
    let state = AppState::new(repository);

    // Create router with all endpoints
    let app = create_router(state);

    // Determine bind address
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    info!("Server listening on http://{}", addr);
    info!("API documentation: http://{}/health", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// Repository initialization is handled by `db::init_repository()`.
