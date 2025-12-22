//! PyO3 wrapper types for Python interop.
//!
//! This module contains specialized types for Python integration, visualization data,
//! and advanced analytics:
//! - Visibility histogram types (VisibilityBlockSummary, VisibilityMapData, VisibilityBin)
//! - Timeline types (ScheduleTimelineBlock, ScheduleTimelineData)
//! - Insights types (InsightsBlock, AnalyticsMetrics, InsightsData)
//! - Trends types (TrendsBlock, TrendsData with empirical rates and heatmaps)
//! - Comparison types (CompareBlock, CompareData)

use pyo3::prelude::*;

// =========================================================
// Visibility Histogram Types
// =========================================================

/// Lightweight block summary for the visibility map.
/// Provides just enough information for filtering and statistics.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct VisibilityBlockSummary {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    #[pyo3(get)]
    pub original_block_id: String, // Original ID from JSON (shown to user)
    #[pyo3(get)]
    pub priority: f64,
    #[pyo3(get)]
    pub num_visibility_periods: usize,
    #[pyo3(get)]
    pub scheduled: bool,
}

#[pymethods]
impl VisibilityBlockSummary {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.original_block_id.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "VisibilityBlockSummary(id={}, priority={:.2}, periods={}, scheduled={})",
            self.original_block_id, self.priority, self.num_visibility_periods, self.scheduled
        )
    }
}

/// Data bundle for the visibility map UI.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct VisibilityMapData {
    #[pyo3(get)]
    pub blocks: Vec<VisibilityBlockSummary>,
    #[pyo3(get)]
    pub priority_min: f64,
    #[pyo3(get)]
    pub priority_max: f64,
    #[pyo3(get)]
    pub total_count: usize,
    #[pyo3(get)]
    pub scheduled_count: usize,
}

#[pymethods]
impl VisibilityMapData {

    fn __repr__(&self) -> String {
        format!(
            "VisibilityMapData(blocks={}, priority=[{:.2}, {:.2}], scheduled={}/{})",
            self.total_count,
            self.priority_min,
            self.priority_max,
            self.scheduled_count,
            self.total_count
        )
    }
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
    /// JSON string containing visibility periods: [{"start": mjd, "stop": mjd}, ...]
    pub visibility_periods_json: Option<String>,
}

// =========================================================
// Schedule Timeline Types
// =========================================================

/// Lightweight scheduling block for scheduled timeline visualizations.
/// Contains only the fields needed for monthly timeline plotting.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct ScheduleTimelineBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    #[pyo3(get)]
    pub original_block_id: String, // Original ID from JSON (shown to user)
    #[pyo3(get)]
    pub priority: f64,
    #[pyo3(get)]
    pub scheduled_start_mjd: f64,
    #[pyo3(get)]
    pub scheduled_stop_mjd: f64,
    #[pyo3(get)]
    pub ra_deg: f64,
    #[pyo3(get)]
    pub dec_deg: f64,
    #[pyo3(get)]
    pub requested_hours: f64,
    #[pyo3(get)]
    pub total_visibility_hours: f64,
    #[pyo3(get)]
    pub num_visibility_periods: usize,
}

#[pymethods]
impl ScheduleTimelineBlock {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.original_block_id.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "ScheduleTimelineBlock(id={}, priority={:.2}, start={:.2}, stop={:.2})",
            self.original_block_id,
            self.priority,
            self.scheduled_start_mjd,
            self.scheduled_stop_mjd
        )
    }
}

/// Complete schedule timeline data with blocks and computed metadata.
/// This structure contains everything the frontend needs to render the scheduled timeline.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct ScheduleTimelineData {
    #[pyo3(get)]
    pub blocks: Vec<ScheduleTimelineBlock>,
    #[pyo3(get)]
    pub priority_min: f64,
    #[pyo3(get)]
    pub priority_max: f64,
    #[pyo3(get)]
    pub total_count: usize,
    #[pyo3(get)]
    pub scheduled_count: usize,
    #[pyo3(get)]
    pub unique_months: Vec<String>,
    #[pyo3(get)]
    pub dark_periods: Vec<(f64, f64)>,
}

#[pymethods]
impl ScheduleTimelineData {

    fn __repr__(&self) -> String {
        format!(
            "ScheduleTimelineData(blocks={}, months={}, priority=[{:.2}, {:.2}])",
            self.total_count,
            self.unique_months.len(),
            self.priority_min,
            self.priority_max
        )
    }
}

