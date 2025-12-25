//! Python-facing Data Transfer Objects (DTOs).
//!
//! This module defines all `#[pyclass]` types exposed to Python through PyO3.
//! These types use only PyO3-compatible primitives (String, f64, Vec, HashMap, etc.)
//! and are isolated from internal Rust models that may use qtty types, siderust types,
//! or complex generic parameters.
//!
//! ## Design Guidelines
//!
//! 1. **Primitives Only**: Use f64 for MJD/Degrees/etc., String for IDs
//! 2. **Flat Structures**: Avoid deep nesting, optimize for Python ergonomics  
//! 3. **No qtty**: All strongly-typed quantities converted to primitives at API boundary
//! 4. **Serializable**: All types should support to/from Python dict/JSON
//! 5. **Documented**: Each field should be clear to Python users

use pyo3::prelude::*;
use pyo3::types::PyTuple;

use serde::{Deserialize, Serialize};

// =========================================================
// Core Schedule Types
// =========================================================

/// Strongly-typed identifier for a schedule record.
#[pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScheduleId(pub i64);

#[pymethods]
impl ScheduleId {
    #[new]
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    #[getter]
    pub fn value(&self) -> i64 {
        self.0
    }

    fn __repr__(&self) -> String {
        format!("ScheduleId({})", self.0)
    }
}

/// Strongly-typed identifier for a target record.
#[pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetId(pub i64);

#[pymethods]
impl TargetId {
    #[new]
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    #[getter]
    pub fn value(&self) -> i64 {
        self.0
    }

    fn __repr__(&self) -> String {
        format!("TargetId({})", self.0)
    }
}

/// Strongly-typed identifier for a constraints record.
#[pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstraintsId(pub i64);

#[pymethods]
impl ConstraintsId {
    #[new]
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    #[getter]
    pub fn value(&self) -> i64 {
        self.0
    }

    fn __repr__(&self) -> String {
        format!("ConstraintsId({})", self.0)
    }
}

/// Strongly-typed identifier for a scheduling block.
#[pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchedulingBlockId(pub i64);

#[pymethods]
impl SchedulingBlockId {
    #[new]
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    #[getter]
    pub fn value(&self) -> i64 {
        self.0
    }

    fn __repr__(&self) -> String {
        format!("SchedulingBlockId({})", self.0)
    }
}

/// Time period in Modified Julian Date (MJD) format.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    /// Start time in MJD
    pub start: f64,
    /// End time in MJD
    pub stop: f64,
}

#[pymethods]
impl Period {
    #[new]
    pub fn py_new(start: f64, stop: f64) -> Self {
        Self { start, stop }
    }

    #[staticmethod]
    pub fn from_datetime(start: Py<PyAny>, stop: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| {
            let datetime_mod = py.import("datetime")?;
            let timezone_utc = datetime_mod.getattr("timezone")?.getattr("utc")?;

            // Helper to convert a datetime object to MJD
            let to_mjd = |dt: &Py<PyAny>| -> PyResult<f64> {
                let dt_obj = dt.as_ref();
                let tzinfo = dt_obj.getattr(py, "tzinfo")?;

                let timestamp = if tzinfo.is_none(py) {
                    // Naive datetime - assume UTC
                    let kwargs = pyo3::types::PyDict::new(py);
                    kwargs.set_item("tzinfo", &timezone_utc)?;
                    let aware = dt_obj.call_method(py, "replace", (), Some(&kwargs))?;
                    aware.call_method0(py, "timestamp")?.extract::<f64>(py)?
                } else {
                    dt_obj.call_method0(py, "timestamp")?.extract::<f64>(py)?
                };

                // Convert Unix timestamp to MJD (MJD 0 = 1858-11-17 00:00:00 UTC)
                let mjd = timestamp / 86400.0 + 40587.0;
                Ok(mjd)
            };

            let start_mjd = to_mjd(&start)?;
            let stop_mjd = to_mjd(&stop)?;

            Ok(Self {
                start: start_mjd,
                stop: stop_mjd,
            })
        })
    }

    #[getter]
    pub fn start_mjd(&self) -> f64 {
        self.start
    }

    #[getter]
    pub fn stop_mjd(&self) -> f64 {
        self.stop
    }

    pub fn contains_mjd(&self, mjd: f64) -> bool {
        let min_mjd = self.start.min(self.stop);
        let max_mjd = self.start.max(self.stop);
        mjd >= min_mjd && mjd <= max_mjd
    }

    pub fn to_datetime<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        // Convert MJD -> seconds since UNIX epoch then use Python's datetime
        let s_secs = (self.start - 40587.0) * 86400.0;
        let e_secs = (self.stop - 40587.0) * 86400.0;

        let datetime_mod = py.import("datetime")?;
        let datetime_cls = datetime_mod.getattr("datetime")?;
        let timezone_utc = datetime_mod.getattr("timezone")?.getattr("utc")?;

        let s_dt = datetime_cls.call_method1("fromtimestamp", (s_secs, timezone_utc.clone()))?;
        let e_dt = datetime_cls.call_method1("fromtimestamp", (e_secs, timezone_utc))?;

        PyTuple::new(py, [s_dt, e_dt])
    }
}


