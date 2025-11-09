/// API routes for analytics endpoints
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::analytics::{
    conflicts::{detect_conflicts, ConflictReport},
    correlations::{compute_correlations, CorrelationMatrix},
    distributions::{compute_distribution_stats, compute_histogram},
    metrics::{compute_metrics, SchedulingMetrics},
    top_observations::{get_top_observations, RankedObservation, SortBy, SortOrder},
};
use crate::state::AppState;

/// Query parameters for correlation endpoint
#[derive(Debug, Deserialize)]
pub struct CorrelationQuery {
    #[serde(default)]
    pub columns: Option<String>,
}

/// Query parameters for top observations endpoint
#[derive(Debug, Deserialize)]
pub struct TopObservationsQuery {
    #[serde(default = "default_sort_by")]
    pub by: String,
    
    #[serde(default = "default_order")]
    pub order: String,
    
    #[serde(default = "default_limit")]
    pub n: usize,
    
    pub scheduled: Option<bool>,
}

fn default_sort_by() -> String {
    "priority".to_string()
}

fn default_order() -> String {
    "descending".to_string()
}

fn default_limit() -> usize {
    10
}

/// Query parameters for distribution endpoint
#[derive(Debug, Deserialize)]
pub struct DistributionQuery {
    pub column: String,
    
    #[serde(default = "default_bins")]
    pub bins: usize,
    
    #[serde(default)]
    pub stats: bool,
}

fn default_bins() -> usize {
    20
}

/// Response wrapper for errors
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// GET /api/v1/analytics/metrics - Get overall scheduling metrics
pub async fn get_metrics(
    State(state): State<AppState>,
) -> Result<Json<SchedulingMetrics>, (StatusCode, Json<ErrorResponse>)> {
    let metrics = state
        .with_dataset(|blocks| compute_metrics(blocks))
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("No dataset loaded: {}", e),
                }),
            )
        })?;
    
    Ok(Json(metrics))
}

/// GET /api/v1/analytics/correlations - Compute correlation matrix
pub async fn get_correlations(
    State(state): State<AppState>,
    Query(params): Query<CorrelationQuery>,
) -> Result<Json<CorrelationMatrix>, (StatusCode, Json<ErrorResponse>)> {
    // Parse columns from comma-separated string
    let columns = if let Some(cols_str) = params.columns {
        cols_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        // Default columns
        vec![
            "priority".to_string(),
            "total_visibility_hours".to_string(),
            "requested_hours".to_string(),
            "elevation_range_deg".to_string(),
        ]
    };
    
    let correlations = state
        .with_dataset(|blocks| compute_correlations(blocks, &columns))
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("No dataset loaded: {}", e),
                }),
            )
        })?;
    
    Ok(Json(correlations))
}

/// GET /api/v1/analytics/conflicts - Detect conflicts in scheduling
pub async fn get_conflicts(
    State(state): State<AppState>,
) -> Result<Json<ConflictReport>, (StatusCode, Json<ErrorResponse>)> {
    let tolerance_sec = 1.0; // Small tolerance for floating point comparisons
    let report = state
        .with_dataset(|blocks| detect_conflicts(blocks, tolerance_sec))
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("No dataset loaded: {}", e),
                }),
            )
        })?;
    
    Ok(Json(report))
}

/// GET /api/v1/analytics/top - Get top N observations
pub async fn get_top(
    State(state): State<AppState>,
    Query(params): Query<TopObservationsQuery>,
) -> Result<Json<Vec<RankedObservation>>, (StatusCode, Json<ErrorResponse>)> {
    let sort_by = match params.by.as_str() {
        "priority" => SortBy::Priority,
        "requested_hours" | "requested" => SortBy::RequestedHours,
        "visibility_hours" | "visibility" => SortBy::VisibilityHours,
        "elevation_range" | "elevation" => SortBy::ElevationRange,
        _ => SortBy::Priority,
    };
    
    let order = match params.order.as_str() {
        "asc" | "ascending" => SortOrder::Ascending,
        "desc" | "descending" => SortOrder::Descending,
        _ => SortOrder::Descending,
    };
    
    let top = state
        .with_dataset(|blocks| get_top_observations(blocks, sort_by, order, params.n, params.scheduled))
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("No dataset loaded: {}", e),
                }),
            )
        })?;
    
    Ok(Json(top))
}

/// GET /api/v1/analytics/distribution - Get distribution statistics or histogram
pub async fn get_distribution(
    State(state): State<AppState>,
    Query(params): Query<DistributionQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let column = params.column.clone();
    
    let result = state
        .with_dataset(|blocks| {
            if params.stats {
                // Return distribution statistics
                compute_distribution_stats(blocks, &column)
                    .map(|stats| serde_json::to_value(stats).unwrap())
            } else {
                // Return histogram
                compute_histogram(blocks, &column, params.bins)
                    .map(|histogram| serde_json::to_value(histogram).unwrap())
            }
        })
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("No dataset loaded: {}", e),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid column: {}", column),
                }),
            )
        })?;
    
    Ok(Json(result))
}