// =========================================================
// Insights Types
// =========================================================

/// Lightweight block for insights analysis with all required metrics.
/// Contains only the fields needed for analytics computations.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct InsightsBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    #[pyo3(get)]
    pub original_block_id: String, // Original ID from JSON (shown to user)
    #[pyo3(get)]
    pub priority: f64,
    #[pyo3(get)]
    pub total_visibility_hours: f64,
    #[pyo3(get)]
    pub requested_hours: f64,
    #[pyo3(get)]
    pub elevation_range_deg: f64,
    #[pyo3(get)]
    pub scheduled: bool,
    #[pyo3(get)]
    pub scheduled_start_mjd: Option<f64>,
    #[pyo3(get)]
    pub scheduled_stop_mjd: Option<f64>,
}

#[pymethods]
impl InsightsBlock {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.original_block_id.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "InsightsBlock(id={}, priority={:.2}, scheduled={})",
            self.original_block_id, self.priority, self.scheduled
        )
    }
}

/// Analytics metrics computed from the dataset.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct AnalyticsMetrics {
    #[pyo3(get)]
    pub total_observations: usize,
    #[pyo3(get)]
    pub scheduled_count: usize,
    #[pyo3(get)]
    pub unscheduled_count: usize,
    #[pyo3(get)]
    pub scheduling_rate: f64,
    #[pyo3(get)]
    pub mean_priority: f64,
    #[pyo3(get)]
    pub median_priority: f64,
    #[pyo3(get)]
    pub mean_priority_scheduled: f64,
    #[pyo3(get)]
    pub mean_priority_unscheduled: f64,
    #[pyo3(get)]
    pub total_visibility_hours: f64,
    #[pyo3(get)]
    pub mean_requested_hours: f64,
}

#[pymethods]
impl AnalyticsMetrics {

    fn __repr__(&self) -> String {
        format!(
            "AnalyticsMetrics(total={}, scheduled={}, rate={:.2}%)",
            self.total_observations,
            self.scheduled_count,
            self.scheduling_rate * 100.0
        )
    }
}

/// Correlation entry for a pair of variables.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct CorrelationEntry {
    #[pyo3(get)]
    pub variable1: String,
    #[pyo3(get)]
    pub variable2: String,
    #[pyo3(get)]
    pub correlation: f64,
}

#[pymethods]
impl CorrelationEntry {

    fn __repr__(&self) -> String {
        format!(
            "CorrelationEntry({} <-> {}: {:.3})",
            self.variable1, self.variable2, self.correlation
        )
    }
}

/// Conflict record for overlapping scheduled observations.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct ConflictRecord {
    #[pyo3(get)]
    pub block_id_1: String, // Original ID from JSON
    #[pyo3(get)]
    pub block_id_2: String, // Original ID from JSON
    #[pyo3(get)]
    pub start_time_1: f64,
    #[pyo3(get)]
    pub stop_time_1: f64,
    #[pyo3(get)]
    pub start_time_2: f64,
    #[pyo3(get)]
    pub stop_time_2: f64,
    #[pyo3(get)]
    pub overlap_hours: f64,
}

#[pymethods]
impl ConflictRecord {

    fn __repr__(&self) -> String {
        format!(
            "ConflictRecord(blocks=({}, {}), overlap={:.2}h)",
            self.block_id_1, self.block_id_2, self.overlap_hours
        )
    }
}

/// Top observation record with all display fields.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct TopObservation {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    #[pyo3(get)]
    pub original_block_id: String, // Original ID from JSON (shown to user)
    #[pyo3(get)]
    pub priority: f64,
    #[pyo3(get)]
    pub total_visibility_hours: f64,
    #[pyo3(get)]
    pub requested_hours: f64,
    #[pyo3(get)]
    pub scheduled: bool,
}

#[pymethods]
impl TopObservation {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.original_block_id.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "TopObservation(id={}, priority={:.2}, visibility={:.1}h)",
            self.original_block_id, self.priority, self.total_visibility_hours
        )
    }
}

