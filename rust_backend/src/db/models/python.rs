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
    pub scheduling_block_id: i64,  // Internal DB ID (for internal operations)
    pub original_block_id: String,  // Original ID from JSON (shown to user)
    pub priority: f64,
    pub num_visibility_periods: usize,
    pub scheduled: bool,
}

#[pymethods]
impl VisibilityBlockSummary {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.original_block_id.clone()
    }
    
    #[getter]
    pub fn original_block_id(&self) -> String {
        self.original_block_id.clone()
    }

    #[getter]
    pub fn priority(&self) -> f64 {
        self.priority
    }

    #[getter]
    pub fn num_visibility_periods(&self) -> usize {
        self.num_visibility_periods
    }

    #[getter]
    pub fn scheduled(&self) -> bool {
        self.scheduled
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
    pub blocks: Vec<VisibilityBlockSummary>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
}

#[pymethods]
impl VisibilityMapData {
    #[getter]
    pub fn blocks(&self) -> Vec<VisibilityBlockSummary> {
        self.blocks.clone()
    }

    #[getter]
    pub fn priority_min(&self) -> f64 {
        self.priority_min
    }

    #[getter]
    pub fn priority_max(&self) -> f64 {
        self.priority_max
    }

    #[getter]
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    #[getter]
    pub fn scheduled_count(&self) -> usize {
        self.scheduled_count
    }

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
    pub scheduling_block_id: i64,  // Internal DB ID (for internal operations)
    pub original_block_id: String,  // Original ID from JSON (shown to user)
    pub priority: f64,
    pub scheduled_start_mjd: f64,
    pub scheduled_stop_mjd: f64,
    pub ra_deg: f64,
    pub dec_deg: f64,
    pub requested_hours: f64,
    pub total_visibility_hours: f64,
    pub num_visibility_periods: usize,
}

#[pymethods]
impl ScheduleTimelineBlock {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.original_block_id.clone()
    }
    
    #[getter]
    pub fn original_block_id(&self) -> String {
        self.original_block_id.clone()
    }

    #[getter]
    pub fn priority(&self) -> f64 {
        self.priority
    }

    #[getter]
    pub fn scheduled_start_mjd(&self) -> f64 {
        self.scheduled_start_mjd
    }

    #[getter]
    pub fn scheduled_stop_mjd(&self) -> f64 {
        self.scheduled_stop_mjd
    }

    #[getter]
    pub fn ra_deg(&self) -> f64 {
        self.ra_deg
    }

    #[getter]
    pub fn dec_deg(&self) -> f64 {
        self.dec_deg
    }

    #[getter]
    pub fn requested_hours(&self) -> f64 {
        self.requested_hours
    }

    #[getter]
    pub fn total_visibility_hours(&self) -> f64 {
        self.total_visibility_hours
    }

    #[getter]
    pub fn num_visibility_periods(&self) -> usize {
        self.num_visibility_periods
    }

    fn __repr__(&self) -> String {
        format!(
            "ScheduleTimelineBlock(id={}, priority={:.2}, start={:.2}, stop={:.2})",
            self.original_block_id, self.priority, self.scheduled_start_mjd, self.scheduled_stop_mjd
        )
    }
}

/// Complete schedule timeline data with blocks and computed metadata.
/// This structure contains everything the frontend needs to render the scheduled timeline.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct ScheduleTimelineData {
    pub blocks: Vec<ScheduleTimelineBlock>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub unique_months: Vec<String>,
    pub dark_periods: Vec<(f64, f64)>,
}

#[pymethods]
impl ScheduleTimelineData {
    #[getter]
    pub fn blocks(&self) -> Vec<ScheduleTimelineBlock> {
        self.blocks.clone()
    }

    #[getter]
    pub fn priority_min(&self) -> f64 {
        self.priority_min
    }

    #[getter]
    pub fn priority_max(&self) -> f64 {
        self.priority_max
    }

    #[getter]
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    #[getter]
    pub fn scheduled_count(&self) -> usize {
        self.scheduled_count
    }

    #[getter]
    pub fn unique_months(&self) -> Vec<String> {
        self.unique_months.clone()
    }

    #[getter]
    pub fn dark_periods(&self) -> Vec<(f64, f64)> {
        self.dark_periods.clone()
    }

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
    pub scheduling_block_id: i64,  // Internal DB ID (for internal operations)
    pub original_block_id: String,  // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub elevation_range_deg: f64,
    pub scheduled: bool,
    pub scheduled_start_mjd: Option<f64>,
    pub scheduled_stop_mjd: Option<f64>,
}

