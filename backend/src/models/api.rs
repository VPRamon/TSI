/// API request and response types
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::schedule::{DatasetMetadata, SchedulingBlock};

// ============================================================================
// Dataset Management
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct DatasetResponse {
    pub metadata: DatasetMetadata,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DatasetListResponse {
    pub blocks: Vec<SchedulingBlock>,
    pub metadata: DatasetMetadata,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

// ============================================================================
// Analytics
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct MetricsResponse {
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
    pub unscheduled_blocks: usize,
    pub scheduling_rate: f64,
    pub total_requested_hours: f64,
    pub total_scheduled_hours: f64,
    pub utilization: f64,
    pub avg_priority: f64,
    pub avg_visibility_hours: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CorrelationRequest {
    #[serde(default)]
    pub columns: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CorrelationResponse {
    pub columns: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConflictBlock {
    pub scheduling_block_id: String,
    pub issue: String,
    pub priority: f64,
    pub requested_hours: f64,
    pub total_visibility_hours: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ConflictsResponse {
    pub conflicts: Vec<ConflictBlock>,
    pub count: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TopObservationsRequest {
    #[serde(default = "default_top_by")]
    pub by: String,
    #[serde(default = "default_top_n")]
    pub n: usize,
}

fn default_top_by() -> String {
    "priority".to_string()
}

fn default_top_n() -> usize {
    10
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TopObservationsResponse {
    pub observations: Vec<SchedulingBlock>,
    pub count: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DistributionRequest {
    pub column: String,
    #[serde(default = "default_bins")]
    pub bins: usize,
}

fn default_bins() -> usize {
    20
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DistributionBin {
    pub min: f64,
    pub max: f64,
    pub count: usize,
    pub frequency: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DistributionResponse {
    pub column: String,
    pub bins: Vec<DistributionBin>,
    pub stats: DistributionStats,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DistributionStats {
    pub mean: f64,
    pub median: f64,
    pub std: f64,
    pub min: f64,
    pub max: f64,
    pub q25: f64,
    pub q75: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InsightsResponse {
    pub insights: Vec<String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Visualizations
// ============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct SkyMapFilters {
    #[serde(default)]
    pub priority_min: Option<f64>,
    #[serde(default)]
    pub priority_max: Option<f64>,
    #[serde(default)]
    pub scheduled_only: Option<bool>,
    #[serde(default)]
    pub unscheduled_only: Option<bool>,
    #[serde(default)]
    pub time_min: Option<f64>,
    #[serde(default)]
    pub time_max: Option<f64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SkyMapPoint {
    pub scheduling_block_id: String,
    pub ra: f64,
    pub dec: f64,
    pub priority: f64,
    pub priority_bin: String,
    pub scheduled: bool,
    pub requested_hours: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SkyMapResponse {
    pub points: Vec<SkyMapPoint>,
    pub count: usize,
}

// ============================================================================
// Progress (SSE)
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct ProgressUpdate {
    pub stage: String,
    pub progress: u8, // 0-100
    pub message: String,
}