/// Complete insights data with metrics, correlations, top observations, and conflicts.
/// This structure contains everything the frontend needs for the insights page.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct InsightsData {
    #[pyo3(get)]
    pub blocks: Vec<InsightsBlock>,
    #[pyo3(get)]
    pub metrics: AnalyticsMetrics,
    #[pyo3(get)]
    pub correlations: Vec<CorrelationEntry>,
    #[pyo3(get)]
    pub top_priority: Vec<TopObservation>,
    #[pyo3(get)]
    pub top_visibility: Vec<TopObservation>,
    #[pyo3(get)]
    pub conflicts: Vec<ConflictRecord>,
    #[pyo3(get)]
    pub total_count: usize,
    #[pyo3(get)]
    pub scheduled_count: usize,
    #[pyo3(get)]
    pub impossible_count: usize,
}

#[pymethods]
impl InsightsData {

    fn __repr__(&self) -> String {
        format!(
            "InsightsData(total={}, scheduled={}, conflicts={})",
            self.total_count,
            self.scheduled_count,
            self.conflicts.len()
        )
    }
}

// =========================================================
// Trends Types
// =========================================================

/// Lightweight block for trends analysis with required metrics.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct TrendsBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    #[pyo3(get)]
    pub original_block_id: String, // Original ID from JSON (shown to user)
    #[pyo3(get)]
    pub priority: f64,
    #[pyo3(get)]
    pub total_visibility_hours: f64,
    #[pyo3(get)]
    pub requested_hours: f64,
    #[pyo3(get)]
    pub scheduled: bool,
}

#[pymethods]
impl TrendsBlock {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.original_block_id.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "TrendsBlock(id={}, priority={:.2}, scheduled={})",
            self.original_block_id, self.priority, self.scheduled
        )
    }
}

/// Empirical rate data point for a single bin.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct EmpiricalRatePoint {
    #[pyo3(get)]
    pub bin_label: String,
    #[pyo3(get)]
    pub mid_value: f64,
    #[pyo3(get)]
    pub scheduled_rate: f64,
    #[pyo3(get)]
    pub count: usize,
}

#[pymethods]
impl EmpiricalRatePoint {

    fn __repr__(&self) -> String {
        format!(
            "EmpiricalRatePoint(mid={:.2}, rate={:.2}, n={})",
            self.mid_value, self.scheduled_rate, self.count
        )
    }
}

/// Smoothed trend data point.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct SmoothedPoint {
    #[pyo3(get)]
    pub x: f64,
    #[pyo3(get)]
    pub y_smoothed: f64,
    #[pyo3(get)]
    pub n_samples: usize,
}

#[pymethods]
impl SmoothedPoint {

    fn __repr__(&self) -> String {
        format!(
            "SmoothedPoint(x={:.2}, y={:.3}, n={})",
            self.x, self.y_smoothed, self.n_samples
        )
    }
}

/// Heatmap bin for 2D probability visualization.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct HeatmapBin {
    #[pyo3(get)]
    pub visibility_mid: f64,
    #[pyo3(get)]
    pub time_mid: f64,
    #[pyo3(get)]
    pub scheduled_rate: f64,
    #[pyo3(get)]
    pub count: usize,
}

#[pymethods]
impl HeatmapBin {

    fn __repr__(&self) -> String {
        format!(
            "HeatmapBin(vis={:.1}, time={:.1}, rate={:.2})",
            self.visibility_mid, self.time_mid, self.scheduled_rate
        )
    }
}

/// Overview metrics for trends analysis.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct TrendsMetrics {
    #[pyo3(get)]
    pub total_count: usize,
    #[pyo3(get)]
    pub scheduled_count: usize,
    #[pyo3(get)]
    pub scheduling_rate: f64,
    #[pyo3(get)]
    pub zero_visibility_count: usize,
    #[pyo3(get)]
    pub priority_min: f64,
    #[pyo3(get)]
    pub priority_max: f64,
    #[pyo3(get)]
    pub priority_mean: f64,
    #[pyo3(get)]
    pub visibility_min: f64,
    #[pyo3(get)]
    pub visibility_max: f64,
    #[pyo3(get)]
    pub visibility_mean: f64,
    #[pyo3(get)]
    pub time_min: f64,
    #[pyo3(get)]
    pub time_max: f64,
    #[pyo3(get)]
    pub time_mean: f64,
}

#[pymethods]
impl TrendsMetrics {

    fn __repr__(&self) -> String {
        format!(
            "TrendsMetrics(total={}, scheduled={}, rate={:.2})",
            self.total_count, self.scheduled_count, self.scheduling_rate
        )
    }
}

