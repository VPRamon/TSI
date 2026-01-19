//! Router configuration for the HTTP API.
//!
//! This module sets up all routes, middleware (CORS, compression, tracing),
//! and creates the axum router ready for serving.

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use super::handlers;
use super::state::AppState;

/// Create the main application router with all routes and middleware.
pub fn create_router(state: AppState) -> Router {
    // CORS configuration - permissive for development, should be restricted in production
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the API router with versioned endpoints
    let api_v1 = Router::new()
        // Schedule CRUD
        .route("/schedules", get(handlers::list_schedules))
        .route("/schedules", post(handlers::create_schedule))
        // Job management
        .route("/jobs/{job_id}", get(handlers::get_job_status))
        .route("/jobs/{job_id}/logs", get(handlers::stream_job_logs))
        // Visualization endpoints
        .route("/schedules/{schedule_id}/sky-map", get(handlers::get_sky_map))
        .route("/schedules/{schedule_id}/distributions", get(handlers::get_distributions))
        .route("/schedules/{schedule_id}/visibility-map", get(handlers::get_visibility_map))
        .route("/schedules/{schedule_id}/timeline", get(handlers::get_timeline))
        .route("/schedules/{schedule_id}/insights", get(handlers::get_insights))
        .route("/schedules/{schedule_id}/trends", get(handlers::get_trends))
        .route("/schedules/{schedule_id}/validation-report", get(handlers::get_validation_report))
        .route("/schedules/{schedule_id}/compare/{other_id}", get(handlers::compare_schedules));

    // Combine all routes
    Router::new()
        .route("/health", get(handlers::health_check))
        .nest("/v1", api_v1)
        // Allow large schedule payloads during uploads.
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::db::repositories::LocalRepository;

    #[test]
    fn test_router_creation() {
        let repo = Arc::new(LocalRepository::new()) as Arc<dyn crate::db::repository::FullRepository>;
        let state = AppState::new(repo);
        let _router = create_router(state);
        // If we got here, router was created successfully
    }
}
