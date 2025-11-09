// Visualization endpoints for Phase 4
// These provide data for interactive visual components
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::models::schedule::VisibilityPeriod;
use crate::state::AppState;

/// Query parameters for visibility map endpoint
#[derive(Debug, Deserialize)]
pub struct VisibilityMapQuery {
    /// Scheduling block ID to get visibility data for
    pub block_id: String,
}

/// Response for visibility map endpoint
#[derive(Debug, Serialize)]
pub struct VisibilityMapResponse {
    pub scheduling_block_id: String,
    pub right_ascension_deg: f64,
    pub declination_deg: f64,
    pub requested_hours: f64,
    pub total_visibility_hours: f64,
    pub priority: f64,
    pub scheduled_flag: bool,
    pub visibility_periods: Vec<VisibilityPeriod>,
    /// Azimuth range constraints if available
    pub azimuth_min_deg: Option<f64>,
    pub azimuth_max_deg: Option<f64>,
    /// Elevation range constraints
    pub elevation_min_deg: Option<f64>,
    pub elevation_max_deg: Option<f64>,
    pub elevation_range_deg: Option<f64>,
}

/// Query parameters for timeline endpoint
#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    /// Optional month filter (1-12)
    pub month: Option<u8>,
    /// Optional year filter
    pub year: Option<i32>,
}

/// Single observation in timeline
#[derive(Debug, Serialize)]
pub struct TimelineObservation {
    pub scheduling_block_id: String,
    pub scheduled_time_mjd: f64,
    pub scheduled_time_iso: String,
    pub scheduled_duration_hours: f64,
    pub priority: f64,
    pub priority_bin: String,
    pub right_ascension_deg: f64,
    pub declination_deg: f64,
}

/// Response for timeline endpoint
#[derive(Debug, Serialize)]
pub struct TimelineResponse {
    pub observations: Vec<TimelineObservation>,
    pub total_count: usize,
    pub month: Option<u8>,
    pub year: Option<i32>,
}

/// Error response for visualization endpoints
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    error: String,
    details: Option<String>,
}

/// GET /api/v1/visualizations/visibility-map?block_id=...
///
/// Returns detailed visibility information for a specific scheduling block
pub async fn get_visibility_map(
    State(state): State<AppState>,
    Query(query): Query<VisibilityMapQuery>,
) -> Result<Json<VisibilityMapResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Find the block in the dataset
    let block = state
        .with_dataset(|blocks| {
            blocks
                .iter()
                .find(|b| b.scheduling_block_id == query.block_id)
                .cloned()
        })
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Dataset not loaded".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Block not found".to_string(),
                    details: Some(format!("No block with ID {}", query.block_id)),
                }),
            )
        })?;

    let response = VisibilityMapResponse {
        scheduling_block_id: block.scheduling_block_id,
        right_ascension_deg: block.ra_in_deg,
        declination_deg: block.dec_in_deg,
        requested_hours: block.requested_hours,
        total_visibility_hours: block.total_visibility_hours,
        priority: block.priority,
        scheduled_flag: block.scheduled_flag,
        visibility_periods: block.visibility.clone(),
        azimuth_min_deg: Some(block.min_azimuth_angle_in_deg),
        azimuth_max_deg: Some(block.max_azimuth_angle_in_deg),
        elevation_min_deg: Some(block.min_elevation_angle_in_deg),
        elevation_max_deg: Some(block.max_elevation_angle_in_deg),
        elevation_range_deg: Some(block.elevation_range_deg),
    };

    Ok(Json(response))
}

/// GET /api/v1/visualizations/timeline?month=...&year=...
///
/// Returns scheduled observations, optionally filtered by month/year
pub async fn get_timeline(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<TimelineResponse>, (StatusCode, Json<ErrorResponse>)> {
    let observations: Vec<TimelineObservation> = state
        .with_dataset(|blocks| {
            // Filter scheduled blocks
            let mut scheduled_blocks: Vec<_> = blocks
                .iter()
                .filter(|b| b.scheduled_flag && b.scheduled_period_start.is_some())
                .collect();

            // Sort by scheduled time
            scheduled_blocks.sort_by(|a, b| {
                a.scheduled_period_start
                    .unwrap_or(0.0)
                    .partial_cmp(&b.scheduled_period_start.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Convert to timeline observations
            scheduled_blocks
                .into_iter()
                .filter_map(|block| {
                    let scheduled_time = block.scheduled_period_start?;
                    
                    // Filter by month/year if specified
                    if let (Some(month), Some(year)) = (query.month, query.year) {
                        // Convert MJD to approximate month/year for filtering
                        // MJD 0 = November 17, 1858
                        // Simple approximation: 1 year ≈ 365.25 days
                        let days_since_mjd0 = scheduled_time;
                        let years_since_1858 = days_since_mjd0 / 365.25;
                        let approx_year = 1858.0 + years_since_1858;
                        let year_fraction = years_since_1858.fract();
                        let approx_month = (year_fraction * 12.0) as u8 + 1;
                        
                        if approx_year as i32 != year || approx_month != month {
                            return None;
                        }
                    }

                    // Calculate scheduled duration
                    let scheduled_duration_hours = if let Some(stop) = block.scheduled_period_stop {
                        (stop - scheduled_time) * 24.0 // Convert MJD days to hours
                    } else {
                        block.requested_hours
                    };

                    Some(TimelineObservation {
                        scheduling_block_id: block.scheduling_block_id.clone(),
                        scheduled_time_mjd: scheduled_time,
                        scheduled_time_iso: mjd_to_iso(scheduled_time),
                        scheduled_duration_hours,
                        priority: block.priority,
                        priority_bin: block.priority_bin.to_string(),
                        right_ascension_deg: block.ra_in_deg,
                        declination_deg: block.dec_in_deg,
                    })
                })
                .collect()
        })
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "Dataset not loaded".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    let total_count = observations.len();

    Ok(Json(TimelineResponse {
        observations,
        total_count,
        month: query.month,
        year: query.year,
    }))
}

/// Convert MJD to ISO 8601 date string (approximate)
fn mjd_to_iso(mjd: f64) -> String {
    // MJD 0 = November 17, 1858
    // Convert to Unix timestamp (days since Jan 1, 1970)
    // MJD of Jan 1, 1970 = 40587
    let unix_days = mjd - 40587.0;
    let unix_seconds = (unix_days * 86400.0) as i64;
    
    // Use chrono to format
    use chrono::DateTime;
    if let Some(datetime) = DateTime::from_timestamp(unix_seconds, 0) {
        datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string()
    } else {
        "Invalid date".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mjd_to_iso() {
        // Test a known date: MJD 51544.0 = Jan 1, 2000 00:00:00
        let iso = mjd_to_iso(51544.0);
        assert_eq!(iso, "2000-01-01T00:00:00Z");
    }

    #[test]
    fn test_mjd_to_iso_recent() {
        // Test a recent date: MJD 60000 ≈ Feb 13, 2023
        let iso = mjd_to_iso(60000.0);
        assert!(iso.starts_with("2023-"));
    }
}