/// Observing constraints for a scheduling block.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    /// Minimum altitude in degrees
    pub min_alt: f64,
    /// Maximum altitude in degrees
    pub max_alt: f64,
    /// Minimum azimuth in degrees
    pub min_az: f64,
    /// Maximum azimuth in degrees
    pub max_az: f64,
    /// Fixed observation time window in MJD
    pub fixed_time: Option<Period>,
}

#[pymethods]
impl Constraints {
    #[new]
    #[pyo3(signature = (min_alt, max_alt, min_az, max_az, fixed_time=None))]
    pub fn new(
        min_alt: f64,
        max_alt: f64,
        min_az: f64,
        max_az: f64,
        fixed_time: Option<Period>,
    ) -> Self {
        Self {
            min_alt,
            max_alt,
            min_az,
            max_az,
            fixed_time,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Constraints(alt=[{:.2}, {:.2}], az=[{:.2}, {:.2}], fixed={:?})",
            self.min_alt, self.max_alt, self.min_az, self.max_az, self.fixed_time
        )
    }
}

/// Individual scheduling block (observation request).
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingBlock {
    /// Database ID for the block
    pub id: i64,
    /// Original ID from JSON (shown to user)
    pub original_block_id: Option<String>,
    /// Right Ascension in degrees (ICRS)
    pub target_ra: f64,
    /// Declination in degrees (ICRS)
    pub target_dec: f64,
    /// Observing constraints
    pub constraints: Constraints,
    /// Observation priority
    pub priority: f64,
    /// Minimum observation duration in seconds
    pub min_observation: f64,
    /// Requested observation duration in seconds
    pub requested_duration: f64,
    /// Visibility periods in MJD
    pub visibility_periods: Vec<Period>,
    /// Scheduled time window in MJD (if scheduled)
    pub scheduled_period: Option<Period>,
}

#[pymethods]
impl SchedulingBlock {
    #[new]
    #[pyo3(signature = (id, original_block_id, target_ra, target_dec, constraints, priority, min_observation, requested_duration, visibility_periods=None, scheduled_period=None))]
    pub fn new(
        id: i64,
        original_block_id: Option<String>,
        target_ra: f64,
        target_dec: f64,
        constraints: Constraints,
        priority: f64,
        min_observation: f64,
        requested_duration: f64,
        visibility_periods: Option<Vec<Period>>,
        scheduled_period: Option<Period>,
    ) -> Self {
        Self {
            id,
            original_block_id,
            target_ra,
            target_dec,
            constraints,
            priority,
            min_observation,
            requested_duration,
            visibility_periods: visibility_periods.unwrap_or_default(),
            scheduled_period,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "SchedulingBlock(id={}, ra={:.2}, dec={:.2}, priority={:.1})",
            self.id, self.target_ra, self.target_dec, self.priority
        )
    }
}

/// Top-level schedule with metadata and blocks.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// Database ID
    pub id: Option<i64>,
    /// Schedule name
    pub name: String,
    /// SHA256 checksum of schedule data
    pub checksum: String,
    /// Dark periods (observing windows)
    pub dark_periods: Vec<Period>,
    /// List of scheduling blocks
    pub blocks: Vec<SchedulingBlock>,
}

#[pymethods]
impl Schedule {
    #[new]
    pub fn new(
        id: Option<i64>,
        name: String,
        checksum: String,
        dark_periods: Vec<Period>,
        blocks: Vec<SchedulingBlock>,
    ) -> Self {
        Self {
            id,
            name,
            checksum,
            dark_periods,
            blocks,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Schedule(name='{}', blocks={}, dark_periods={})",
            self.name,
            self.blocks.len(),
            self.dark_periods.len()
        )
    }
}

/// Metadata about a stored schedule.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMetadata {
    /// Database ID
    pub schedule_id: Option<i64>,
    /// Schedule name
    pub schedule_name: String,
    /// Creation timestamp (ISO format)
    pub upload_timestamp: String,
    /// SHA256 checksum of schedule data
    pub checksum: String,
}

#[pymethods]
impl ScheduleMetadata {
    #[new]
    pub fn new(
        schedule_id: Option<i64>,
        schedule_name: String,
        upload_timestamp: String,
        checksum: String,
    ) -> Self {
        Self {
            schedule_id,
            schedule_name,
            upload_timestamp,
            checksum,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ScheduleMetadata(id={:?}, name='{}', timestamp='{}')",
            self.schedule_id, self.schedule_name, self.upload_timestamp
        )
    }
}

