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

/// Request body for creating a new schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduleRequest {
    /// Name for the schedule
    pub name: String,
    /// Schedule JSON data (raw JSON string or parsed object)
    pub schedule_json: serde_json::Value,
    /// Optional visibility JSON data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility_json: Option<serde_json::Value>,
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
    /// ID of the created schedule
    pub schedule_id: i64,
    /// Name of the schedule
    pub schedule_name: String,
    /// Message about the operation
    pub message: String,
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
