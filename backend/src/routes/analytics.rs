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
use crate::models::api::ErrorResponse;
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

/// Query parameters for trends endpoint
#[derive(Debug, Deserialize)]
pub struct TrendsQuery {
    /// Metric to analyze: scheduling_rate, utilization, priority_distribution
    #[serde(default = "default_metric")]
    pub metric: String,
    
    /// Group by: month, week, day
    #[serde(default = "default_group_by")]
    pub group_by: String,
}

fn default_metric() -> String {
    "scheduling_rate".to_string()
}

fn default_group_by() -> String {
    "month".to_string()
}

/// Single time point in trends data
#[derive(Debug, Serialize)]
pub struct TrendPoint {
    pub period: String,
    pub value: f64,
    pub count: usize,
}

/// Response for trends endpoint
#[derive(Debug, Serialize)]
pub struct TrendsResponse {
    pub metric: String,
    pub group_by: String,
    pub data: Vec<TrendPoint>,
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
                    details: None,
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
                    details: None,
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
                    details: None,
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
                    details: None,
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
                    details: None,
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Invalid column: {}", column),
                    details: None,
                }),
            )
        })?;
    
    Ok(Json(result))
}

/// GET /api/v1/analytics/trends - Get time series trends
pub async fn get_trends(
    State(state): State<AppState>,
    Query(params): Query<TrendsQuery>,
) -> Result<Json<TrendsResponse>, (StatusCode, Json<ErrorResponse>)> {
    use std::collections::HashMap;
    
    let data = state
        .with_dataset(|blocks| {
            // Group observations by time period
            let mut period_groups: HashMap<String, Vec<&crate::models::schedule::SchedulingBlock>> = HashMap::new();
            
            for block in blocks.iter() {
                if let Some(scheduled_time) = block.scheduled_period_start {
                    // Convert MJD to period string
                    let period = match params.group_by.as_str() {
                        "month" => {
                            // Approximate month from MJD
                            let days_since_mjd0 = scheduled_time;
                            let years_since_1858 = days_since_mjd0 / 365.25;
                            let year = 1858 + years_since_1858 as i32;
                            let year_fraction = years_since_1858.fract();
                            let month = (year_fraction * 12.0) as u8 + 1;
                            format!("{}-{:02}", year, month)
                        }
                        "week" => {
                            // Week number from MJD
                            let week = (scheduled_time / 7.0) as i32;
                            format!("Week {}", week)
                        }
                        _ => {
                            // Day
                            format!("Day {}", scheduled_time as i32)
                        }
                    };
                    
                    period_groups.entry(period).or_insert_with(Vec::new).push(block);
                }
            }
            
            // Compute metric for each period
            let mut results: Vec<TrendPoint> = period_groups
                .into_iter()
                .map(|(period, group_blocks)| {
                    let count = group_blocks.len();
                    let value = match params.metric.as_str() {
                        "scheduling_rate" => {
                            // Percentage of blocks that are scheduled
                            let scheduled = group_blocks.iter().filter(|b| b.scheduled_flag).count();
                            (scheduled as f64 / count as f64) * 100.0
                        }
                        "utilization" => {
                            // Scheduled hours / visibility hours
                            let scheduled_hours: f64 = group_blocks
                                .iter()
                                .filter_map(|b| {
                                    if let (Some(start), Some(stop)) = (b.scheduled_period_start, b.scheduled_period_stop) {
                                        Some((stop - start) * 24.0) // Convert MJD to hours
                                    } else {
                                        None
                                    }
                                })
                                .sum();
                            let visibility_hours: f64 = group_blocks
                                .iter()
                                .map(|b| b.total_visibility_hours)
                                .sum();
                            if visibility_hours > 0.0 {
                                (scheduled_hours / visibility_hours) * 100.0
                            } else {
                                0.0
                            }
                        }
                        "priority_distribution" | "avg_priority" => {
                            // Average priority
                            let sum: f64 = group_blocks.iter().map(|b| b.priority).sum();
                            sum / count as f64
                        }
                        _ => count as f64,
                    };
                    
                    TrendPoint {
                        period,
                        value,
                        count,
                    }
                })
                .collect();
            
            // Sort by period
            results.sort_by(|a, b| a.period.cmp(&b.period));
            
            results
        })
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("No dataset loaded: {}", e),
                    details: None,
                }),
            )
        })?;
    
    Ok(Json(TrendsResponse {
        metric: params.metric,
        group_by: params.group_by,
        data,
    }))
}