#[pymethods]
impl InsightsBlock {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.original_block_id.clone()
    }
    
    #[getter]
    pub fn original_block_id(&self) -> String {
        self.original_block_id.clone()
    }

    #[getter]
    pub fn priority(&self) -> f64 {
        self.priority
    }

    #[getter]
    pub fn total_visibility_hours(&self) -> f64 {
        self.total_visibility_hours
    }

    #[getter]
    pub fn requested_hours(&self) -> f64 {
        self.requested_hours
    }

    #[getter]
    pub fn elevation_range_deg(&self) -> f64 {
        self.elevation_range_deg
    }

    #[getter]
    pub fn scheduled(&self) -> bool {
        self.scheduled
    }

    #[getter]
    pub fn scheduled_start_mjd(&self) -> Option<f64> {
        self.scheduled_start_mjd
    }

    #[getter]
    pub fn scheduled_stop_mjd(&self) -> Option<f64> {
        self.scheduled_stop_mjd
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
    pub total_observations: usize,
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub scheduling_rate: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub mean_priority_scheduled: f64,
    pub mean_priority_unscheduled: f64,
    pub total_visibility_hours: f64,
    pub mean_requested_hours: f64,
}

#[pymethods]
impl AnalyticsMetrics {
    #[getter]
    pub fn total_observations(&self) -> usize {
        self.total_observations
    }

    #[getter]
    pub fn scheduled_count(&self) -> usize {
        self.scheduled_count
    }

    #[getter]
    pub fn unscheduled_count(&self) -> usize {
        self.unscheduled_count
    }

    #[getter]
    pub fn scheduling_rate(&self) -> f64 {
        self.scheduling_rate
    }

    #[getter]
    pub fn mean_priority(&self) -> f64 {
        self.mean_priority
    }

    #[getter]
    pub fn median_priority(&self) -> f64 {
        self.median_priority
    }

    #[getter]
    pub fn mean_priority_scheduled(&self) -> f64 {
        self.mean_priority_scheduled
    }

    #[getter]
    pub fn mean_priority_unscheduled(&self) -> f64 {
        self.mean_priority_unscheduled
    }

    #[getter]
    pub fn total_visibility_hours(&self) -> f64 {
        self.total_visibility_hours
    }

    #[getter]
    pub fn mean_requested_hours(&self) -> f64 {
        self.mean_requested_hours
    }

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
    pub variable1: String,
    pub variable2: String,
    pub correlation: f64,
}

#[pymethods]
impl CorrelationEntry {
    #[getter]
    pub fn variable1(&self) -> String {
        self.variable1.clone()
    }

    #[getter]
    pub fn variable2(&self) -> String {
        self.variable2.clone()
    }

    #[getter]
    pub fn correlation(&self) -> f64 {
        self.correlation
    }

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
    pub block_id_1: String,  // Original ID from JSON
    pub block_id_2: String,  // Original ID from JSON
    pub start_time_1: f64,
    pub stop_time_1: f64,
    pub start_time_2: f64,
    pub stop_time_2: f64,
    pub overlap_hours: f64,
}

#[pymethods]
impl ConflictRecord {
    #[getter]
    pub fn block_id_1(&self) -> String {
        self.block_id_1.clone()
    }

    #[getter]
    pub fn block_id_2(&self) -> String {
        self.block_id_2.clone()
    }

    #[getter]
    pub fn start_time_1(&self) -> f64 {
        self.start_time_1
    }

    #[getter]
    pub fn stop_time_1(&self) -> f64 {
        self.stop_time_1
    }

    #[getter]
    pub fn start_time_2(&self) -> f64 {
        self.start_time_2
    }

    #[getter]
    pub fn stop_time_2(&self) -> f64 {
        self.stop_time_2
    }

    #[getter]
    pub fn overlap_hours(&self) -> f64 {
        self.overlap_hours
    }

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
    pub scheduling_block_id: i64,  // Internal DB ID (for internal operations)
    pub original_block_id: String,  // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub scheduled: bool,
}