/// Complete trends data with empirical rates, smoothed curves, and heatmap bins.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct TrendsData {
    #[pyo3(get)]
    pub blocks: Vec<TrendsBlock>,
    #[pyo3(get)]
    pub metrics: TrendsMetrics,
    #[pyo3(get)]
    pub by_priority: Vec<EmpiricalRatePoint>,
    #[pyo3(get)]
    pub by_visibility: Vec<EmpiricalRatePoint>,
    #[pyo3(get)]
    pub by_time: Vec<EmpiricalRatePoint>,
    #[pyo3(get)]
    pub smoothed_visibility: Vec<SmoothedPoint>,
    #[pyo3(get)]
    pub smoothed_time: Vec<SmoothedPoint>,
    #[pyo3(get)]
    pub heatmap_bins: Vec<HeatmapBin>,
    #[pyo3(get)]
    pub priority_values: Vec<f64>,
}

#[pymethods]
impl TrendsData {

    fn __repr__(&self) -> String {
        format!(
            "TrendsData(total={}, scheduled={}, rate={:.2})",
            self.metrics.total_count, self.metrics.scheduled_count, self.metrics.scheduling_rate
        )
    }
}

// =========================================================
// Schedule Comparison Types
// =========================================================

/// Lightweight scheduling block for schedule comparison.
/// Contains fields needed for comparing two schedules.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct CompareBlock {
    #[pyo3(get)]
    pub scheduling_block_id: String,
    #[pyo3(get)]
    pub priority: f64,
    #[pyo3(get)]
    pub scheduled: bool,
    #[pyo3(get)]
    pub requested_hours: f64,
}

#[pymethods]
impl CompareBlock {

    fn __repr__(&self) -> String {
        format!(
            "CompareBlock(id={}, priority={:.2}, scheduled={}, hours={:.1})",
            self.scheduling_block_id, self.priority, self.scheduled, self.requested_hours
        )
    }
}

/// Summary statistics for schedule comparison.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct CompareStats {
    #[pyo3(get)]
    pub scheduled_count: usize,
    #[pyo3(get)]
    pub unscheduled_count: usize,
    #[pyo3(get)]
    pub total_priority: f64,
    #[pyo3(get)]
    pub mean_priority: f64,
    #[pyo3(get)]
    pub median_priority: f64,
    #[pyo3(get)]
    pub total_hours: f64,
}

#[pymethods]
impl CompareStats {

    fn __repr__(&self) -> String {
        format!(
            "CompareStats(scheduled={}, mean_priority={:.2}, total_hours={:.1})",
            self.scheduled_count, self.mean_priority, self.total_hours
        )
    }
}

/// Change tracking for blocks between schedules.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct SchedulingChange {
    #[pyo3(get)]
    pub scheduling_block_id: String,
    #[pyo3(get)]
    pub priority: f64,
    #[pyo3(get)]
    pub change_type: String, // "newly_scheduled" or "newly_unscheduled"
}

#[pymethods]
impl SchedulingChange {

    fn __repr__(&self) -> String {
        format!(
            "SchedulingChange(id={}, priority={:.2}, type={})",
            self.scheduling_block_id, self.priority, self.change_type
        )
    }
}

/// Complete comparison data for two schedules.
/// Contains blocks from both schedules and computed statistics.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct CompareData {
    #[pyo3(get)]
    pub current_blocks: Vec<CompareBlock>,
    #[pyo3(get)]
    pub comparison_blocks: Vec<CompareBlock>,
    #[pyo3(get)]
    pub current_stats: CompareStats,
    #[pyo3(get)]
    pub comparison_stats: CompareStats,
    #[pyo3(get)]
    pub common_ids: Vec<String>,
    #[pyo3(get)]
    pub only_in_current: Vec<String>,
    #[pyo3(get)]
    pub only_in_comparison: Vec<String>,
    #[pyo3(get)]
    pub scheduling_changes: Vec<SchedulingChange>,
    #[pyo3(get)]
    pub current_name: String,
    #[pyo3(get)]
    pub comparison_name: String,
}

#[pymethods]
impl CompareData {

    fn __repr__(&self) -> String {
        format!(
            "CompareData(current={} blocks, comparison={} blocks, common={}, changes={})",
            self.current_blocks.len(),
            self.comparison_blocks.len(),
            self.common_ids.len(),
            self.scheduling_changes.len()
        )
    }
}
