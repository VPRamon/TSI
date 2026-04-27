//! Data Transfer Objects for the HTTP API.
//!
//! These DTOs are used for request/response serialization in the REST API.
//! Most visualization DTOs are re-exported from the routes module since they
//! already derive Serialize/Deserialize.

use serde::{Deserialize, Deserializer, Serialize};

// Re-export existing DTOs that are already serializable
pub use crate::api::{
    // Compare
    AdvancedCompare,
    AdvancedCompareParams,
    AdvancedGlobalMetrics,
    // Insights
    AnalyticsMetrics,
    CoherentBlock,
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
    /// Optional algorithm trace as raw JSONL text. When provided, the
    /// trace is parsed (one event per line) and persisted alongside the
    /// schedule so algorithm-specific frontend extensions can replay the
    /// search.
    #[serde(default)]
    pub algorithm_trace_jsonl: Option<String>,
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
    /// Shift-continuity tolerance in minutes for the advanced segmentation.
    /// Default: 5.0
    #[serde(default)]
    pub epsilon_minutes: Option<f64>,
    /// Minimum number of tasks for a coherent block to survive. Default: 3
    #[serde(default)]
    pub min_block_size: Option<usize>,
    /// Merge tolerance in minutes for adjacent coherent blocks.
    /// Default: same as `epsilon_minutes`.
    #[serde(default)]
    pub merge_epsilon_minutes: Option<f64>,
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

/// One sample exposed by the bulk-import latency ring buffer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportSampleDto {
    pub duration_ms: u64,
    pub items: usize,
    pub created: usize,
    pub rejected: usize,
    pub concurrency: usize,
    pub environment_id: i64,
    pub recorded_at_unix_ms: u64,
}

/// Diagnostics response served by `/v1/_health/db`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbDiagnosticsResponse {
    /// Currently effective `BULK_IMPORT_CONCURRENCY` knob.
    pub bulk_import_concurrency: usize,
    /// Most recent bulk-import requests (oldest first).
    pub recent_bulk_imports: Vec<BulkImportSampleDto>,
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
    /// Geographic location of the observatory.
    pub observer_location: crate::api::GeographicLocation,
    /// Overall time window of the schedule (MJD).
    pub schedule_period: SchedulePeriodDto,
    /// Optional algorithm provenance populated when the schedule was produced
    /// with a structured algorithm trace.  Absent for schedules uploaded
    /// without a trace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_metadata: Option<ScheduleMetadataDto>,
}

/// Minimal algorithm provenance attached to a schedule in list responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMetadataDto {
    pub algorithm: String,
}

/// Schedule period in MJD, as returned by the list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulePeriodDto {
    pub start_mjd: f64,
    pub end_mjd: f64,
}

impl From<crate::api::ScheduleInfo> for ScheduleInfoDto {
    fn from(info: crate::api::ScheduleInfo) -> Self {
        Self {
            schedule_id: info.schedule_id.value(),
            schedule_name: info.schedule_name,
            observer_location: info.observer_location,
            schedule_period: SchedulePeriodDto {
                start_mjd: info.schedule_period.start.value(),
                end_mjd: info.schedule_period.end.value(),
            },
            schedule_metadata: None,
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

// ===========================
// Environment Endpoints DTOs
// ===========================

/// Request to create a new environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEnvironmentRequest {
    /// Name of the environment (must be unique)
    pub name: String,
}

/// Response containing environment information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentResponse {
    pub environment_id: i64,
    pub name: String,
    pub structure: Option<crate::api::EnvironmentStructure>,
    /// IDs of schedules currently assigned to this environment.
    pub schedule_ids: Vec<i64>,
    pub created_at: String,
}

/// List of environments response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentListResponse {
    pub environments: Vec<EnvironmentResponse>,
    pub total: usize,
}

impl From<crate::api::EnvironmentInfo> for EnvironmentResponse {
    fn from(info: crate::api::EnvironmentInfo) -> Self {
        Self {
            environment_id: info.environment_id,
            name: info.name,
            structure: info.structure,
            schedule_ids: info.schedule_ids.into_iter().map(|id| id.value()).collect(),
            created_at: info.created_at.to_rfc3339(),
        }
    }
}

/// Bulk-import request: a batch of schedules to load into one environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentBulkImportRequest {
    pub items: Vec<EnvironmentBulkImportItem>,
}

/// A single schedule payload inside a bulk-import request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentBulkImportItem {
    pub name: String,
    pub schedule_json: serde_json::Value,
    #[serde(default)]
    pub location_override: Option<crate::api::GeographicLocation>,
    #[serde(default)]
    pub algorithm_trace_jsonl: Option<String>,
}

/// Bulk-import response, partitioning the batch into accepted and rejected items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentBulkImportResponse {
    pub created: Vec<EnvironmentBulkImportCreated>,
    pub rejected: Vec<EnvironmentBulkImportRejected>,
}

/// Successfully imported schedule entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentBulkImportCreated {
    pub schedule_id: i64,
    pub name: String,
}

/// Rejected schedule entry, with a human-readable reason and the
/// list of structure fields that mismatched (empty for non-structure errors).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentBulkImportRejected {
    pub name: String,
    pub reason: String,
    pub mismatch_fields: Vec<String>,
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

    #[test]
    fn bulk_import_item_round_trips_optional_algorithm_trace() {
        let json = r#"{"name":"sched","schedule_json":{"a":1},"algorithm_trace_jsonl":"{\"kind\":\"started\",\"algorithm\":\"est\"}"}"#;
        let item: EnvironmentBulkImportItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.name, "sched");
        assert_eq!(
            item.algorithm_trace_jsonl.as_deref(),
            Some("{\"kind\":\"started\",\"algorithm\":\"est\"}")
        );

        let bare = r#"{"name":"sched","schedule_json":{"a":1}}"#;
        let item: EnvironmentBulkImportItem = serde_json::from_str(bare).unwrap();
        assert!(item.algorithm_trace_jsonl.is_none());
    }
}