#[pymethods]
impl TopObservation {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.original_block_id.clone()
    }
    
    #[getter]
    pub fn original_block_id(&self) -> String {
        self.original_block_id.clone()
    }

    #[getter]
    pub fn priority(&self) -> f64 {
        self.priority
    }

    #[getter]
    pub fn total_visibility_hours(&self) -> f64 {
        self.total_visibility_hours
    }

    #[getter]
    pub fn requested_hours(&self) -> f64 {
        self.requested_hours
    }

    #[getter]
    pub fn scheduled(&self) -> bool {
        self.scheduled
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

#[pymethods]
impl InsightsData {
    #[getter]
    pub fn blocks(&self) -> Vec<InsightsBlock> {
        self.blocks.clone()
    }

    #[getter]
    pub fn metrics(&self) -> AnalyticsMetrics {
        self.metrics.clone()
    }

    #[getter]
    pub fn correlations(&self) -> Vec<CorrelationEntry> {
        self.correlations.clone()
    }

    #[getter]
    pub fn top_priority(&self) -> Vec<TopObservation> {
        self.top_priority.clone()
    }

    #[getter]
    pub fn top_visibility(&self) -> Vec<TopObservation> {
        self.top_visibility.clone()
    }

    #[getter]
    pub fn conflicts(&self) -> Vec<ConflictRecord> {
        self.conflicts.clone()
    }

    #[getter]
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    #[getter]
    pub fn scheduled_count(&self) -> usize {
        self.scheduled_count
    }

    #[getter]
    pub fn impossible_count(&self) -> usize {
        self.impossible_count
    }

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
    pub scheduling_block_id: i64,  // Internal DB ID (for internal operations)
    pub original_block_id: String,  // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub scheduled: bool,
}

#[pymethods]
impl TrendsBlock {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.original_block_id.clone()
    }
    
    #[getter]
    pub fn original_block_id(&self) -> String {
        self.original_block_id.clone()
    }

    #[getter]
    pub fn priority(&self) -> f64 {
        self.priority
    }

    #[getter]
    pub fn total_visibility_hours(&self) -> f64 {
        self.total_visibility_hours
    }

    #[getter]
    pub fn requested_hours(&self) -> f64 {
        self.requested_hours
    }

    #[getter]
    pub fn scheduled(&self) -> bool {
        self.scheduled
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
    pub bin_label: String,
    pub mid_value: f64,
    pub scheduled_rate: f64,
    pub count: usize,
}

#[pymethods]
impl EmpiricalRatePoint {
    #[getter]
    pub fn bin_label(&self) -> String {
        self.bin_label.clone()
    }

    #[getter]
    pub fn mid_value(&self) -> f64 {
        self.mid_value
    }

    #[getter]
    pub fn scheduled_rate(&self) -> f64 {
        self.scheduled_rate
    }

    #[getter]
    pub fn count(&self) -> usize {
        self.count
    }

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
    pub x: f64,
    pub y_smoothed: f64,
    pub n_samples: usize,
}

#[pymethods]
impl SmoothedPoint {
    #[getter]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[getter]
    pub fn y_smoothed(&self) -> f64 {
        self.y_smoothed
    }

    #[getter]
    pub fn n_samples(&self) -> usize {
        self.n_samples
    }

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
    pub visibility_mid: f64,
    pub time_mid: f64,
    pub scheduled_rate: f64,
    pub count: usize,
}

#[pymethods]
impl HeatmapBin {
    #[getter]
    pub fn visibility_mid(&self) -> f64 {
        self.visibility_mid
    }

    #[getter]
    pub fn time_mid(&self) -> f64 {
        self.time_mid
    }

    #[getter]
    pub fn scheduled_rate(&self) -> f64 {
        self.scheduled_rate
    }

    #[getter]
    pub fn count(&self) -> usize {
        self.count
    }

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
    pub total_count: usize,
    pub scheduled_count: usize,
    pub scheduling_rate: f64,
    pub zero_visibility_count: usize,
    pub priority_min: f64,
    pub priority_max: f64,
    pub priority_mean: f64,
    pub visibility_min: f64,
    pub visibility_max: f64,
    pub visibility_mean: f64,
    pub time_min: f64,
    pub time_max: f64,
    pub time_mean: f64,
}

#[pymethods]
impl TrendsMetrics {
    #[getter]
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    #[getter]
    pub fn scheduled_count(&self) -> usize {
        self.scheduled_count
    }

    #[getter]
    pub fn scheduling_rate(&self) -> f64 {
        self.scheduling_rate
    }

    #[getter]
    pub fn zero_visibility_count(&self) -> usize {
        self.zero_visibility_count
    }

    #[getter]
    pub fn priority_min(&self) -> f64 {
        self.priority_min
    }

    #[getter]
    pub fn priority_max(&self) -> f64 {
        self.priority_max
    }

    #[getter]
    pub fn priority_mean(&self) -> f64 {
        self.priority_mean
    }

    #[getter]
    pub fn visibility_min(&self) -> f64 {
        self.visibility_min
    }

    #[getter]
    pub fn visibility_max(&self) -> f64 {
        self.visibility_max
    }

    #[getter]
    pub fn visibility_mean(&self) -> f64 {
        self.visibility_mean
    }

    #[getter]
    pub fn time_min(&self) -> f64 {
        self.time_min
    }

    #[getter]
    pub fn time_max(&self) -> f64 {
        self.time_max
    }

    #[getter]
    pub fn time_mean(&self) -> f64 {
        self.time_mean
    }

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

#[pymethods]
impl TrendsData {
    #[getter]
    pub fn blocks(&self) -> Vec<TrendsBlock> {
        self.blocks.clone()
    }

    #[getter]
    pub fn metrics(&self) -> TrendsMetrics {
        self.metrics.clone()
    }

    #[getter]
    pub fn by_priority(&self) -> Vec<EmpiricalRatePoint> {
        self.by_priority.clone()
    }

    #[getter]
    pub fn by_visibility(&self) -> Vec<EmpiricalRatePoint> {
        self.by_visibility.clone()
    }

    #[getter]
    pub fn by_time(&self) -> Vec<EmpiricalRatePoint> {
        self.by_time.clone()
    }

    #[getter]
    pub fn smoothed_visibility(&self) -> Vec<SmoothedPoint> {
        self.smoothed_visibility.clone()
    }

    #[getter]
    pub fn smoothed_time(&self) -> Vec<SmoothedPoint> {
        self.smoothed_time.clone()
    }

    #[getter]
    pub fn heatmap_bins(&self) -> Vec<HeatmapBin> {
        self.heatmap_bins.clone()
    }

    #[getter]
    pub fn priority_values(&self) -> Vec<f64> {
        self.priority_values.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "TrendsData(total={}, scheduled={}, rate={:.2})",
            self.metrics.total_count,
            self.metrics.scheduled_count,
            self.metrics.scheduling_rate
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
    pub scheduling_block_id: String,
    pub priority: f64,
    pub scheduled: bool,
    pub requested_hours: f64,
}

#[pymethods]
impl CompareBlock {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.scheduling_block_id.clone()
    }

    #[getter]
    pub fn priority(&self) -> f64 {
        self.priority
    }

    #[getter]
    pub fn scheduled(&self) -> bool {
        self.scheduled
    }

    #[getter]
    pub fn requested_hours(&self) -> f64 {
        self.requested_hours
    }

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
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub total_priority: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub total_hours: f64,
}

#[pymethods]
impl CompareStats {
    #[getter]
    pub fn scheduled_count(&self) -> usize {
        self.scheduled_count
    }

    #[getter]
    pub fn unscheduled_count(&self) -> usize {
        self.unscheduled_count
    }

    #[getter]
    pub fn total_priority(&self) -> f64 {
        self.total_priority
    }

    #[getter]
    pub fn mean_priority(&self) -> f64 {
        self.mean_priority
    }

    #[getter]
    pub fn median_priority(&self) -> f64 {
        self.median_priority
    }

    #[getter]
    pub fn total_hours(&self) -> f64 {
        self.total_hours
    }

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
    pub scheduling_block_id: String,
    pub priority: f64,
    pub change_type: String, // "newly_scheduled" or "newly_unscheduled"
}

#[pymethods]
impl SchedulingChange {
    #[getter]
    pub fn scheduling_block_id(&self) -> String {
        self.scheduling_block_id.clone()
    }

    #[getter]
    pub fn priority(&self) -> f64 {
        self.priority
    }

    #[getter]
    pub fn change_type(&self) -> String {
        self.change_type.clone()
    }

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

#[pymethods]
impl CompareData {
    #[getter]
    pub fn current_blocks(&self) -> Vec<CompareBlock> {
        self.current_blocks.clone()
    }

    #[getter]
    pub fn comparison_blocks(&self) -> Vec<CompareBlock> {
        self.comparison_blocks.clone()
    }

    #[getter]
    pub fn current_stats(&self) -> CompareStats {
        self.current_stats.clone()
    }

    #[getter]
    pub fn comparison_stats(&self) -> CompareStats {
        self.comparison_stats.clone()
    }

    #[getter]
    pub fn common_ids(&self) -> Vec<String> {
        self.common_ids.clone()
    }

    #[getter]
    pub fn only_in_current(&self) -> Vec<String> {
        self.only_in_current.clone()
    }

    #[getter]
    pub fn only_in_comparison(&self) -> Vec<String> {
        self.only_in_comparison.clone()
    }

    #[getter]
    pub fn scheduling_changes(&self) -> Vec<SchedulingChange> {
        self.scheduling_changes.clone()
    }

    #[getter]
    pub fn current_name(&self) -> String {
        self.current_name.clone()
    }

    #[getter]
    pub fn comparison_name(&self) -> String {
        self.comparison_name.clone()
    }

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
