//! Router configuration for the HTTP API.
//!
//! This module sets up all routes, middleware (CORS, compression, tracing),
//! and creates the axum router ready for serving.

use axum::{
    extract::DefaultBodyLimit,
    routing::{delete, get, patch, post},
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
        .route(
            "/schedules/{schedule_id}",
            delete(handlers::delete_schedule),
        )
        .route("/schedules/{schedule_id}", patch(handlers::update_schedule))
        // Job management
        .route("/jobs/{job_id}", get(handlers::get_job_status))
        .route("/jobs/{job_id}/logs", get(handlers::stream_job_logs))
        // Visualization endpoints
        .route(
            "/schedules/{schedule_id}/sky-map",
            get(handlers::get_sky_map),
        )
        .route(
            "/schedules/{schedule_id}/distributions",
            get(handlers::get_distributions),
        )
        .route(
            "/schedules/{schedule_id}/visibility-map",
            get(handlers::get_visibility_map),
        )
        .route(
            "/schedules/{schedule_id}/visibility-histogram",
            get(handlers::get_visibility_histogram),
        )
        .route(
            "/schedules/{schedule_id}/alt-az",
            post(handlers::compute_alt_az),
        )
        .route(
            "/schedules/{schedule_id}/timeline",
            get(handlers::get_timeline),
        )
        .route(
            "/schedules/{schedule_id}/insights",
            get(handlers::get_insights),
        )
        .route(
            "/schedules/{schedule_id}/fragmentation",
            get(handlers::get_fragmentation),
        )
        .route("/schedules/{schedule_id}/trends", get(handlers::get_trends))
        .route(
            "/schedules/{schedule_id}/validation-report",
            get(handlers::get_validation_report),
        )
        .route(
            "/schedules/{schedule_id}/compare/{other_id}",
            get(handlers::compare_schedules),
        );

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
    use crate::db::repositories::LocalRepository;
    use crate::db::services as db_services;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use std::sync::Arc;
    use tower::util::ServiceExt;

    #[test]
    fn test_router_creation() {
        let repo =
            Arc::new(LocalRepository::new()) as Arc<dyn crate::db::repository::FullRepository>;
        let state = AppState::new(repo);
        let _router = create_router(state);
        // If we got here, router was created successfully
    }

    #[tokio::test]
    async fn default_router_accepts_native_schedule_uploads() {
        let repo =
            Arc::new(LocalRepository::new()) as Arc<dyn crate::db::repository::FullRepository>;
        let state = AppState::new(Arc::clone(&repo));
        let app = create_router(state.clone());

        let request = Request::builder()
            .method("POST")
            .uri("/v1/schedules")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "name": "router-upload",
                    "populate_analytics": false,
                    "schedule_json": {
                        "name": "",
                        "geographic_location": {
                            "lat_deg": 28.7624,
                            "lon_deg": -17.8892,
                            "height": 2396.0
                        },
                        "blocks": [
                            {
                                "id": 1,
                                "original_block_id": "block-1",
                                "target_ra": 158.03,
                                "target_dec": -68.03,
                                "constraints": {
                                    "min_alt": 60.0,
                                    "max_alt": 90.0,
                                    "min_az": 0.0,
                                    "max_az": 360.0,
                                    "fixed_time": null
                                },
                                "priority": 8.5,
                                "min_observation": 3600.0,
                                "requested_duration": 7200.0,
                                "visibility_periods": [],
                                "scheduled_period": null
                            }
                        ]
                    }
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let job_id = payload["job_id"].as_str().unwrap().to_string();

        for _ in 0..100 {
            if let Some(job) = state.job_tracker.get_job(&job_id) {
                if job.status != crate::services::job_tracker::JobStatus::Running {
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        let job = state.job_tracker.get_job(&job_id).unwrap();
        assert_eq!(
            job.status,
            crate::services::job_tracker::JobStatus::Completed
        );

        let schedules = db_services::list_schedules(repo.as_ref()).await.unwrap();
        assert_eq!(schedules.len(), 1);
        assert_eq!(schedules[0].schedule_name, "router-upload");
    }

    #[tokio::test]
    async fn default_router_rejects_invalid_schedule_payloads() {
        let repo =
            Arc::new(LocalRepository::new()) as Arc<dyn crate::db::repository::FullRepository>;
        let state = AppState::new(Arc::clone(&repo));
        let app = create_router(state);

        let request = Request::builder()
            .method("POST")
            .uri("/v1/schedules")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "name": "invalid-upload",
                    "populate_analytics": false,
                    "schedule_json": {
                        "missing": "blocks"
                    }
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let schedules = db_services::list_schedules(repo.as_ref()).await.unwrap();
        assert!(schedules.is_empty());
    }
}
