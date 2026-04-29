//! HTTP handlers for the REST API.
//!
//! Each handler corresponds to an API endpoint and delegates to the
//! existing service layer for business logic.

use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, Sse},
    Json,
};
use futures::stream::Stream;
use serde::Serialize;
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use super::dto::{
    BulkDeleteSchedulesRequest, BulkDeleteSchedulesResponse, CompareQuery, CreateScheduleRequest,
    CreateScheduleResponse, DeleteScheduleResponse, HealthResponse, JobStatusResponse,
    ListSchedulesParams, ScheduleInfoDto, ScheduleListResponse, TrendsQuery, UpdateScheduleRequest,
    VisibilityBin, VisibilityHistogramQuery,
};
use super::error::AppError;
use super::state::AppState;
use crate::api::{AltAzData, AltAzRequest, ScheduleId, SchedulingBlock};
use crate::db::services as db_services;
use crate::http::extensions::BackendExtensions;
use crate::services::schedule_processor::TraceValidatorFn;

/// Build a [`TraceValidatorFn`] from the registered backend extensions,
/// or return `None` when no validators are registered. Run on the hot
/// path of every schedule upload, so kept O(validators).
fn build_trace_validator(extensions: Arc<BackendExtensions>) -> Option<TraceValidatorFn> {
    if extensions.trace_validators.is_empty() {
        return None;
    }
    Some(Arc::new(
        move |algorithm: &str, summary: &serde_json::Value| {
            if let Some(validator) = extensions.trace_validator_for(algorithm) {
                validator.validate_summary(summary)
            } else {
                Ok(())
            }
        },
    ))
}

/// Result type for handlers.
pub type HandlerResult<T> = Result<Json<T>, AppError>;

fn normalize_schedule_name(name: &str) -> String {
    name.trim().to_lowercase()
}

fn has_duplicate_schedule_name(
    schedules: &[crate::api::ScheduleInfo],
    candidate_name: &str,
    exclude_id: Option<ScheduleId>,
) -> bool {
    let normalized_candidate = normalize_schedule_name(candidate_name);

    schedules.iter().any(|schedule| {
        if let Some(exclude) = exclude_id {
            if schedule.schedule_id == exclude {
                return false;
            }
        }

        normalize_schedule_name(&schedule.schedule_name) == normalized_candidate
    })
}

#[derive(Debug, Clone, Serialize)]
struct NativeScheduleExport {
    name: String,
    schedule_period: crate::api::Period,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dark_periods: Vec<crate::api::Period>,
    geographic_location: crate::api::GeographicLocation,
    blocks: Vec<NativeScheduleBlockExport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    possible_periods: Option<BTreeMap<String, Vec<crate::api::Period>>>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeScheduleBlockExport {
    original_block_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    block_name: Option<String>,
    target_ra: qtty::Degrees,
    target_dec: qtty::Degrees,
    constraints: crate::api::Constraints,
    priority: f64,
    min_observation: qtty::Seconds,
    requested_duration: qtty::Seconds,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    visibility_periods: Vec<crate::api::Period>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_period: Option<crate::api::Period>,
}

fn build_possible_periods_map(
    blocks: &[crate::api::SchedulingBlock],
) -> Option<BTreeMap<String, Vec<crate::api::Period>>> {
    let mut map = BTreeMap::new();

    for block in blocks {
        if block.visibility_periods.is_empty() {
            continue;
        }

        let key = block.original_block_id.trim();
        if key.is_empty() {
            continue;
        }

        map.insert(key.to_string(), block.visibility_periods.clone());
    }

    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

impl From<&crate::api::Schedule> for NativeScheduleExport {
    fn from(schedule: &crate::api::Schedule) -> Self {
        Self {
            name: schedule.name.clone(),
            schedule_period: schedule.schedule_period,
            dark_periods: schedule.dark_periods.clone(),
            geographic_location: schedule.geographic_location,
            blocks: schedule
                .blocks
                .iter()
                .map(|block| NativeScheduleBlockExport {
                    original_block_id: block.original_block_id.clone(),
                    block_name: if block.block_name.trim().is_empty() {
                        None
                    } else {
                        Some(block.block_name.clone())
                    },
                    target_ra: block.target_ra,
                    target_dec: block.target_dec,
                    constraints: block.constraints.clone(),
                    priority: block.priority,
                    min_observation: block.min_observation,
                    requested_duration: block.requested_duration,
                    visibility_periods: block.visibility_periods.clone(),
                    scheduled_period: block.scheduled_period,
                })
                .collect(),
            possible_periods: build_possible_periods_map(&schedule.blocks),
        }
    }
}

fn block_matches_visibility_histogram_query(
    block: &SchedulingBlock,
    query: &VisibilityHistogramQuery,
) -> bool {
    if let Some(min_p) = query.priority_min {
        if block.priority < min_p {
            return false;
        }
    }
    if let Some(max_p) = query.priority_max {
        if block.priority > max_p {
            return false;
        }
    }
    if let Some(ref ids) = query.block_ids {
        if let Some(id) = block.id {
            if !ids.contains(&id.value()) {
                return false;
            }
        }
    }
    if let Some(scheduled) = query.scheduled {
        if block.scheduled_period.is_some() != scheduled {
            return false;
        }
    }

    true
}

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

/// GET /v1/_health/db
///
/// Diagnostics endpoint that surfaces the bulk-import latency ring buffer
/// and current concurrency knob. Used to investigate "missing schedules"
/// reports during bulk uploads (R1 instrumentation): a sudden jump in
/// `duration_ms` or `rejected` count, or growth of `recorded_at_unix_ms`
/// gaps, points at pool starvation or chunk-level failures. The endpoint
/// is read-only and intentionally cheap (no DB round-trip).
pub async fn db_diagnostics(
    State(state): State<AppState>,
) -> HandlerResult<super::dto::DbDiagnosticsResponse> {
    let samples = state
        .bulk_import_latencies
        .snapshot()
        .into_iter()
        .map(|s| super::dto::BulkImportSampleDto {
            duration_ms: s.duration_ms,
            items: s.items,
            created: s.created,
            rejected: s.rejected,
            concurrency: s.concurrency,
            environment_id: s.environment_id,
            recorded_at_unix_ms: s.recorded_at_unix_ms,
        })
        .collect::<Vec<_>>();
    Ok(Json(super::dto::DbDiagnosticsResponse {
        bulk_import_concurrency: state.bulk_import_concurrency,
        recent_bulk_imports: samples,
    }))
}

// =============================================================================
// Schedule CRUD
// =============================================================================

/// GET /v1/schedules
///
/// List schedules in the database with pagination metadata.
///
/// Query parameters:
/// - `limit`  (optional, default 200, capped at 1000)
/// - `offset` (optional, default 0)
///
/// The response envelope is `{ items, total, limit, offset }` and an
/// `x-total-count` header echoes `total` for HTTP clients that prefer
/// header-based pagination metadata.
pub async fn list_schedules(
    State(state): State<AppState>,
    Query(params): Query<ListSchedulesParams>,
) -> Result<axum::response::Response, AppError> {
    const DEFAULT_LIMIT: u32 = 200;
    const MAX_LIMIT: u32 = 1000;

    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = params.offset.unwrap_or(0);

    let (rows, total) =
        db_services::list_schedules_with_algorithms(state.repository.as_ref(), limit, offset)
            .await?;

    let items: Vec<ScheduleInfoDto> = rows
        .into_iter()
        .map(|(info, algo)| {
            let mut dto: ScheduleInfoDto = info.into();
            if let Some(algorithm) = algo {
                dto.schedule_metadata = Some(super::dto::ScheduleMetadataDto { algorithm });
            }
            dto
        })
        .collect();

    let body = ScheduleListResponse {
        items,
        total,
        limit,
        offset,
    };

    use axum::http::header::{HeaderName, HeaderValue};
    use axum::response::IntoResponse;

    let mut response = Json(body).into_response();
    if let Ok(value) = HeaderValue::from_str(&total.to_string()) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("x-total-count"), value);
    }
    Ok(response)
}