/// Schedule information with block counts.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleInfo {
    pub metadata: ScheduleMetadata,
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
    pub unscheduled_blocks: usize,
}

#[pymethods]
impl ScheduleInfo {
    fn __repr__(&self) -> String {
        format!(
            "ScheduleInfo(id={:?}, name='{}', blocks={}/{})",
            self.metadata.schedule_id,
            self.metadata.schedule_name,
            self.scheduled_blocks,
            self.total_blocks
        )
    }
}

// =========================================================
// Analytics Types - Lightweight Block
// =========================================================

/// Minimal block data for visualization queries.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightweightBlock {
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub priority_bin: String,
    pub requested_duration_seconds: f64,
    pub target_ra_deg: f64,
    pub target_dec_deg: f64,
    pub scheduled_period: Option<Period>,
}

// =========================================================
// Sky Map Types
// =========================================================

/// Priority bin information for sky map.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityBinInfo {
    pub label: String,
    pub min_priority: f64,
    pub max_priority: f64,
    pub color: String,
}

/// Sky map visualization data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyMapData {
    pub blocks: Vec<LightweightBlock>,
    pub priority_bins: Vec<PriorityBinInfo>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub ra_min: f64,
    pub ra_max: f64,
    pub dec_min: f64,
    pub dec_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub scheduled_time_min: Option<f64>,
    pub scheduled_time_max: Option<f64>,
}

// =========================================================
// Distribution Types
// =========================================================

/// Block data for distribution analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionBlock {
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub elevation_range_deg: f64,
    pub scheduled: bool,
}

/// Distribution statistics.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

/// Complete distribution dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionData {
    pub blocks: Vec<DistributionBlock>,
    pub priority_stats: DistributionStats,
    pub visibility_stats: DistributionStats,
    pub requested_hours_stats: DistributionStats,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub impossible_count: usize,
}

// =========================================================
// Timeline Types
// =========================================================

/// Timeline block data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTimelineBlock {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
    pub priority: f64,
    pub scheduled_start_mjd: f64,
    pub scheduled_stop_mjd: f64,
    pub ra_deg: f64,
    pub dec_deg: f64,
    pub requested_hours: f64,
    pub total_visibility_hours: f64,
    pub num_visibility_periods: usize,
}

/// Schedule timeline dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTimelineData {
    pub blocks: Vec<ScheduleTimelineBlock>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub unique_months: Vec<String>,
    pub dark_periods: Vec<Period>,
}

// =========================================================
// Insights Types
// =========================================================

/// Block data for insights analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsBlock {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub elevation_range_deg: f64,
    pub scheduled: bool,
    pub scheduled_start_mjd: Option<f64>,
    pub scheduled_stop_mjd: Option<f64>,
}

/// Analytics metrics.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Correlation entry.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationEntry {
    pub variable1: String,
    pub variable2: String,
    pub correlation: f64,
}

/// Conflict record.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub block_id_1: String,
    pub block_id_2: String,
    pub start_time_1: f64,
    pub stop_time_1: f64,
    pub start_time_2: f64,
    pub stop_time_2: f64,
    pub overlap_hours: f64,
}

/// Top observation entry.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopObservation {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub scheduled: bool,
}

/// Complete insights dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Block data for trends analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsBlock {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub scheduled: bool,
}

/// Empirical scheduling rate point.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpiricalRatePoint {
    pub bin_label: String,
    pub mid_value: f64,
    pub scheduled_rate: f64,
    pub count: usize,
}

/// Smoothed trend point.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmoothedPoint {
    pub x: f64,
    pub y_smoothed: f64,
    pub n_samples: usize,
}

/// Heatmap bin data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapBin {
    pub visibility_mid: f64,
    pub time_mid: f64,
    pub scheduled_rate: f64,
    pub count: usize,
}

/// Trends metrics summary.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Complete trends dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
// Comparison Types
// =========================================================

/// Comparison block data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareBlock {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub scheduled: bool,
    pub requested_hours: f64,
}

/// Comparison statistics.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareStats {
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub total_priority: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub total_hours: f64,
}

/// Scheduling change record.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingChange {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub change_type: String, // "newly_scheduled" or "newly_unscheduled"
}

/// Complete comparison dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// =========================================================
// Phase 2 Analytics (Pre-computed Summary)
// =========================================================

