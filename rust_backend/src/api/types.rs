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
use serde::{Deserialize, Serialize};
use siderust::astro::ModifiedJulianDate;

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

/// Time period in Modified Julian Date (MJD) format.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    /// Start time in MJD
    pub start: ModifiedJulianDate,
    /// End time in MJD
    pub stop: ModifiedJulianDate,
}

/// Observing constraints for a scheduling block.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    /// Minimum altitude in degrees (optional)
    pub min_altitude: Option<f64>,
    /// Maximum altitude in degrees (optional)
    pub max_altitude: Option<f64>,
    /// Minimum azimuth in degrees (optional)
    pub min_azimuth: Option<f64>,
    /// Maximum azimuth in degrees (optional)
    pub max_azimuth: Option<f64>,
    /// Fixed observation time in MJD (optional)
    pub fixed_time: Option<f64>,
}

#[pymethods]
impl Constraints {
    #[new]
    #[pyo3(signature = (min_altitude=None, max_altitude=None, min_azimuth=None, max_azimuth=None, fixed_time=None))]
    pub fn new(
        min_altitude: Option<f64>,
        max_altitude: Option<f64>,
        min_azimuth: Option<f64>,
        max_azimuth: Option<f64>,
        fixed_time: Option<f64>,
    ) -> Self {
        Self {
            min_altitude,
            max_altitude,
            min_azimuth,
            max_azimuth,
            fixed_time,
        }
    }

    fn __repr__(&self) -> String {
        format!("Constraints(alt=[{:?}, {:?}], az=[{:?}, {:?}], fixed={:?})",
            self.min_altitude, self.max_altitude,
            self.min_azimuth, self.max_azimuth,
            self.fixed_time)
    }
}

/// Individual scheduling block (observation request).
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingBlock {
    /// Unique block ID
    pub id: String,
    /// Right Ascension in degrees (ICRS)
    pub ra: f64,
    /// Declination in degrees (ICRS)
    pub dec: f64,
    /// Observation priority (0-5)
    pub priority: f64,
    /// Whether block was scheduled
    pub scheduled: bool,
    /// Scheduled start time in MJD (if scheduled)
    pub scheduled_start: Option<f64>,
    /// Scheduled end time in MJD (if scheduled)
    pub scheduled_end: Option<f64>,
    /// Observing constraints
    pub constraints: Option<Constraints>,
}

#[pymethods]
impl SchedulingBlock {
    #[new]
    #[pyo3(signature = (id, ra, dec, priority, scheduled=false, scheduled_start=None, scheduled_end=None, constraints=None))]
    pub fn new(
        id: String,
        ra: f64,
        dec: f64,
        priority: f64,
        scheduled: bool,
        scheduled_start: Option<f64>,
        scheduled_end: Option<f64>,
        constraints: Option<Constraints>,
    ) -> Self {
        Self {
            id,
            ra,
            dec,
            priority,
            scheduled,
            scheduled_start,
            scheduled_end,
            constraints,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "SchedulingBlock(id='{}', ra={:.2}, dec={:.2}, priority={:.1}, scheduled={})",
            self.id, self.ra, self.dec, self.priority, self.scheduled
        )
    }
}

/// Top-level schedule with metadata and blocks.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// Schedule name
    pub name: String,
    /// List of scheduling blocks
    pub blocks: Vec<SchedulingBlock>,
    /// Dark periods (observing windows)
    pub dark_periods: Vec<Period>,
    /// Visibility periods (astronomical constraints)
    pub possible_periods: Vec<Period>,
}

#[pymethods]
impl Schedule {
    #[new]
    pub fn new(
        name: String,
        blocks: Vec<SchedulingBlock>,
        dark_periods: Vec<Period>,
        possible_periods: Vec<Period>,
    ) -> Self {
        Self {
            name,
            blocks,
            dark_periods,
            possible_periods,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Schedule(name='{}', blocks={}, dark_periods={}, possible_periods={})",
            self.name,
            self.blocks.len(),
            self.dark_periods.len(),
            self.possible_periods.len()
        )
    }
}

/// Metadata about a stored schedule.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMetadata {
    /// Database ID
    pub schedule_id: i64,
    /// Schedule name
    pub name: String,
    /// Creation timestamp (ISO format)
    pub timestamp: String,
    /// SHA256 checksum of schedule data
    pub checksum: String,
}

#[pymethods]
impl ScheduleMetadata {
    #[new]
    pub fn new(schedule_id: i64, name: String, timestamp: String, checksum: String) -> Self {
        Self {
            schedule_id,
            name,
            timestamp,
            checksum,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ScheduleMetadata(id={}, name='{}', timestamp='{}')",
            self.schedule_id, self.name, self.timestamp
        )
    }
}

/// Schedule information with block counts.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleInfo {
    pub schedule_id: i64,
    pub name: String,
    pub timestamp: String,
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
}

#[pymethods]
impl ScheduleInfo {
    fn __repr__(&self) -> String {
        format!(
            "ScheduleInfo(id={}, name='{}', blocks={}/{})",
            self.schedule_id, self.name, self.scheduled_blocks, self.total_blocks
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
    pub original_block_id: String,
    pub priority: f64,
    pub scheduled: bool,
    pub visibility_hours: f64,
    pub ra: f64,
    pub dec: f64,
}

/// Distribution statistics.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    pub mean_visibility: f64,
    pub median_visibility: f64,
    pub std_visibility: f64,
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
}

/// Complete distribution dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionData {
    pub blocks: Vec<DistributionBlock>,
    pub stats: DistributionStats,
}

