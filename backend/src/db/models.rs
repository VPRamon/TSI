//! Shared data models re-exported for database layer consumers.

pub use crate::api::{
    CompareBlock, Constraints, EmpiricalRatePoint, HeatmapBin, InsightsBlock, Period, Schedule,
    ScheduleTimelineBlock, ScheduleTimelineData, SchedulingBlock, TrendsBlock, TrendsData,
    TrendsMetrics, ValidationIssue, ValidationReport, VisibilityBlockSummary, VisibilityMapData,
};
pub use crate::models::ModifiedJulianDate;
pub use crate::routes::insights::{
    AnalyticsMetrics, ConflictRecord, CorrelationEntry, InsightsData, TopObservation,
};

/// Minimal visibility histogram input row.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlockHistogramData {
    pub scheduling_block_id: i64,
    pub priority: i32,
    pub visibility_periods: Option<Vec<Period>>,
}

/// Visibility histogram bin.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct VisibilityBin {
    pub bin_start_unix: i64,
    pub bin_end_unix: i64,
    pub visible_count: i64,
}

impl VisibilityBin {
    pub fn new(bin_start_unix: i64, bin_end_unix: i64, visible_count: i64) -> Self {
        Self {
            bin_start_unix,
            bin_end_unix,
            visible_count,
        }
    }
}
