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

use super::extensions::BackendExtensions;
use super::handlers;
use super::state::AppState;

/// Create the main application router with all routes and middleware,
/// using the default (empty) [`BackendExtensions`].
pub fn create_router(state: AppState) -> Router {
    create_router_with_extensions(state, BackendExtensions::default())
}

/// Create the main application router with integrator-supplied
/// extensions merged in.
pub fn create_router_with_extensions(state: AppState, mut extensions: BackendExtensions) -> Router {
    // Stash the (still-mutable) extensions clone on AppState so handlers
    // can look up trace validators at request time. Routes are taken
    // out below before the registry is shared.
    let extra_routes = extensions.take_extra_routes();
    let state = state.with_extensions(extensions);
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
        .route("/schedules/{schedule_id}", get(handlers::get_schedule))
        .route(
            "/schedules/{schedule_id}",
            delete(handlers::delete_schedule),
        )
        .route("/schedules/{schedule_id}", patch(handlers::update_schedule))
        // Environment CRUD
        .route("/environments", get(handlers::list_environments))
        .route("/environments", post(handlers::create_environment))
        .route(
            "/environments/{environment_id}",
            get(handlers::get_environment),
        )
        .route(
            "/environments/{environment_id}",
            delete(handlers::delete_environment),
        )
        .route(
            "/environments/{environment_id}/schedules",
            post(handlers::bulk_import_schedules),
        )
        .route(
            "/schedules/{schedule_id}/environment",
            delete(handlers::unassign_schedule_environment),
        )
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
            "/schedules/{schedule_id}/algorithm_trace",
            get(handlers::get_algorithm_trace),
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
        )
        // KPI summaries (A1)
        .route(
            "/schedules/{schedule_id}/kpis",
            get(handlers::get_schedule_kpis),
        )
        .route(
            "/environments/{environment_id}/kpis",
            get(handlers::get_environment_kpis),
        )
        // Diagnostics
        .route("/_health/db", get(handlers::db_diagnostics));

    // Merge integrator-contributed routes under the same `/v1` prefix.
    let api_v1 = match extra_routes {
        Some(extra) => api_v1.merge(extra),
        None => api_v1,
    };

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
    use crate::api::{
        Constraints, ModifiedJulianDate, Period, Schedule, SchedulingBlock, SchedulingBlockId,
    };
    use crate::db::repositories::LocalRepository;
    use crate::db::services as db_services;
    use crate::siderust::coordinates::centers::Geodetic;
    use crate::siderust::coordinates::frames::ECEF;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use qtty::{Degrees, Meters, Seconds};
    use std::sync::Arc;
    use tower::util::ServiceExt;

    async fn wait_for_job_completion(state: &AppState, job_id: &str) {
        for _ in 0..100 {
            if let Some(job) = state.job_tracker.get_job(job_id) {
                if job.status != crate::services::job_tracker::JobStatus::Running {
                    return;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
    }

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

    #[tokio::test]
    async fn get_schedule_returns_canonical_native_json() {
        let repo =
            Arc::new(LocalRepository::new()) as Arc<dyn crate::db::repository::FullRepository>;
        let state = AppState::new(Arc::clone(&repo));
        let app = create_router(state);

        let dark_period = Period {
            start: ModifiedJulianDate::new(60694.1),
            end: ModifiedJulianDate::new(60694.3),
        };

        let visibility_period = Period {
            start: ModifiedJulianDate::new(60694.15),
            end: ModifiedJulianDate::new(60694.2),
        };

        let schedule = Schedule {
            id: Some(999),
            name: "canonical-export-test".to_string(),
            checksum: "test-checksum".to_string(),
            schedule_period: Period {
                start: ModifiedJulianDate::new(60694.0),
                end: ModifiedJulianDate::new(60701.0),
            },
            dark_periods: vec![dark_period],
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.8892),
                Degrees::new(28.7624),
                Meters::new(2396.0),
            ),
            astronomical_nights: vec![dark_period],
            blocks: vec![SchedulingBlock {
                id: Some(SchedulingBlockId::new(42)),
                original_block_id: "block-1".to_string(),
                block_name: "Block One".to_string(),
                target_ra: Degrees::new(158.03),
                target_dec: Degrees::new(-68.03),
                constraints: Constraints {
                    min_alt: Degrees::new(60.0),
                    max_alt: Degrees::new(90.0),
                    min_az: Degrees::new(0.0),
                    max_az: Degrees::new(360.0),
                    fixed_time: None,
                },
                priority: 8.5,
                min_observation: Seconds::new(3600.0),
                requested_duration: Seconds::new(7200.0),
                visibility_periods: vec![visibility_period],
                scheduled_period: Some(visibility_period),
            }],
        };

        let metadata = db_services::store_schedule(repo.as_ref(), &schedule)
            .await
            .unwrap();

        let request = Request::builder()
            .method("GET")
            .uri(format!("/v1/schedules/{}", metadata.schedule_id.value()))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(payload.get("name").is_some());
        assert!(payload.get("schedule_period").is_some());
        assert!(payload.get("geographic_location").is_some());
        assert!(payload.get("blocks").is_some());
        assert!(payload.get("possible_periods").is_some());

        // Internal persistence fields should not leak in exported JSON.
        assert!(payload.get("id").is_none());
        assert!(payload.get("checksum").is_none());
        assert!(payload.get("astronomical_nights").is_none());
        assert!(payload["blocks"][0].get("id").is_none());

        let exported_json = serde_json::to_string(&payload).unwrap();
        crate::models::schedule::validate_schedule_json_str(&exported_json)
            .expect("canonical export should validate as native TSI schedule JSON");
    }

    #[tokio::test]
    async fn create_schedule_rejects_duplicate_name() {
        let repo =
            Arc::new(LocalRepository::new()) as Arc<dyn crate::db::repository::FullRepository>;
        let state = AppState::new(Arc::clone(&repo));
        let app = create_router(state.clone());

        let first_request = Request::builder()
            .method("POST")
            .uri("/v1/schedules")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "name": "duplicate-name-test",
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
                                "original_block_id": "dup-block-1",
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

        let first_response = app.clone().oneshot(first_request).await.unwrap();
        assert_eq!(first_response.status(), StatusCode::ACCEPTED);
        let first_body = to_bytes(first_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let first_payload: serde_json::Value = serde_json::from_slice(&first_body).unwrap();
        let first_job_id = first_payload["job_id"].as_str().unwrap();
        wait_for_job_completion(&state, first_job_id).await;
        let first_job = state
            .job_tracker
            .get_job(first_job_id)
            .expect("first upload job should exist");
        assert_eq!(
            first_job.status,
            crate::services::job_tracker::JobStatus::Completed
        );

        let second_request = Request::builder()
            .method("POST")
            .uri("/v1/schedules")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "name": "duplicate-name-test",
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
                                "id": 2,
                                "original_block_id": "dup-block-2",
                                "target_ra": 10.0,
                                "target_dec": 20.0,
                                "constraints": {
                                    "min_alt": 30.0,
                                    "max_alt": 90.0,
                                    "min_az": 0.0,
                                    "max_az": 360.0,
                                    "fixed_time": null
                                },
                                "priority": 2.0,
                                "min_observation": 1800.0,
                                "requested_duration": 3600.0,
                                "visibility_periods": [],
                                "scheduled_period": null
                            }
                        ]
                    }
                })
                .to_string(),
            ))
            .unwrap();

        let second_response = app.oneshot(second_request).await.unwrap();
        assert_eq!(second_response.status(), StatusCode::BAD_REQUEST);

        let second_body = to_bytes(second_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let second_payload: serde_json::Value = serde_json::from_slice(&second_body).unwrap();
        assert!(second_payload["message"]
            .as_str()
            .unwrap_or_default()
            .contains("already exists"));
    }

    #[tokio::test]
    async fn update_schedule_rejects_duplicate_name() {
        let repo =
            Arc::new(LocalRepository::new()) as Arc<dyn crate::db::repository::FullRepository>;
        let state = AppState::new(Arc::clone(&repo));
        let app = create_router(state);

        let schedule_a = Schedule {
            id: None,
            name: "rename-dup-a".to_string(),
            checksum: "rename-dup-a-checksum".to_string(),
            schedule_period: Period {
                start: ModifiedJulianDate::new(60694.0),
                end: ModifiedJulianDate::new(60701.0),
            },
            dark_periods: vec![],
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.8892),
                Degrees::new(28.7624),
                Meters::new(2396.0),
            ),
            astronomical_nights: vec![],
            blocks: vec![],
        };

        let schedule_b = Schedule {
            id: None,
            name: "rename-dup-b".to_string(),
            checksum: "rename-dup-b-checksum".to_string(),
            schedule_period: Period {
                start: ModifiedJulianDate::new(60694.0),
                end: ModifiedJulianDate::new(60701.0),
            },
            dark_periods: vec![],
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.8892),
                Degrees::new(28.7624),
                Meters::new(2396.0),
            ),
            astronomical_nights: vec![],
            blocks: vec![],
        };

        let metadata_a = db_services::store_schedule(repo.as_ref(), &schedule_a)
            .await
            .unwrap();
        let _metadata_b = db_services::store_schedule(repo.as_ref(), &schedule_b)
            .await
            .unwrap();

        let request = Request::builder()
            .method("PATCH")
            .uri(format!("/v1/schedules/{}", metadata_a.schedule_id.value()))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "name": "rename-dup-b"
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(payload["message"]
            .as_str()
            .unwrap_or_default()
            .contains("already exists"));
    }
}