// =========================================================
// Timeline Types
// =========================================================

/// Timeline block data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTimelineBlock {
    pub original_block_id: String,
    pub priority: f64,
    pub scheduled_start: f64,
    pub scheduled_end: f64,
    pub ra: f64,
    pub dec: f64,
}

/// Schedule timeline dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTimelineData {
    pub blocks: Vec<ScheduleTimelineBlock>,
}

// =========================================================
// Insights Types
// =========================================================

/// Block data for insights analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsBlock {
    pub original_block_id: String,
    pub priority: f64,
    pub scheduled: bool,
    pub visibility_hours: f64,
    pub ra: f64,
    pub dec: f64,
}

/// Analytics metrics.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMetrics {
    pub total_blocks: usize,
    pub scheduled_count: usize,
    pub mean_priority: f64,
    pub mean_visibility: f64,
}

/// Correlation entry.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationEntry {
    pub metric1: String,
    pub metric2: String,
    pub correlation: f64,
}

/// Conflict record.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub block_id_1: String,
    pub block_id_2: String,
    pub overlap_start: f64,
    pub overlap_end: f64,
}

/// Top observation entry.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopObservation {
    pub original_block_id: String,
    pub metric_value: f64,
    pub priority: f64,
    pub scheduled: bool,
}

/// Complete insights dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsData {
    pub blocks: Vec<InsightsBlock>,
    pub metrics: AnalyticsMetrics,
    pub correlations: Vec<CorrelationEntry>,
    pub conflicts: Vec<ConflictRecord>,
    pub top_by_priority: Vec<TopObservation>,
    pub top_by_visibility: Vec<TopObservation>,
}

// =========================================================
// Trends Types
// =========================================================

/// Block data for trends analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsBlock {
    pub original_block_id: String,
    pub priority: f64,
    pub scheduled: bool,
    pub visibility_hours: f64,
}

/// Empirical scheduling rate point.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpiricalRatePoint {
    pub priority: f64,
    pub rate: f64,
    pub count: usize,
}

/// Smoothed trend point.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmoothedPoint {
    pub priority: f64,
    pub rate: f64,
}

/// Heatmap bin data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapBin {
    pub priority_bin: f64,
    pub visibility_bin: f64,
    pub count: usize,
    pub scheduled_count: usize,
}

/// Trends metrics summary.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsMetrics {
    pub overall_rate: f64,
    pub priority_bins: usize,
    pub visibility_bins: usize,
}

/// Complete trends dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsData {
    pub blocks: Vec<TrendsBlock>,
    pub empirical_rates: Vec<EmpiricalRatePoint>,
    pub smoothed_trend: Vec<SmoothedPoint>,
    pub heatmap: Vec<HeatmapBin>,
    pub metrics: TrendsMetrics,
}

// =========================================================
// Comparison Types
// =========================================================

/// Comparison block data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareBlock {
    pub original_block_id: String,
    pub priority: f64,
    pub scheduled_a: bool,
    pub scheduled_b: bool,
    pub ra: f64,
    pub dec: f64,
}

/// Comparison statistics.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareStats {
    pub total_blocks: usize,
    pub both_scheduled: usize,
    pub only_a: usize,
    pub only_b: usize,
    pub neither: usize,
}

/// Scheduling change record.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingChange {
    pub original_block_id: String,
    pub change_type: String, // "added", "removed", "unchanged"
    pub priority: f64,
}

/// Complete comparison dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareData {
    pub blocks: Vec<CompareBlock>,
    pub stats: CompareStats,
    pub changes: Vec<SchedulingChange>,
}

// =========================================================
// Phase 2 Analytics (Pre-computed Summary)
// =========================================================

/// Pre-computed schedule summary from ETL tables.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleSummary {
    pub schedule_id: i64,
    pub total_blocks: i32,
    pub scheduled_blocks: i32,
    pub scheduling_rate: f64,
    pub mean_priority: f64,
    pub mean_visibility_hours: f64,
    pub total_visibility_hours: f64,
}

/// Priority-based scheduling rate.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityRate {
    pub priority_bin: f64,
    pub total_count: i32,
    pub scheduled_count: i32,
    pub rate: f64,
}

/// Visibility histogram bin.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityBin {
    pub visibility_bin: f64,
    pub count: i32,
}

/// Heatmap bin data (pre-computed).
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapBinData {
    pub priority_bin: f64,
    pub visibility_bin: f64,
    pub count: i32,
}

// =========================================================
// Phase 3 Analytics (Visibility Time Bins)
// =========================================================

/// Visibility time metadata.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityTimeMetadata {
    pub schedule_id: i64,
    pub min_mjd: f64,
    pub max_mjd: f64,
    pub bin_size_days: f64,
    pub total_bins: i32,
}

/// Pre-computed visibility time bin.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityTimeBin {
    pub bin_start_mjd: f64,
    pub bin_end_mjd: f64,
    pub visibility_count: i32,
}

// =========================================================
// Visibility Map Types
// =========================================================

/// Block summary for visibility map.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityBlockSummary {
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
