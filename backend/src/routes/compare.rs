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

// =========================================================
// Advanced compare types
// =========================================================

/// Parameters actually used by the advanced compare pipeline, echoed back so
/// the caller can verify which defaults were applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCompareParams {
    pub epsilon_minutes: f64,
    pub min_block_size: usize,
    pub merge_epsilon_minutes: f64,
}

/// Global metrics for the advanced coherent-block comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedGlobalMetrics {
    /// `common / (common + only_in_current + only_in_comparison)` over
    /// identifiable blocks (non-empty `original_block_id`).
    pub match_ratio: f64,
    pub matched_count: usize,
    /// Common tasks with valid `scheduled_start_mjd` + `scheduled_stop_mjd`
    /// in **both** schedules.
    pub timed_common_count: usize,
    pub only_in_current_count: usize,
    pub only_in_comparison_count: usize,
    pub coherent_block_count: usize,
    pub ungrouped_common_count: usize,
    /// `LIS(pos_b ordered by pos_a) / timed_common_count`.
    /// `null` when `timed_common_count == 0`.
    pub order_preservation_ratio: Option<f64>,
    /// Median of per-task `shift_minutes` (comparison − current start).
    /// `null` when `timed_common_count == 0`.
    pub global_shift_median_minutes: Option<f64>,
    /// MAD (median absolute deviation) of per-task shift relative to the
    /// global median. `null` when `timed_common_count == 0`.
    pub local_shift_mad_minutes: Option<f64>,
    /// Blocks with empty `original_block_id` in the current schedule.
    pub ignored_missing_key_current: usize,
    /// Blocks with empty `original_block_id` in the comparison schedule.
    pub ignored_missing_key_comparison: usize,
}

/// A single coherent block in the segmented comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherentBlock {
    pub block_index: usize,
    pub original_block_ids: Vec<String>,
    pub size: usize,
    pub pos_a_start: usize,
    pub pos_a_end: usize,
    pub pos_b_start: usize,
    pub pos_b_end: usize,
    pub start_a_mjd: f64,
    pub end_a_mjd: f64,
    pub start_b_mjd: f64,
    pub end_b_mjd: f64,
    pub avg_shift_minutes: f64,
    pub shift_std_minutes: f64,
}

/// Top-level wrapper for the advanced compare payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedCompare {
    pub params_used: AdvancedCompareParams,
    pub global_metrics: AdvancedGlobalMetrics,
    pub blocks: Vec<CoherentBlock>,
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

    pub advanced_compare: AdvancedCompare,
}

pub const GET_COMPARE_DATA: &str = "get_compare_data";
