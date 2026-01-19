//! Data Transfer Objects for the HTTP API.
//!
//! These DTOs are used for request/response serialization in the REST API.
//! Most visualization DTOs are re-exported from the routes module since they
//! already derive Serialize/Deserialize.

use serde::{Deserialize, Serialize};

// Re-export existing DTOs that are already serializable
pub use crate::api::{
    // Compare
    CompareBlock, CompareData, CompareStats, SchedulingChange,
    // Distribution
    DistributionBlock, DistributionData, DistributionStats,
    // Insights
    AnalyticsMetrics, ConflictRecord, CorrelationEntry, InsightsBlock, InsightsData, TopObservation,
    // Landing
    ScheduleInfo,
    // Sky Map
    LightweightBlock, PriorityBinInfo, SkyMapData,
    // Timeline
    ScheduleTimelineBlock, ScheduleTimelineData,
    // Trends
    EmpiricalRatePoint, HeatmapBin, SmoothedPoint, TrendsBlock, TrendsData, TrendsMetrics,
    // Validation
    ValidationIssue, ValidationReport,
    // Visibility
    VisibilityBlockSummary, VisibilityMapData,
};
pub use crate::db::models::VisibilityBin;

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
    pub priority_min: Option<i32>,
    /// Maximum priority filter (inclusive, optional)
    #[serde(default)]
    pub priority_max: Option<i32>,
    /// Filter by specific block IDs (optional)
    #[serde(default)]
    pub block_ids: Option<Vec<i64>>,
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
