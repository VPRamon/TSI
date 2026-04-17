use serde::{Deserialize, Serialize};

// =========================================================
// Compare types
// =========================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareBlock {
    /// Opaque per-schedule identifier (database block id as a string).
    /// Never used as the cross-schedule match key.
    pub scheduling_block_id: String,
    /// Canonical cross-schedule match key. Non-empty values are matched across
    /// schedules; empty means the block is unique to its schedule.
    #[serde(default)]
    pub original_block_id: String,
    #[serde(default)]
    pub block_name: String,
    pub priority: f64,
    pub scheduled: bool,
    pub requested_hours: qtty::Hours,
    /// Scheduled start in MJD (only populated when the block is scheduled).
    #[serde(default)]
    pub scheduled_start_mjd: Option<f64>,
    /// Scheduled stop in MJD (only populated when the block is scheduled).
    #[serde(default)]
    pub scheduled_stop_mjd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareStats {
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    /// Sum of priorities over scheduled blocks. Rendered in the UI as
    /// "Cumulative priority".
    pub total_priority: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub total_hours: qtty::Hours,
    pub gap_count: Option<i32>,
    pub gap_mean_hours: Option<qtty::Hours>,
    pub gap_median_hours: Option<qtty::Hours>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingChange {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub change_type: String,
}

/// Table-ready row capturing a block's identity and both schedules' state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareDiffBlock {
    pub original_block_id: String,
    pub block_name: String,
    pub priority: f64,
    pub requested_hours: qtty::Hours,
    pub current_scheduling_block_id: Option<String>,
    pub comparison_scheduling_block_id: Option<String>,
    pub current_scheduled_start_mjd: Option<f64>,
    pub current_scheduled_stop_mjd: Option<f64>,
    pub comparison_scheduled_start_mjd: Option<f64>,
    pub comparison_scheduled_stop_mjd: Option<f64>,
}

/// A block scheduled in both schedules but with a different scheduled window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetimedBlockChange {
    pub original_block_id: String,
    pub block_name: String,
    pub priority: f64,
    pub requested_hours: qtty::Hours,
    pub current_scheduling_block_id: Option<String>,
    pub comparison_scheduling_block_id: Option<String>,
    pub current_scheduled_start_mjd: Option<f64>,
    pub current_scheduled_stop_mjd: Option<f64>,
    pub comparison_scheduled_start_mjd: Option<f64>,
    pub comparison_scheduled_stop_mjd: Option<f64>,
    pub start_shift_hours: f64,
    pub stop_shift_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareData {
    pub current_blocks: Vec<CompareBlock>,
    pub comparison_blocks: Vec<CompareBlock>,
    pub current_stats: CompareStats,
    pub comparison_stats: CompareStats,

    // Legacy arrays — kept for API compatibility. Populated from matched
    // original_block_id values; frontend uses the grouped fields below.
    pub common_ids: Vec<String>,
    pub only_in_current: Vec<String>,
    pub only_in_comparison: Vec<String>,
    pub scheduling_changes: Vec<SchedulingChange>,

    // Grouped diff tables.
    pub scheduled_only_current: Vec<CompareDiffBlock>,
    pub scheduled_only_comparison: Vec<CompareDiffBlock>,
    pub only_in_current_blocks: Vec<CompareDiffBlock>,
    pub only_in_comparison_blocks: Vec<CompareDiffBlock>,
    pub retimed_blocks: Vec<RetimedBlockChange>,

    pub current_name: String,
    pub comparison_name: String,
}

pub const GET_COMPARE_DATA: &str = "get_compare_data";
