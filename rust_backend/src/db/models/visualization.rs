//! Visualization and analytics domain models.
//!
//! This module contains specialized types for visualization data and analytics:
//! - Visibility histogram types (VisibilityBlockSummary, VisibilityMapData, VisibilityBin)
//! - Timeline types (ScheduleTimelineBlock, ScheduleTimelineData)
//! - Insights types (InsightsBlock, AnalyticsMetrics, InsightsData)
//! - Trends types (TrendsBlock, TrendsData with empirical rates and heatmaps)
//! - Comparison types (CompareBlock, CompareData)


use siderust::astro::ModifiedJulianDate;
use qtty::*;
use super::Period;

// =========================================================
// Visibility Histogram Types
// =========================================================

/// Lightweight block summary for the visibility map.
/// Provides just enough information for filtering and statistics.
#[derive(Debug, Clone)]
pub struct VisibilityBlockSummary {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub num_visibility_periods: usize,
    pub scheduled: bool,
}


/// Data bundle for the visibility map UI.
#[derive(Debug, Clone)]
pub struct VisibilityMapData {
    pub blocks: Vec<VisibilityBlockSummary>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
}


/// Represents a single time bin in a visibility histogram.
/// Used internally in Rust for efficient computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibilityBin {
    /// Start of the bin as Unix timestamp (seconds since epoch)
    pub bin_start_unix: i64,
    /// End of the bin as Unix timestamp (seconds since epoch)
    pub bin_end_unix: i64,
    /// Number of unique scheduling blocks visible in this bin
    pub visible_count: u32,
}

impl VisibilityBin {
    /// Create a new visibility bin
    pub fn new(bin_start_unix: i64, bin_end_unix: i64, visible_count: u32) -> Self {
        Self {
            bin_start_unix,
            bin_end_unix,
            visible_count,
        }
    }

    /// Check if a time period (in Unix timestamps) overlaps with this bin
    pub fn overlaps_period(&self, period_start_unix: i64, period_end_unix: i64) -> bool {
        period_start_unix < self.bin_end_unix && period_end_unix > self.bin_start_unix
    }
}

/// A row from the database containing minimal data needed for histogram computation
#[derive(Debug, Clone)]
pub struct BlockHistogramData {
    /// Scheduling block ID
    pub scheduling_block_id: i64,
    /// Priority of the block
    pub priority: i32,
    /// Visibility periods for this block
    pub visibility_periods: Option<Vec<Period>>,
}

// =========================================================
// Schedule Timeline Types
// =========================================================

/// Lightweight scheduling block for scheduled timeline visualizations.
/// Contains only the fields needed for monthly timeline plotting.
#[derive(Debug, Clone)]
pub struct ScheduleTimelineBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub scheduled_start_mjd: ModifiedJulianDate,
    pub scheduled_stop_mjd: ModifiedJulianDate,
    pub ra_deg: Degrees,
    pub dec_deg: Degrees,
    pub requested_hours: Hours,
    pub total_visibility_hours: Hours,
    pub num_visibility_periods: usize,
}


// =========================================================
// Insights Types
// =========================================================

/// Lightweight block for insights analysis with all required metrics.
/// Contains only the fields needed for analytics computations.
#[derive(Debug, Clone)]
pub struct InsightsBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: Hours,
    pub requested_hours: Hours,
    pub elevation_range_deg: Degrees,
    pub scheduled: bool,
    pub scheduled_start_mjd: Option<ModifiedJulianDate>,
    pub scheduled_stop_mjd: Option<ModifiedJulianDate>,
}


/// Analytics metrics computed from the dataset.
#[derive(Debug, Clone)]
pub struct AnalyticsMetrics {
    pub total_observations: usize,
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub scheduling_rate: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub mean_priority_scheduled: f64,
    pub mean_priority_unscheduled: f64,
    pub total_visibility_hours: Hours,
    pub mean_requested_hours: Hours,
}


/// Correlation entry for a pair of variables.
#[derive(Debug, Clone)]
pub struct CorrelationEntry {
    pub variable1: String,
    pub variable2: String,
    pub correlation: f64,
}


/// Conflict record for overlapping scheduled observations.
#[derive(Debug, Clone)]
pub struct ConflictRecord {
    pub block_id_1: String, // Original ID from JSON
    pub block_id_2: String, // Original ID from JSON
    pub start_time_1: ModifiedJulianDate,
    pub stop_time_1: ModifiedJulianDate,
    pub start_time_2: ModifiedJulianDate,
    pub stop_time_2: ModifiedJulianDate,
    pub overlap_hours: Hours,
}