/// GET /v1/schedules/{schedule_id}
///
/// Get a single schedule by ID.
pub async fn get_schedule(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<serde_json::Value> {
    let schedule_id = ScheduleId::new(schedule_id);
    let schedule = db_services::get_schedule(state.repository.as_ref(), schedule_id).await?;

    // Always export canonical native TSI schedule JSON, independent of the
    // adapter used at import time.
    let export = NativeScheduleExport::from(&schedule);

    let export_json = serde_json::to_string(&export).map_err(|e| {
        AppError::Internal(format!(
            "Failed to serialize canonical schedule export for schedule {}: {}",
            schedule_id, e
        ))
    })?;

    crate::models::schedule::validate_schedule_json_str(&export_json).map_err(|e| {
        AppError::Internal(format!(
            "Canonical schedule export failed validation for schedule {}: {}",
            schedule_id, e
        ))
    })?;

    let export_value = serde_json::from_str(&export_json).map_err(|e| {
        AppError::Internal(format!(
            "Failed to materialize canonical schedule export for schedule {}: {}",
            schedule_id, e
        ))
    })?;

    Ok(Json(export_value))
}

/// POST /v1/schedules
///
/// Create a new schedule asynchronously. Returns a job ID for tracking progress.
pub async fn create_schedule(
    State(state): State<AppState>,
    Json(mut request): Json<CreateScheduleRequest>,
) -> Result<(axum::http::StatusCode, Json<CreateScheduleResponse>), AppError> {
    request.name = request.name.trim().to_string();

    // Apply geographic location override when provided. This replaces any
    // `geographic_location` embedded in the schedule JSON, allowing callers
    // to select a well-known site (e.g. OBS-N, OBS-S) at load time.
    if let Some(ref loc) = request.location_override {
        let loc_value = serde_json::to_value(loc)
            .map_err(|e| AppError::BadRequest(format!("Invalid location: {}", e)))?;
        if let Some(obj) = request.schedule_json.as_object_mut() {
            obj.insert("geographic_location".to_string(), loc_value);
        }
    }

    // Convert JSON values to strings for the service layer
    let schedule_json_str = serde_json::to_string(&request.schedule_json)
        .map_err(|e| AppError::BadRequest(format!("Invalid schedule JSON: {}", e)))?;

    state
        .import_adapter
        .validate_schedule_payload(&schedule_json_str)
        .map_err(|e| AppError::BadRequest(format!("Invalid schedule payload: {}", e)))?;

    // Validate period override up front before spawning the async task.
    if let Some(ref ov) = request.schedule_period_override {
        if ov.start_mjd >= ov.end_mjd {
            return Err(AppError::BadRequest(format!(
                "schedule_period_override: start_mjd ({}) must be strictly less than end_mjd ({})",
                ov.start_mjd, ov.end_mjd
            )));
        }
    }

    if !request.name.is_empty() {
        let schedules = db_services::list_schedules(state.repository.as_ref()).await?;
        if has_duplicate_schedule_name(&schedules, &request.name, None) {
            return Err(AppError::BadRequest(format!(
                "A schedule named '{}' already exists. Please choose a different name.",
                request.name
            )));
        }
    }

    // Create a job for tracking progress
    let job_id = state.job_tracker.create_job();
    let response_job_id = job_id.clone();

    // Spawn background task to process the schedule
    let tracker = state.job_tracker.clone();
    let repo = state.repository.clone();
    let import_adapter = state.import_adapter.clone();
    let schedule_name = request.name.clone();
    let populate_analytics = request.populate_analytics;
    let period_override = request.schedule_period_override.clone();
    let algorithm_trace_jsonl = request.algorithm_trace_jsonl.clone();
    let trace_validator = build_trace_validator(state.extensions.clone());

    tokio::spawn(async move {
        let _ = crate::services::schedule_processor::process_schedule_async(
            job_id,
            tracker,
            repo,
            import_adapter,
            schedule_name,
            schedule_json_str,
            populate_analytics,
            period_override,
            algorithm_trace_jsonl,
            trace_validator,
        )
        .await;
    });

    Ok((
        axum::http::StatusCode::ACCEPTED,
        Json(CreateScheduleResponse {
            job_id: response_job_id.clone(),
            message: format!(
                "Schedule upload started. Track progress at /v1/jobs/{}/logs",
                response_job_id
            ),
        }),
    ))
}

/// DELETE /v1/schedules/{schedule_id}
///
/// Delete a schedule and all associated data.
pub async fn delete_schedule(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<DeleteScheduleResponse> {
    let schedule_id = ScheduleId::new(schedule_id);
    db_services::delete_schedule(state.repository.as_ref(), schedule_id).await?;

    Ok(Json(DeleteScheduleResponse {
        message: format!("Schedule {} deleted successfully", schedule_id),
    }))
}

/// POST /v1/schedules/bulk-delete
///
/// Delete many schedules in a single transaction. Considerably faster than
/// looping the single-id endpoint when the user removes a selection of
/// schedules at once.
pub async fn bulk_delete_schedules(
    State(state): State<AppState>,
    Json(request): Json<BulkDeleteSchedulesRequest>,
) -> HandlerResult<BulkDeleteSchedulesResponse> {
    let ids: Vec<ScheduleId> = request
        .schedule_ids
        .into_iter()
        .map(ScheduleId::new)
        .collect();
    let deleted_count = db_services::bulk_delete_schedules(state.repository.as_ref(), &ids).await?;
    Ok(Json(BulkDeleteSchedulesResponse {
        deleted_count,
        message: format!(
            "Deleted {} schedule{}",
            deleted_count,
            if deleted_count == 1 { "" } else { "s" }
        ),
    }))
}

/// PATCH /v1/schedules/{schedule_id}
///
/// Update schedule metadata (name and/or observer location).
pub async fn update_schedule(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
    Json(mut request): Json<UpdateScheduleRequest>,
) -> HandlerResult<ScheduleInfoDto> {
    if request.name.is_none() && request.location.is_none() {
        return Err(AppError::BadRequest(
            "At least one of 'name' or 'location' must be provided".to_string(),
        ));
    }

    let schedule_id = ScheduleId::new(schedule_id);

    if let Some(name) = request.name.as_ref() {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(AppError::BadRequest(
                "Schedule name cannot be empty".to_string(),
            ));
        }

        let schedules = db_services::list_schedules(state.repository.as_ref()).await?;
        if has_duplicate_schedule_name(&schedules, trimmed_name, Some(schedule_id)) {
            return Err(AppError::BadRequest(format!(
                "A schedule named '{}' already exists. Please choose a different name.",
                trimmed_name
            )));
        }

        request.name = Some(trimmed_name.to_string());
    }

    let info = db_services::update_schedule_metadata(
        state.repository.as_ref(),
        schedule_id,
        request.name,
        request.location,
    )
    .await?;

    Ok(Json(info.into()))
}

// =============================================================================
// Visualization Endpoints
// =============================================================================

/// GET /v1/schedules/{schedule_id}/sky-map
///
/// Get sky map visualization data for a schedule.
pub async fn get_sky_map(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::SkyMapData> {
    let schedule_id = ScheduleId::new(schedule_id);
    let data = crate::services::sky_map::get_sky_map_data(state.repository.as_ref(), schedule_id)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/distributions
///
/// Get distribution analysis data for a schedule.
pub async fn get_distributions(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::DistributionData> {
    let schedule_id = ScheduleId::new(schedule_id);
    let data = crate::services::distributions::get_distribution_data(
        state.repository.as_ref(),
        schedule_id,
    )
    .await
    .map_err(AppError::Internal)?;
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

/// GET /v1/schedules/{schedule_id}/visibility-histogram
///
/// Get visibility histogram data for a schedule with optional filters.
pub async fn get_visibility_histogram(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
    Query(query): Query<VisibilityHistogramQuery>,
) -> HandlerResult<Vec<VisibilityBin>> {
    use crate::db::models::BlockHistogramData;
    use crate::services::visibility::compute_visibility_histogram_rust;

    let schedule_id = ScheduleId::new(schedule_id);

    // Get schedule time range
    let time_range = state
        .repository
        .get_schedule_time_range(schedule_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("No time range found for schedule {}", schedule_id))
        })?;

    // Convert MJD to Unix timestamps
    const MJD_EPOCH_UNIX: i64 = -3506716800;
    let start_unix = MJD_EPOCH_UNIX + (time_range.start.value() * 86400.0) as i64;
    let end_unix = MJD_EPOCH_UNIX + (time_range.end.value() * 86400.0) as i64;

    // Determine bin duration
    let bin_duration_seconds = if let Some(minutes) = query.bin_duration_minutes {
        minutes * 60
    } else {
        let num_bins = query.num_bins.unwrap_or(50);
        let time_range_seconds = end_unix - start_unix;
        std::cmp::max(1, time_range_seconds / num_bins as i64)
    };

    // Fetch blocks with visibility data
    let blocks = state
        .repository
        .get_blocks_for_schedule(schedule_id)
        .await?;

    // Convert to histogram data format and apply filters
    let histogram_blocks: Vec<BlockHistogramData> = blocks
        .into_iter()
        .filter(|b| block_matches_visibility_histogram_query(b, &query))
        .map(|b| BlockHistogramData {
            scheduling_block_id: b.id.map(|id| id.value()).unwrap_or(0),
            priority: b.priority,
            visibility_periods: Some(b.visibility_periods),
        })
        .collect();

    // Compute histogram using the service
    let bins = tokio::task::spawn_blocking(move || {
        compute_visibility_histogram_rust(
            histogram_blocks.into_iter(),
            start_unix,
            end_unix,
            bin_duration_seconds,
            query.priority_min,
            query.priority_max,
        )
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))?
    .map_err(|e| AppError::Internal(format!("Histogram computation error: {}", e)))?;

    Ok(Json(bins))
}

/// POST /v1/schedules/{schedule_id}/alt-az
///
/// Compute altitude and azimuth curves for selected targets over a custom time window.
pub async fn compute_alt_az(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
    Json(request): Json<AltAzRequest>,
) -> HandlerResult<AltAzData> {
    let schedule_id = ScheduleId::new(schedule_id);

    let _schedule = db_services::get_schedule(state.repository.as_ref(), schedule_id).await?;

    let data =
        crate::services::compute_alt_az_data(schedule_id, &request).map_err(AppError::Internal)?;

    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/timeline
///
/// Get timeline visualization data for a schedule.
pub async fn get_timeline(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::ScheduleTimelineData> {
    let schedule_id = ScheduleId::new(schedule_id);
    let data = crate::services::timeline::get_schedule_timeline_data(
        state.repository.as_ref(),
        schedule_id,
    )
    .await
    .map_err(AppError::Internal)?;
    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/algorithm_trace
///
/// Get the algorithm trace for a schedule. Returns 404 if the schedule
/// has no trace (e.g. it was produced by an algorithm that does not emit
/// one).
pub async fn get_algorithm_trace(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::AlgorithmTraceResponse> {
    let schedule_id = ScheduleId::new(schedule_id);
    let trace = db_services::get_algorithm_trace(state.repository.as_ref(), schedule_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "No algorithm trace stored for schedule {schedule_id}"
            ))
        })?;
    Ok(Json(trace))
}

/// GET /v1/schedules/{schedule_id}/insights
///
/// Get insights analysis data for a schedule.
pub async fn get_insights(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::InsightsData> {
    let schedule_id = ScheduleId::new(schedule_id);
    let data = crate::services::insights::get_insights_data(state.repository.as_ref(), schedule_id)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/fragmentation
///
/// Get fragmentation analysis data for a schedule.
pub async fn get_fragmentation(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::FragmentationData> {
    let schedule_id = ScheduleId::new(schedule_id);
    let data = crate::services::fragmentation::get_fragmentation_data(
        state.repository.as_ref(),
        schedule_id,
    )
    .await
    .map_err(AppError::Internal)?;
    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/kpis
///
/// Compact KPI summary for a single schedule (A1). Backed by the
/// `schedule_kpis` service, which composes the existing insights and
/// fragmentation analytics into the small shape consumed by the
/// Workspace verdict / delta / evolution UIs.
pub async fn get_schedule_kpis(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::services::schedule_kpis::ScheduleKpi> {
    let sid = ScheduleId::new(schedule_id);
    let kpi = crate::services::schedule_kpis::compute_kpi_summary(state.repository.as_ref(), sid)
        .await
        .map_err(AppError::Internal)?;
    Ok(Json(kpi))
}

/// GET /v1/environments/{environment_id}/kpis
///
/// Batched KPI summaries for every schedule assigned to the given
/// environment. Per-schedule failures are collected in `errors` so a
/// single broken schedule does not blank out the whole comparison page.
pub async fn get_environment_kpis(
    State(state): State<AppState>,
    Path(environment_id): Path<i64>,
) -> HandlerResult<crate::services::schedule_kpis::EnvironmentKpisResponse> {
    use crate::services::schedule_kpis::{
        compute_kpi_summary, EnvironmentKpiError, EnvironmentKpisResponse,
    };

    let env = state
        .repository
        .get_environment(environment_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Environment {} not found", environment_id)))?;

    let mut kpis = Vec::with_capacity(env.schedule_ids.len());
    let mut errors = Vec::new();
    for sid in env.schedule_ids.clone() {
        match compute_kpi_summary(state.repository.as_ref(), sid).await {
            Ok(kpi) => kpis.push(kpi),
            Err(reason) => errors.push(EnvironmentKpiError {
                schedule_id: sid.value(),
                reason,
            }),
        }
    }

    Ok(Json(EnvironmentKpisResponse {
        environment_id,
        kpis,
        errors,
    }))
}

/// GET /v1/schedules/{schedule_id}/trends
///
/// Get trends analysis data for a schedule.
pub async fn get_trends(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
    Query(query): Query<TrendsQuery>,
) -> HandlerResult<crate::api::TrendsData> {
    let schedule_id = ScheduleId::new(schedule_id);
    let n_bins = query.bins.unwrap_or(10);
    let bandwidth = query.bandwidth.unwrap_or(0.5);
    let n_smooth_points = query.points.unwrap_or(12);

    let data = crate::services::trends::get_trends_data(
        state.repository.as_ref(),
        schedule_id,
        n_bins,
        bandwidth,
        n_smooth_points,
    )
    .await
    .map_err(AppError::Internal)?;
    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/validation-report
///
/// Get validation report for a schedule.
pub async fn get_validation_report(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<crate::api::ValidationReport> {
    let schedule_id = ScheduleId::new(schedule_id);
    let data =
        crate::services::validation::get_validation_report(state.repository.as_ref(), schedule_id)
            .await
            .map_err(AppError::Internal)?;
    Ok(Json(data))
}

/// GET /v1/schedules/{schedule_id}/compare/{other_id}
///
/// Compare two schedules.
pub async fn compare_schedules(
    State(state): State<AppState>,
    Path((schedule_id, other_id)): Path<(i64, i64)>,
    Query(query): Query<CompareQuery>,
) -> HandlerResult<crate::api::CompareData> {
    let current_id = ScheduleId::new(schedule_id);
    let comparison_id = ScheduleId::new(other_id);

    let current_name = match query.current_name {
        Some(name) if !name.trim().is_empty() => name,
        _ => db_services::get_schedule(state.repository.as_ref(), current_id)
            .await
            .ok()
            .map(|s| s.name)
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| format!("Schedule #{}", schedule_id)),
    };

    let comparison_name = match query.comparison_name {
        Some(name) if !name.trim().is_empty() => name,
        _ => db_services::get_schedule(state.repository.as_ref(), comparison_id)
            .await
            .ok()
            .map(|s| s.name)
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| format!("Schedule #{}", other_id)),
    };

    let data = crate::services::compare::get_compare_data(
        state.repository.as_ref(),
        current_id,
        comparison_id,
        current_name,
        comparison_name,
        query.epsilon_minutes,
        query.min_block_size,
        query.merge_epsilon_minutes,
    )
    .await
    .map_err(AppError::Internal)?;

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

// ===========================
// Environment Handlers
// ===========================

/// List all environments.
pub async fn list_environments(
    State(state): State<AppState>,
) -> HandlerResult<super::dto::EnvironmentListResponse> {
    let environments = state
        .repository
        .list_environments()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let total = environments.len();
    let environments = environments.into_iter().map(|e| e.into()).collect();

    Ok(Json(super::dto::EnvironmentListResponse {
        environments,
        total,
    }))
}

/// Get environment by ID.
pub async fn get_environment(
    State(state): State<AppState>,
    Path(environment_id): Path<i64>,
) -> HandlerResult<super::dto::EnvironmentResponse> {
    let environment = state
        .repository
        .get_environment(environment_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    match environment {
        Some(env) => Ok(Json(env.into())),
        None => Err(AppError::NotFound(format!(
            "Environment {} not found",
            environment_id
        ))),
    }
}

/// Create a new environment.
pub async fn create_environment(
    State(state): State<AppState>,
    Json(req): Json<super::dto::CreateEnvironmentRequest>,
) -> HandlerResult<super::dto::EnvironmentResponse> {
    let environment = state
        .repository
        .create_environment(&req.name)
        .await
        .map_err(AppError::from)?;

    Ok(Json(environment.into()))
}

/// Delete an environment.
pub async fn delete_environment(
    State(state): State<AppState>,
    Path(environment_id): Path<i64>,
) -> HandlerResult<DeleteScheduleResponse> {
    state
        .repository
        .delete_environment(environment_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(DeleteScheduleResponse {
        message: format!("Environment {} deleted successfully", environment_id),
    }))
}

/// POST /v1/environments/{environment_id}/schedules
///
/// Bulk-import a batch of schedules into a single environment. Items are
/// processed sequentially so that the first item can deterministically
/// initialise the environment's structure and preschedule cache, and
/// subsequent items reuse that cache without redundant computation.
///
/// Each item is independent: parse / structure / store failures push to
/// `rejected` and the loop continues. The handler only returns 404 when
/// the environment itself does not exist.
pub async fn bulk_import_schedules(
    State(state): State<AppState>,
    Path(environment_id): Path<i64>,
    Json(req): Json<super::dto::EnvironmentBulkImportRequest>,
) -> HandlerResult<super::dto::EnvironmentBulkImportResponse> {
    use super::dto::{
        EnvironmentBulkImportCreated, EnvironmentBulkImportItem, EnvironmentBulkImportRejected,
        EnvironmentBulkImportResponse,
    };
    use crate::api::EnvironmentStructure;
    use crate::models::schedule::compute_schedule_checksum;
    use crate::services::environment_preschedule::{
        apply_to_schedule, compute_env_preschedule, EnvPreschedulePayload,
    };
    use crate::services::environment_structure::{matches, structure_from_schedule};
    use futures::stream::{self, StreamExt};
    use std::time::Instant;

    /// Outcome for a single bulk-import item; an item can produce up to one
    /// `created` entry and any number of `rejected` entries (the optional
    /// algorithm-trace step contributes a synthetic `.trace` rejection on
    /// failure).
    struct ItemOutcome {
        created: Option<EnvironmentBulkImportCreated>,
        rejected: Vec<EnvironmentBulkImportRejected>,
    }

    impl ItemOutcome {
        fn empty() -> Self {
            Self {
                created: None,
                rejected: Vec::new(),
            }
        }

        fn rejected_one(name: String, reason: String, mismatch_fields: Vec<String>) -> Self {
            Self {
                created: None,
                rejected: vec![EnvironmentBulkImportRejected {
                    name,
                    reason,
                    mismatch_fields,
                }],
            }
        }
    }

    // Verify the environment exists up-front so we can return 404 early.
    let request_started_at = Instant::now();
    let env = state
        .repository
        .get_environment(environment_id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Environment {} not found", environment_id)))?;

    let mut created: Vec<EnvironmentBulkImportCreated> = Vec::new();
    let mut rejected: Vec<EnvironmentBulkImportRejected> = Vec::new();

    let items = req.items;
    let total_items = items.len();
    if items.is_empty() {
        return Ok(Json(EnvironmentBulkImportResponse { created, rejected }));
    }
    tracing::info!(
        environment_id = environment_id,
        items = total_items,
        concurrency = state.bulk_import_concurrency,
        "bulk_import_schedules: start"
    );

    // Phase A hoist: structure + preschedule are constant within a request,
    // so we resolve them once and share a single Arc with all worker tasks.
    let mut env_structure = env.structure.clone();
    let mut cached_preschedule: Option<EnvPreschedulePayload> = None;

    // Bootstrap: if the environment is empty, we must initialise its
    // structure from the first parseable item *before* running anything in
    // parallel — concurrent workers would otherwise race on
    // `initialise_environment` and `compute_env_preschedule`.
    let mut bootstrap_outcomes: Vec<(usize, ItemOutcome)> = Vec::new();
    let mut remaining: Vec<(usize, EnvironmentBulkImportItem)> =
        items.into_iter().enumerate().collect();

    if env_structure.is_none() {
        // Walk items in order until one parses successfully; report
        // failures inline so item ordering in the response is preserved.
        let mut consumed = 0usize;
        while consumed < remaining.len() {
            let (idx, item) = &remaining[consumed];
            let item_name = item.name.trim().to_string();
            let mut schedule_json = item.schedule_json.clone();
            if let Some(ref loc) = item.location_override {
                if let Ok(loc_value) = serde_json::to_value(loc) {
                    if let Some(obj) = schedule_json.as_object_mut() {
                        obj.insert("geographic_location".to_string(), loc_value);
                    }
                }
            }
            match state.import_adapter.parse_schedule_value(&schedule_json) {
                Ok(mut schedule) => {
                    if !item_name.is_empty() {
                        schedule.name = item_name.clone();
                    }
                    let new_structure = structure_from_schedule(&schedule);
                    let preschedule_payload = compute_env_preschedule(&schedule);
                    let payload_json = match serde_json::to_value(&preschedule_payload) {
                        Ok(v) => v,
                        Err(e) => {
                            bootstrap_outcomes.push((
                                *idx,
                                ItemOutcome::rejected_one(
                                    item_name,
                                    format!("Failed to serialise preschedule: {}", e),
                                    vec![],
                                ),
                            ));
                            consumed += 1;
                            continue;
                        }
                    };
                    if let Err(e) = state
                        .repository
                        .initialise_environment(environment_id, &new_structure, &payload_json)
                        .await
                    {
                        bootstrap_outcomes.push((
                            *idx,
                            ItemOutcome::rejected_one(
                                item_name,
                                format!("Failed to initialise environment: {}", e),
                                vec![],
                            ),
                        ));
                        consumed += 1;
                        continue;
                    }
                    env_structure = Some(new_structure);
                    cached_preschedule = Some(preschedule_payload);
                    break;
                }
                Err(e) => {
                    bootstrap_outcomes.push((
                        *idx,
                        ItemOutcome::rejected_one(
                            item_name,
                            format!("Failed to parse schedule: {}", e),
                            vec![],
                        ),
                    ));
                    consumed += 1;
                }
            }
        }
        // Drop the items we already accounted for during bootstrap.
        remaining.drain(0..consumed);
    }

    // If structure is *still* missing (every bootstrap item failed to
    // parse), there's nothing left to do; surface what we have.
    let structure = match env_structure {
        Some(s) => Arc::new(s),
        None => {
            for (_, out) in bootstrap_outcomes {
                created.extend(out.created);
                rejected.extend(out.rejected);
            }
            return Ok(Json(EnvironmentBulkImportResponse { created, rejected }));
        }
    };

    // Load the preschedule if we didn't just compute it locally during
    // bootstrap. This single fetch replaces what used to be one per item.
    if cached_preschedule.is_none() {
        let cached = match state.repository.get_preschedule(environment_id).await {
            Ok(Some(v)) => v,
            Ok(None) => {
                rejected.push(EnvironmentBulkImportRejected {
                    name: String::new(),
                    reason: format!("Environment {} has no cached preschedule", environment_id),
                    mismatch_fields: vec![],
                });
                for (_, out) in bootstrap_outcomes {
                    created.extend(out.created);
                    rejected.extend(out.rejected);
                }
                return Ok(Json(EnvironmentBulkImportResponse { created, rejected }));
            }
            Err(e) => return Err(AppError::Internal(e.to_string())),
        };
        cached_preschedule = Some(
            serde_json::from_value::<EnvPreschedulePayload>(cached)
                .map_err(|e| AppError::Internal(format!("Cached preschedule invalid: {e}")))?,
        );
    }
    let preschedule = Arc::new(cached_preschedule.unwrap());

    /// Process one item end-to-end: parse, apply preschedule, store
    /// schedule + analytics, assign to environment, optionally store the
    /// algorithm trace. Pure function of the inputs (no shared mutable
    /// state) so it is safe to run concurrently with peers.
    async fn process_item(
        state: AppState,
        environment_id: i64,
        structure: Arc<EnvironmentStructure>,
        preschedule: Arc<EnvPreschedulePayload>,
        item: EnvironmentBulkImportItem,
    ) -> ItemOutcome {
        let item_name = item.name.trim().to_string();
        let algorithm_trace_jsonl = item.algorithm_trace_jsonl;

        // Step 1: optional location override (mutate the Value in place).
        let mut schedule_json = item.schedule_json;
        if let Some(ref loc) = item.location_override {
            match serde_json::to_value(loc) {
                Ok(loc_value) => {
                    if let Some(obj) = schedule_json.as_object_mut() {
                        obj.insert("geographic_location".to_string(), loc_value);
                    }
                }
                Err(e) => {
                    return ItemOutcome::rejected_one(
                        item_name,
                        format!("Invalid location override: {}", e),
                        vec![],
                    );
                }
            }
        }

        // Step 2: parse via the adapter directly from the Value. Bulk-import
        // uses the structural fast path: the block visibility, dark periods,
        // and astronomical nights are about to be overwritten from the env
        // preschedule, so adapters can skip the per-item astronomy work.
        let mut schedule = match state
            .import_adapter
            .parse_schedule_value_structural(&schedule_json)
        {
            Ok(s) => s,
            Err(e) => {
                return ItemOutcome::rejected_one(
                    item_name,
                    format!("Failed to parse schedule: {}", e),
                    vec![],
                );
            }
        };

        // Step 3: apply user-provided name and recompute checksum from the
        // canonical (post-override) payload.
        if !item_name.is_empty() {
            schedule.name = item_name.clone();
        }
        let canonical_payload = match serde_json::to_string(&schedule_json) {
            Ok(s) => s,
            Err(e) => {
                return ItemOutcome::rejected_one(
                    item_name,
                    format!("Failed to serialise canonical payload: {}", e),
                    vec![],
                );
            }
        };
        schedule.checksum =
            compute_schedule_checksum(&format!("{}:{}", schedule.name, canonical_payload));

        // Step 4: validate against the shared structure.
        if let Err(mismatch) = matches(&structure, &schedule) {
            return ItemOutcome::rejected_one(
                item_name,
                mismatch.to_string(),
                mismatch.fields.clone(),
            );
        }

        // Step 5/6: apply the cached preschedule.
        apply_to_schedule(&mut schedule, &preschedule);
        schedule.astronomical_nights = preschedule.astronomical_nights.clone();
        // Use pre-computed dark periods (Moon below horizon ∩ Sun < -18°).
        // Fall back to computing on the fly if the cached preschedule predates
        // the dark_periods field (backwards compatibility via #[serde(default)]).
        schedule.dark_periods = if !preschedule.dark_periods.is_empty() {
            preschedule.dark_periods.clone()
        } else {
            crate::services::astronomical_night::compute_dark_periods(
                &schedule.geographic_location,
                &schedule.schedule_period,
                &preschedule.astronomical_nights,
            )
        };

        // Step 7: store the schedule. Analytics are deferred: the
        // `/insights` and friends already call `ensure_analytics` on read,
        // so populating per-item synchronously here just inflates upload
        // latency for users who may never view every uploaded schedule.
        let stored = match db_services::store_schedule_with_options(
            state.repository.as_ref(),
            &schedule,
            false,
        )
        .await
        {
            Ok(meta) => meta,
            Err(e) => {
                return ItemOutcome::rejected_one(
                    item_name,
                    format!("Failed to store schedule: {}", e),
                    vec![],
                );
            }
        };

        // Step 8: assign the stored schedule to the environment.
        if let Err(e) = state
            .repository
            .assign_schedule(stored.schedule_id, environment_id)
            .await
        {
            return ItemOutcome::rejected_one(
                item_name,
                format!("Failed to assign schedule to environment: {}", e),
                vec![],
            );
        }

        let mut outcome = ItemOutcome::empty();
        outcome.created = Some(EnvironmentBulkImportCreated {
            schedule_id: stored.schedule_id.value(),
            name: stored.schedule_name,
        });

        // Step 9: persist optional algorithm trace.
        if let Some(trace_text) = algorithm_trace_jsonl {
            match crate::services::schedule_processor::parse_algorithm_trace_jsonl(&trace_text) {
                Ok((algorithm, summary, iterations)) => {
                    if let Some(validator) = state.extensions.trace_validator_for(&algorithm) {
                        if let Err(e) = validator.validate_summary(&summary) {
                            outcome.rejected.push(EnvironmentBulkImportRejected {
                                name: format!("{}.trace", item_name),
                                reason: format!(
                                    "Algorithm trace validation failed for `{algorithm}`: {e}"
                                ),
                                mismatch_fields: vec![],
                            });
                            return outcome;
                        }
                    }
                    if let Err(e) = state
                        .repository
                        .store_algorithm_trace(
                            stored.schedule_id,
                            &algorithm,
                            &summary,
                            &iterations,
                        )
                        .await
                    {
                        outcome.rejected.push(EnvironmentBulkImportRejected {
                            name: format!("{}.trace", item_name),
                            reason: format!("Failed to store algorithm trace: {e}"),
                            mismatch_fields: vec![],
                        });
                    }
                }
                Err(e) => {
                    outcome.rejected.push(EnvironmentBulkImportRejected {
                        name: format!("{}.trace", item_name),
                        reason: format!("Failed to parse algorithm trace JSONL: {e}"),
                        mismatch_fields: vec![],
                    });
                }
            }
        }

        outcome
    }

    // Parallel fan-out. `buffered` preserves the original item order in the
    // response while keeping at most `concurrency` items in flight against
    // the repository at once. `concurrency` is bounded by both the
    // configured limit and the number of items so we don't allocate
    // unused slots.
    let concurrency = state.bulk_import_concurrency.max(1).min(remaining.len());
    let parallel_outcomes: Vec<(usize, ItemOutcome)> = stream::iter(remaining.into_iter())
        .map(|(idx, item)| {
            let state = state.clone();
            let structure = Arc::clone(&structure);
            let preschedule = Arc::clone(&preschedule);
            async move {
                (
                    idx,
                    process_item(state, environment_id, structure, preschedule, item).await,
                )
            }
        })
        .buffered(concurrency)
        .collect()
        .await;

    // Merge bootstrap + parallel outcomes back into request order so the
    // response mirrors the input layout.
    let mut all_outcomes = bootstrap_outcomes;
    all_outcomes.extend(parallel_outcomes);
    all_outcomes.sort_by_key(|(i, _)| *i);
    for (_, out) in all_outcomes {
        created.extend(out.created);
        rejected.extend(out.rejected);
    }

    // R3 correctness check (H3 — dedup masking): the postgres
    // `store_schedule` repository call deduplicates by checksum, so two
    // items in the same request that collapse to the same row would each
    // appear in `created` with the same `schedule_id`. Detect that here
    // and emit a structured warning so silent merges become observable
    // without changing the response shape (clients already see the same
    // id twice).
    {
        let mut seen: std::collections::HashSet<i64> = std::collections::HashSet::new();
        for c in &created {
            if !seen.insert(c.schedule_id) {
                tracing::warn!(
                    environment_id = environment_id,
                    duplicate_schedule_id = c.schedule_id,
                    name = %c.name,
                    "bulk_import_schedules: duplicate schedule_id in created list \
                     (likely checksum dedup masking)"
                );
            }
        }
    }

    let elapsed_ms = request_started_at.elapsed().as_millis() as u64;
    tracing::info!(
        environment_id = environment_id,
        items = total_items,
        created = created.len(),
        rejected = rejected.len(),
        elapsed_ms = elapsed_ms,
        concurrency = state.bulk_import_concurrency,
        "bulk_import_schedules: done"
    );
    state
        .bulk_import_latencies
        .push(super::state::BulkImportSample {
            duration_ms: elapsed_ms,
            items: total_items,
            created: created.len(),
            rejected: rejected.len(),
            concurrency: state.bulk_import_concurrency,
            environment_id,
            recorded_at_unix_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        });

    Ok(Json(EnvironmentBulkImportResponse { created, rejected }))
}

/// DELETE /v1/schedules/{schedule_id}/environment
///
/// Detach a schedule from its environment. The underlying repository
/// call is a no-op when the schedule is unassigned (or when it does not
/// exist), so this endpoint always returns 200 with a descriptive message.
pub async fn unassign_schedule_environment(
    State(state): State<AppState>,
    Path(schedule_id): Path<i64>,
) -> HandlerResult<DeleteScheduleResponse> {
    let sid = ScheduleId::new(schedule_id);
    state
        .repository
        .unassign_schedule(sid)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(DeleteScheduleResponse {
        message: format!("Schedule {} unassigned from its environment", schedule_id),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Constraints, ModifiedJulianDate, Period, SchedulingBlockId};

    fn make_block(id: i64, priority: f64, scheduled: bool) -> SchedulingBlock {
        SchedulingBlock {
            id: Some(SchedulingBlockId(id)),
            original_block_id: format!("block-{id}"),
            block_name: String::new(),
            target_ra: 10.0.into(),
            target_dec: 20.0.into(),
            constraints: Constraints {
                min_alt: 30.0.into(),
                max_alt: 90.0.into(),
                min_az: 0.0.into(),
                max_az: 360.0.into(),
                fixed_time: None,
            },
            priority,
            min_observation: 300.0.into(),
            requested_duration: 3600.0.into(),
            visibility_periods: vec![Period {
                start: ModifiedJulianDate::new(60000.0),
                end: ModifiedJulianDate::new(60000.5),
            }],
            scheduled_period: scheduled.then(|| Period {
                start: ModifiedJulianDate::new(60000.1),
                end: ModifiedJulianDate::new(60000.2),
            }),
        }
    }

    #[test]
    fn visibility_histogram_query_filters_scheduled_blocks() {
        let scheduled_block = make_block(1, 8.0, true);
        let unscheduled_block = make_block(2, 8.0, false);

        let scheduled_query = VisibilityHistogramQuery {
            scheduled: Some(true),
            ..Default::default()
        };
        let unscheduled_query = VisibilityHistogramQuery {
            scheduled: Some(false),
            ..Default::default()
        };

        assert!(block_matches_visibility_histogram_query(
            &scheduled_block,
            &scheduled_query
        ));
        assert!(!block_matches_visibility_histogram_query(
            &unscheduled_block,
            &scheduled_query
        ));
        assert!(block_matches_visibility_histogram_query(
            &unscheduled_block,
            &unscheduled_query
        ));
        assert!(!block_matches_visibility_histogram_query(
            &scheduled_block,
            &unscheduled_query
        ));
    }

    #[test]
    fn visibility_histogram_query_combines_scheduled_with_other_filters() {
        let matching_block = make_block(7, 6.0, true);
        let wrong_status = make_block(7, 6.0, false);
        let wrong_priority = make_block(7, 3.0, true);
        let wrong_id = make_block(9, 6.0, true);

        let query = VisibilityHistogramQuery {
            priority_min: Some(5.0),
            priority_max: Some(7.0),
            block_ids: Some(vec![7]),
            scheduled: Some(true),
            ..Default::default()
        };

        assert!(block_matches_visibility_histogram_query(
            &matching_block,
            &query
        ));
        assert!(!block_matches_visibility_histogram_query(
            &wrong_status,
            &query
        ));
        assert!(!block_matches_visibility_histogram_query(
            &wrong_priority,
            &query
        ));
        assert!(!block_matches_visibility_histogram_query(&wrong_id, &query));
    }
}