/// Pre-computed schedule summary from ETL tables (from schedule_summary_analytics).
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleSummary {
    pub schedule_id: i64,
    pub total_blocks: i32,
    pub scheduled_blocks: i32,
    pub unscheduled_blocks: i32,
    pub impossible_blocks: i32,
    pub scheduling_rate: f64,
    pub priority_min: Option<f64>,
    pub priority_max: Option<f64>,
    pub priority_mean: Option<f64>,
    pub priority_median: Option<f64>,
    pub priority_scheduled_mean: Option<f64>,
    pub priority_unscheduled_mean: Option<f64>,
    pub visibility_total_hours: f64,
    pub visibility_mean_hours: Option<f64>,
    pub requested_total_hours: f64,
    pub requested_mean_hours: Option<f64>,
    pub scheduled_total_hours: f64,
    pub corr_priority_visibility: Option<f64>,
    pub corr_priority_requested: Option<f64>,
    pub corr_visibility_requested: Option<f64>,
    pub conflict_count: i32,
}

/// Priority-level rate data (from schedule_priority_rates table).
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityRate {
    pub priority_value: i32,
    pub total_count: i32,
    pub scheduled_count: i32,
    pub scheduling_rate: f64,
    pub visibility_mean_hours: Option<f64>,
    pub requested_mean_hours: Option<f64>,
}

/// Visibility bin data for visibility histograms.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityBin {
    pub bin_start_unix: i64,
    pub bin_end_unix: i64,
    pub visible_count: u32,
}

/// Visibility bin data (from schedule_visibility_bins table).
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityBinData {
    pub bin_index: i32,
    pub bin_min_hours: f64,
    pub bin_max_hours: f64,
    pub bin_mid_hours: f64,
    pub total_count: i32,
    pub scheduled_count: i32,
    pub scheduling_rate: f64,
}

/// Row data for visibility histogram computations.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHistogramData {
    pub scheduling_block_id: i64,
    pub priority: i32,
    pub visibility_periods: Option<Vec<Period>>,
}

/// Heatmap bin data (from schedule_heatmap_bins table).
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapBinData {
    pub visibility_mid_hours: f64,
    pub time_mid_hours: f64,
    pub total_count: i32,
    pub scheduled_count: i32,
    pub scheduling_rate: f64,
}

// =========================================================
// Phase 3 Analytics (Visibility Time Bins)
// =========================================================

/// Visibility time series metadata (from schedule_visibility_time_metadata table).
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityTimeMetadata {
    pub schedule_id: i64,
    pub time_range_start_unix: i64,
    pub time_range_end_unix: i64,
    pub bin_duration_seconds: i32,
    pub total_bins: i32,
    pub total_blocks: i32,
    pub blocks_with_visibility: i32,
    pub priority_min: Option<f64>,
    pub priority_max: Option<f64>,
    pub max_visible_in_bin: i32,
    pub mean_visible_per_bin: Option<f64>,
}

/// Pre-computed visibility time bin data (from schedule_visibility_time_bins table).
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityTimeBin {
    pub bin_start_unix: i64,
    pub bin_end_unix: i64,
    pub bin_index: i32,
    pub total_visible_count: i32,
    pub priority_q1_count: i32,
    pub priority_q2_count: i32,
    pub priority_q3_count: i32,
    pub priority_q4_count: i32,
    pub scheduled_visible_count: i32,
    pub unscheduled_visible_count: i32,
}

// =========================================================
// Visibility Map Types
// =========================================================

/// Block summary for visibility map.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityBlockSummary {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
    pub priority: f64,
    pub num_visibility_periods: usize,
    pub scheduled: bool,
}

/// Visibility map visualization data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityMapData {
    pub blocks: Vec<VisibilityBlockSummary>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
}

// =========================================================
// Validation Types
// =========================================================

/// Validation issue.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub block_id: i64,
    pub original_block_id: Option<String>,
    pub issue_type: String,
    pub category: String,
    pub criticality: String,
    pub field_name: Option<String>,
    pub current_value: Option<String>,
    pub expected_value: Option<String>,
    pub description: String,
}
/// Validation report data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub schedule_id: i64,
    pub total_blocks: usize,
    pub valid_blocks: usize,
    pub impossible_blocks: Vec<ValidationIssue>,
    pub validation_errors: Vec<ValidationIssue>,
    pub validation_warnings: Vec<ValidationIssue>,
}

// =========================================================
// Algorithm Result Types
// =========================================================

/// Scheduling conflict detected between two blocks.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingConflict {
    pub block_id_1: String,
    pub block_id_2: String,
    pub overlap_start: f64,
    pub overlap_end: f64,
    pub overlap_duration_hours: f64,
}

#[pymethods]
impl SchedulingConflict {
    fn __repr__(&self) -> String {
        format!(
            "SchedulingConflict('{}' vs '{}', overlap={:.2}h)",
            self.block_id_1, self.block_id_2, self.overlap_duration_hours
        )
    }
}