/// Top observation record with all display fields.
#[derive(Debug, Clone)]
pub struct TopObservation {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: Hours,
    pub requested_hours: Hours,
    pub scheduled: bool,
}


/// Complete insights data with metrics, correlations, top observations, and conflicts.
/// This structure contains everything the frontend needs for the insights page.
#[derive(Debug, Clone)]
pub struct InsightsData {
    pub blocks: Vec<InsightsBlock>,
    pub metrics: AnalyticsMetrics,
    pub correlations: Vec<CorrelationEntry>,
    pub top_priority: Vec<TopObservation>,
    pub top_visibility: Vec<TopObservation>,
    pub conflicts: Vec<ConflictRecord>,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub impossible_count: usize,
}


// =========================================================
// Trends Types
// =========================================================

/// Lightweight block for trends analysis with required metrics.
#[derive(Debug, Clone)]
pub struct TrendsBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: Hours,
    pub requested_hours: Hours,
    pub scheduled: bool,
}


/// Empirical rate data point for a single bin.
#[derive(Debug, Clone)]
pub struct EmpiricalRatePoint {
    pub bin_label: String,
    pub mid_value: f64,
    pub scheduled_rate: f64,
    pub count: usize,
}


/// Smoothed trend data point.
#[derive(Debug, Clone)]
pub struct SmoothedPoint {
    pub x: f64,
    pub y_smoothed: f64,
    pub n_samples: usize,
}


/// Heatmap bin for 2D probability visualization.
#[derive(Debug, Clone)]
pub struct HeatmapBin {
    pub visibility_mid: Hours,
    pub time_mid: Hours,
    pub scheduled_rate: f64,
    pub count: usize,
}


/// Overview metrics for trends analysis.
#[derive(Debug, Clone)]
pub struct TrendsMetrics {
    pub total_count: usize,
    pub scheduled_count: usize,
    pub scheduling_rate: f64,
    pub zero_visibility_count: usize,
    pub priority_min: f64,
    pub priority_max: f64,
    pub priority_mean: f64,
    pub visibility_min: Hours,
    pub visibility_max: Hours,
    pub visibility_mean: Hours,
    pub time_min: Hours,
    pub time_max: Hours,
    pub time_mean: Hours,
}


/// Complete trends data with empirical rates, smoothed curves, and heatmap bins.
#[derive(Debug, Clone)]
pub struct TrendsData {
    pub blocks: Vec<TrendsBlock>,
    pub metrics: TrendsMetrics,
    pub by_priority: Vec<EmpiricalRatePoint>,
    pub by_visibility: Vec<EmpiricalRatePoint>,
    pub by_time: Vec<EmpiricalRatePoint>,
    pub smoothed_visibility: Vec<SmoothedPoint>,
    pub smoothed_time: Vec<SmoothedPoint>,
    pub heatmap_bins: Vec<HeatmapBin>,
    pub priority_values: Vec<f64>,
}


// =========================================================
// Schedule Comparison Types
// =========================================================

/// Lightweight scheduling block for schedule comparison.
/// Contains fields needed for comparing two schedules.
#[derive(Debug, Clone)]
pub struct CompareBlock {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub scheduled: bool,
    pub requested_hours: Hours,
}


/// Summary statistics for schedule comparison.
#[derive(Debug, Clone)]
pub struct CompareStats {
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub total_priority: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub total_hours: Hours,
}


/// Change tracking for blocks between schedules.
#[derive(Debug, Clone)]
pub struct SchedulingChange {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub change_type: String, // "newly_scheduled" or "newly_unscheduled"
}


/// Complete comparison data for two schedules.
/// Contains blocks from both schedules and computed statistics.
#[derive(Debug, Clone)]
pub struct CompareData {
    pub current_blocks: Vec<CompareBlock>,
    pub comparison_blocks: Vec<CompareBlock>,
    pub current_stats: CompareStats,
    pub comparison_stats: CompareStats,
    pub common_ids: Vec<String>,
    pub only_in_current: Vec<String>,
    pub only_in_comparison: Vec<String>,
    pub scheduling_changes: Vec<SchedulingChange>,
    pub current_name: String,
    pub comparison_name: String,
}