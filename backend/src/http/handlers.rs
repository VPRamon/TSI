//! HTTP handlers for the REST API.
//!
//! Each handler corresponds to an API endpoint and delegates to the
//! existing service layer for business logic.

use axum::{
    extract::{Path, Query, State},
    Json,
    response::sse::{Event, Sse},
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::time::Duration;

use super::dto::{
    CompareQuery, CreateScheduleRequest, CreateScheduleResponse, HealthResponse,
    JobStatusResponse, ScheduleInfoDto, ScheduleListResponse, TrendsQuery,
};
use super::error::AppError;
use super::state::AppState;
use crate::api::ScheduleId;
use crate::db::services as db_services;

/// Result type for handlers.
pub type HandlerResult<T> = Result<Json<T>, AppError>;

// =============================================================================
// Health Check
// =============================================================================

/// GET /health
///
/// Health check endpoint to verify the service is running and database is accessible.
pub async fn health_check(State(state): State<AppState>) -> HandlerResult<HealthResponse> {
    let db_status = match db_services::health_check(state.repository.as_ref()).await {
        Ok(true) => "connected".to_string(),
        Ok(false) => "disconnected".to_string(),
        Err(e) => format!("error: {}", e),
    };

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        version: "v1".to_string(),
        database: db_status,
    }))
}

// =============================================================================
// Schedule CRUD
// =============================================================================

/// GET /v1/schedules
///
/// List all schedules in the database.
pub async fn list_schedules(State(state): State<AppState>) -> HandlerResult<ScheduleListResponse> {
    let schedules = db_services::list_schedules(state.repository.as_ref()).await?;
    
    let schedule_dtos: Vec<ScheduleInfoDto> = schedules.into_iter().map(Into::into).collect();
    let total = schedule_dtos.len();

    Ok(Json(ScheduleListResponse {
        schedules: schedule_dtos,
        total,
    }))
}

/// POST /v1/schedules
///
/// Create a new schedule asynchronously. Returns a job ID for tracking progress.
pub async fn create_schedule(
    State(state): State<AppState>,
    Json(request): Json<CreateScheduleRequest>,
) -> Result<(axum::http::StatusCode, Json<CreateScheduleResponse>), AppError> {
    // Convert JSON values to strings for the service layer
    let schedule_json_str = serde_json::to_string(&request.schedule_json)
        .map_err(|e| AppError::BadRequest(format!("Invalid schedule JSON: {}", e)))?;

    // Create a job for tracking progress
    let job_id = state.job_tracker.create_job();
    let response_job_id = job_id.clone();
    
    // Spawn background task to process the schedule
    let tracker = state.job_tracker.clone();
    let repo = state.repository.clone();
    let schedule_name = request.name.clone();
    let populate_analytics = request.populate_analytics;
    
    tokio::spawn(async move {
        let _ = crate::services::schedule_processor::process_schedule_async(
            job_id,
            tracker,
            repo,
            schedule_name,
            schedule_json_str,
            populate_analytics,
        )
        .await;
    });

    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(CreateScheduleResponse {
            job_id: response_job_id.clone(),
            message: format!("Schedule upload started. Track progress at /v1/jobs/{}/logs", response_job_id),
        }),
    ))
}

// =============================================================================
// Visualization Endpoints
// =============================================================================

