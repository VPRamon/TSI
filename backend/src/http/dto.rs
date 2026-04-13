//! Data Transfer Objects for the HTTP API.
//!
//! These DTOs are used for request/response serialization in the REST API.
//! Most visualization DTOs are re-exported from the routes module since they
//! already derive Serialize/Deserialize.

use serde::{Deserialize, Deserializer, Serialize};

// Re-export existing DTOs that are already serializable
pub use crate::api::{
    // Insights
    AnalyticsMetrics,
    // Compare
    CompareBlock,
    CompareData,
    CompareStats,
    ConflictRecord,
    CorrelationEntry,
    // Distribution
    DistributionBlock,
    DistributionData,
    DistributionStats,
    // Trends
    EmpiricalRatePoint,
    HeatmapBin,
    InsightsBlock,
    InsightsData,
    // Sky Map
    LightweightBlock,
    PriorityBinInfo,
    // Landing
    ScheduleInfo,
    // Timeline
    ScheduleTimelineBlock,
    ScheduleTimelineData,
    SchedulingChange,
    SkyMapData,
    SmoothedPoint,
    TopObservation,
    TrendsBlock,
    TrendsData,
    TrendsMetrics,
    // Validation
    ValidationIssue,
    ValidationReport,
    // Visibility
    VisibilityBlockSummary,
    VisibilityMapData,
};
pub use crate::db::models::VisibilityBin;

/// Optional manual schedule period override, expressed as MJD values.
///
/// When present in [`CreateScheduleRequest`], the importer uses this window
/// instead of any period inferred from the payload (or the hard-coded fallback
/// used when no timing information is available in the file).
///
/// Validation requirements:
/// - `start_mjd` must be strictly less than `end_mjd`
/// - If any block carries a `scheduled_period`, it must fall within this window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulePeriodOverride {
    /// Schedule window start in Modified Julian Date.
    pub start_mjd: f64,
    /// Schedule window end in Modified Julian Date.
    pub end_mjd: f64,
}

/// Request body for creating a new schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduleRequest {
    /// Name for the schedule
    pub name: String,
    /// Schedule JSON data (may include optional possible_periods field)
    pub schedule_json: serde_json::Value,
    /// Whether to populate analytics after storing (default: true)
    #[serde(default = "default_true")]
    pub populate_analytics: bool,
    /// Optional geographic location override. When provided, overrides any
    /// `geographic_location` present in `schedule_json`. Intended for use
    /// when loading a schedule that does not embed site coordinates, allowing
    /// the caller to select a well-known observatory site.
    #[serde(default)]
    pub location_override: Option<crate::api::GeographicLocation>,
    /// Optional schedule period override. When provided, replaces the period
    /// inferred from the payload (or the fallback period used when no timing
    /// data is present in the file). Useful for importing unscheduled datasets
    /// where no blocks carry timing information.
    #[serde(default)]
    pub schedule_period_override: Option<SchedulePeriodOverride>,
}

fn default_true() -> bool {
    true
}

/// Response for schedule creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduleResponse {
    /// Job ID for tracking the async processing
    pub job_id: String,
    /// Message about the operation
    pub message: String,
}

/// Job status response for async processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusResponse {
    /// Job ID
    pub job_id: String,
    /// Job status
    pub status: String,
    /// Log entries
    pub logs: Vec<crate::services::job_tracker::LogEntry>,
    /// Result if completed
    pub result: Option<serde_json::Value>,
}

/// Query parameters for trends endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrendsQuery {
    /// Number of bins for empirical rate calculation
    #[serde(default)]
    pub bins: Option<usize>,
    /// Bandwidth for kernel smoothing
    #[serde(default)]
    pub bandwidth: Option<f64>,
    /// Number of points for smoothed curves
    #[serde(default)]
    pub points: Option<usize>,
}

/// Query parameters for compare endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompareQuery {
    /// Name for the current schedule (optional)
    #[serde(default)]
    pub current_name: Option<String>,
    /// Name for the comparison schedule (optional)
    #[serde(default)]
    pub comparison_name: Option<String>,
}

/// Query parameters for visibility histogram endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisibilityHistogramQuery {
    /// Bin duration in minutes (optional, default: computed from num_bins)
    #[serde(default)]
    pub bin_duration_minutes: Option<i64>,
    /// Number of bins (used if bin_duration_minutes not specified, default: 50)
    #[serde(default)]
    pub num_bins: Option<usize>,
    /// Minimum priority filter (inclusive, optional)
    #[serde(default)]
    pub priority_min: Option<f64>,
    /// Maximum priority filter (inclusive, optional)
    #[serde(default)]
    pub priority_max: Option<f64>,
    /// Filter by specific block IDs (optional, comma-separated in query string)
    #[serde(default, deserialize_with = "deserialize_comma_sep_i64")]
    pub block_ids: Option<Vec<i64>>,
    /// Filter by scheduled status (optional)
    #[serde(default)]
    pub scheduled: Option<bool>,
}

/// Deserializes a comma-separated string (e.g. `"1,2,3"`) into `Option<Vec<i64>>`.
///
/// `serde_urlencoded` cannot deserialize repeated query params into `Vec<T>`, so
/// we encode block IDs as a single `block_ids=1,2,3` parameter instead.
fn deserialize_comma_sep_i64<'de, D>(deserializer: D) -> Result<Option<Vec<i64>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.map(|s| {
        s.split(',')
            .filter_map(|part| part.trim().parse::<i64>().ok())
            .collect()
    }))
}

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Status of the service
    pub status: String,
    /// Version of the API
    pub version: String,
    /// Database connection status
    pub database: String,
}

/// Schedule list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleListResponse {
    /// List of schedules
    pub schedules: Vec<ScheduleInfoDto>,
    /// Total count
    pub total: usize,
}

/// Schedule info DTO for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleInfoDto {
    /// Schedule ID
    pub schedule_id: i64,
    /// Schedule name
    pub schedule_name: String,
}

impl From<crate::api::ScheduleInfo> for ScheduleInfoDto {
    fn from(info: crate::api::ScheduleInfo) -> Self {
        Self {
            schedule_id: info.schedule_id.value(),
            schedule_name: info.schedule_name,
        }
    }
}

/// Request body for updating schedule metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScheduleRequest {
    /// New name for the schedule (optional)
    #[serde(default)]
    pub name: Option<String>,
    /// New geographic location override (optional)
    #[serde(default)]
    pub location: Option<crate::api::GeographicLocation>,
}

/// Response for successful deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteScheduleResponse {
    /// Confirmation message
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visibility_histogram_query_parses_comma_separated_block_ids() {
        let q: VisibilityHistogramQuery = serde_urlencoded::from_str("block_ids=1,2,3").unwrap();
        assert_eq!(q.block_ids, Some(vec![1, 2, 3]));
    }

    #[test]
    fn visibility_histogram_query_parses_single_block_id() {
        let q: VisibilityHistogramQuery = serde_urlencoded::from_str("block_ids=42").unwrap();
        assert_eq!(q.block_ids, Some(vec![42]));
    }

    #[test]
    fn visibility_histogram_query_none_when_block_ids_absent() {
        let q: VisibilityHistogramQuery = serde_urlencoded::from_str("num_bins=50").unwrap();
        assert_eq!(q.block_ids, None);
    }

    #[test]
    fn visibility_histogram_query_ignores_invalid_block_ids() {
        let q: VisibilityHistogramQuery = serde_urlencoded::from_str("block_ids=1,bad,3").unwrap();
        // "bad" is silently skipped; only valid integers survive
        assert_eq!(q.block_ids, Some(vec![1, 3]));
    }
}