/// GET /v1/schedules/{schedule_id}/sky-map
///
/// Get sky map visualization data for a schedule.
pub async fn get_sky_map(
    State(_state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::SkyMapData> {
    let schedule_id = ScheduleId::new(schedule_id);
    
    // Use the sync version wrapped in spawn_blocking for CPU-intensive work
    let data = tokio::task::spawn_blocking(move || {
        crate::services::py_get_sky_map_data_analytics(schedule_id)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/distributions
///
/// Get distribution analysis data for a schedule.
pub async fn get_distributions(
    State(_state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::DistributionData> {
    let schedule_id = ScheduleId::new(schedule_id);
    
    let data = tokio::task::spawn_blocking(move || {
        crate::services::py_get_distribution_data_analytics(schedule_id)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/visibility-map
///
/// Get visibility map data for a schedule.
pub async fn get_visibility_map(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::VisibilityMapData> {
    let schedule_id = ScheduleId::new(schedule_id);
    
    let data = state
        .repository
        .fetch_visibility_map_data(schedule_id)
        .await?;

    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/timeline
///
/// Get timeline visualization data for a schedule.
pub async fn get_timeline(
    State(_state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::ScheduleTimelineData> {
    let schedule_id = ScheduleId::new(schedule_id);
    
    let data = tokio::task::spawn_blocking(move || {
        crate::services::py_get_schedule_timeline_data(schedule_id)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/insights
///
/// Get insights analysis data for a schedule.
pub async fn get_insights(
    State(_state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::InsightsData> {
    let schedule_id = ScheduleId::new(schedule_id);
    
    let data = tokio::task::spawn_blocking(move || {
        crate::services::py_get_insights_data(schedule_id)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/trends
///
/// Get trends analysis data for a schedule.
pub async fn get_trends(
    State(_state): State<AppState>,
    Path(schedule_id): Path<i64>,
    Query(query): Query<TrendsQuery>,
) -> HandlerResult<crate::api::TrendsData> {
    let schedule_id = ScheduleId::new(schedule_id);
    let n_bins = query.bins.unwrap_or(10);
    let bandwidth = query.bandwidth.unwrap_or(0.5);
    let n_smooth_points = query.points.unwrap_or(12);
    
    let data = tokio::task::spawn_blocking(move || {
        crate::services::py_get_trends_data(schedule_id, n_bins, bandwidth, n_smooth_points)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/validation-report
///
/// Get validation report for a schedule.
pub async fn get_validation_report(
    State(_state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::ValidationReport> {
    let schedule_id = ScheduleId::new(schedule_id);
    
    let data = tokio::task::spawn_blocking(move || {
        crate::services::py_get_validation_report(schedule_id)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/compare/{other_id}
///
/// Compare two schedules.
pub async fn compare_schedules(
    State(_state): State<AppState>,
    Path((schedule_id, other_id)): Path<(i64, i64)>,
    Query(query): Query<CompareQuery>,
) -> HandlerResult<crate::api::CompareData> {
    let current_id = ScheduleId::new(schedule_id);
    let comparison_id = ScheduleId::new(other_id);
    let current_name = query.current_name.unwrap_or_else(|| "Schedule A".to_string());
    let comparison_name = query.comparison_name.unwrap_or_else(|| "Schedule B".to_string());
    
    let data = tokio::task::spawn_blocking(move || {
        crate::services::py_get_compare_data(current_id, comparison_id, current_name, comparison_name)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(data))
}

// =============================================================================
// Async Job Management
// =============================================================================

/// GET /v1/jobs/{job_id}
///
/// Get the current status and logs of a background job.
pub async fn get_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> HandlerResult<JobStatusResponse> {
    let job = state
        .job_tracker
        .get_job(&job_id)
        .ok_or_else(|| AppError::NotFound(format!("Job {} not found", job_id)))?;

    Ok(Json(JobStatusResponse {
        job_id: job.job_id,
        status: format!("{:?}", job.status).to_lowercase(),
        logs: job.logs,
        result: job.result,
    }))
}

/// GET /v1/jobs/{job_id}/logs
///
/// Stream job logs via Server-Sent Events (SSE).
pub async fn stream_job_logs(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    // Verify job exists
    if state.job_tracker.get_job(&job_id).is_none() {
        return Err(AppError::NotFound(format!("Job {} not found", job_id)));
    }

    let tracker = state.job_tracker.clone();
    let stream = async_stream::stream! {
        let mut last_log_count = 0;
        loop {
            // Get current logs
            let logs = tracker.get_logs(&job_id);
            
            // Send new logs since last check
            for log in logs.iter().skip(last_log_count) {
                let event_data = serde_json::to_string(log).unwrap_or_default();
                yield Ok(Event::default().data(event_data));
            }
            last_log_count = logs.len();

            // Check if job is complete
            if let Some(job) = tracker.get_job(&job_id) {
                if job.status != crate::services::job_tracker::JobStatus::Running {
                    // Send final status event
                    // Use serde serialization instead of Debug formatting to ensure
                    // consistent lowercase status values ("completed", "failed")
                    let final_event = serde_json::json!({
                        "status": job.status,
                        "result": job.result,
                    });
                    yield Ok(Event::default()
                        .event("complete")
                        .data(serde_json::to_string(&final_event).unwrap_or_default()));
                    break;
                }
            } else {
                break;
            }

            // Wait before checking again
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    };

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive"),
    ))
}
