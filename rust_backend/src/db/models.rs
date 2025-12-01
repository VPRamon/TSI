/// Core domain model for the scheduling system.
/// This is an in-memory representation of the "schedule concept"
/// from the database schema: schedules, dark periods, scheduling blocks,
/// and their assignments (schedule_scheduling_blocks).
use chrono::{DateTime, Utc};
use pyo3::{exceptions::PyValueError, prelude::*};
use siderust::{
    astro::ModifiedJulianDate,
    coordinates::spherical::direction::ICRS,
    units::angular::Degrees,
    units::time::{Days, Seconds},
};

macro_rules! py_id_type {
    ($(#[$meta:meta])* $name:ident, $desc:literal) => {
        $(#[$meta])*
        #[pyclass(module = "tsi_rust")]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct $name(pub i64);

        #[pymethods]
        impl $name {
            #[new]
            pub fn new(value: i64) -> Self {
                Self(value)
            }

            #[getter]
            pub fn value(&self) -> i64 {
                self.0
            }

            fn __repr__(&self) -> String {
                format!("{}({})", $desc, self.0)
            }
        }
    };
}

py_id_type!(
    /// Strongly-typed identifier for a schedule record (maps to BIGINT).
    ScheduleId,
    "ScheduleId"
);
py_id_type!(
    /// Strongly-typed identifier for a target record.
    TargetId,
    "TargetId"
);
py_id_type!(
    /// Strongly-typed identifier for a constraints record.
    ConstraintsId,
    "ConstraintsId"
);
py_id_type!(
    /// Strongly-typed identifier for a scheduling block.
    SchedulingBlockId,
    "SchedulingBlockId"
);

/// Simple representation of a time window in Modified Julian Date.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Period {
    pub start: ModifiedJulianDate,
    pub stop: ModifiedJulianDate,
}

impl Period {
    pub fn new(start: ModifiedJulianDate, stop: ModifiedJulianDate) -> Option<Self> {
        if start.value() < stop.value() {
            Some(Self { start, stop })
        } else {
            None
        }
    }

    /// Length of the interval in seconds (like your computed column).
    pub fn duration(&self) -> Days {
        Days::new(self.stop.value() - self.start.value())
    }

    /// Check if a given MJD instant lies inside this interval (inclusive start, exclusive end).
    pub fn contains(&self, t_mjd: ModifiedJulianDate) -> bool {
        self.start.value() <= t_mjd.value() && t_mjd.value() < self.stop.value()
    }

    /// Check if this interval overlaps with another.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start.value() < other.stop.value() && other.start.value() < self.stop.value()
    }
}

#[pymethods]
impl Period {
    #[new]
    pub fn py_new(start_mjd: f64, stop_mjd: f64) -> PyResult<Self> {
        Period::new(
            ModifiedJulianDate::new(start_mjd),
            ModifiedJulianDate::new(stop_mjd),
        )
        .ok_or_else(|| PyValueError::new_err("start must be before stop"))
    }

    #[getter]
    pub fn start_mjd(&self) -> f64 {
        self.start.value()
    }

    #[getter]
    pub fn stop_mjd(&self) -> f64 {
        self.stop.value()
    }

    pub fn duration_days(&self) -> f64 {
        self.stop.value() - self.start.value()
    }

    pub fn contains_mjd(&self, mjd: f64) -> bool {
        self.contains(ModifiedJulianDate::new(mjd))
    }

    pub fn overlaps_period(&self, other: &Period) -> bool {
        self.overlaps(other)
    }

    fn __repr__(&self) -> String {
        format!(
            "Period(start={:.5}, stop={:.5})",
            self.start.value(),
            self.stop.value()
        )
    }
}

#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct Constraints {
    pub min_alt: Degrees,
    pub max_alt: Degrees,
    pub min_az: Degrees,
    pub max_az: Degrees,
    pub fixed_time: Option<Period>,
}

#[pymethods]
impl Constraints {
    #[new]
    #[pyo3(signature = (min_alt_deg, max_alt_deg, min_az_deg, max_az_deg, fixed_time=None))]
    pub fn py_new(
        min_alt_deg: f64,
        max_alt_deg: f64,
        min_az_deg: f64,
        max_az_deg: f64,
        fixed_time: Option<Period>,
    ) -> Self {
        Self {
            min_alt: Degrees::new(min_alt_deg),
            max_alt: Degrees::new(max_alt_deg),
            min_az: Degrees::new(min_az_deg),
            max_az: Degrees::new(max_az_deg),
            fixed_time,
        }
    }

    #[getter]
    pub fn min_alt_deg(&self) -> f64 {
        self.min_alt.value()
    }

    #[getter]
    pub fn max_alt_deg(&self) -> f64 {
        self.max_alt.value()
    }

    #[getter]
    pub fn min_az_deg(&self) -> f64 {
        self.min_az.value()
    }

    #[getter]
    pub fn max_az_deg(&self) -> f64 {
        self.max_az.value()
    }

    #[getter]
    pub fn fixed_time(&self) -> Option<Period> {
        self.fixed_time
    }

    fn __repr__(&self) -> String {
        let fixed = self
            .fixed_time
            .map(|p| format!("[{:.3}, {:.3}]", p.start.value(), p.stop.value()));
        format!(
            "Constraints(alt=({:.1}, {:.1}), az=({:.1}, {:.1}), fixed_time={:?})",
            self.min_alt.value(),
            self.max_alt.value(),
            self.min_az.value(),
            self.max_az.value(),
            fixed
        )
    }
}

/// Atomic observing request (mirrors scheduling_blocks).
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct SchedulingBlock {
    pub id: SchedulingBlockId,
    pub original_block_id: Option<String>,  // Original schedulingBlockId from JSON
    pub target: ICRS,
    pub constraints: Constraints,
    pub priority: f64,
    pub min_observation: Seconds,
    pub requested_duration: Seconds,
    pub visibility_periods: Vec<Period>,
    pub scheduled_period: Option<Period>,
}

#[pymethods]
impl SchedulingBlock {
    #[new]
    #[pyo3(signature = (id, ra_deg, dec_deg, constraints, priority, min_observation_seconds, requested_duration_seconds, visibility_periods, scheduled_period=None))]
    pub fn py_new(
        id: SchedulingBlockId,
        ra_deg: f64,
        dec_deg: f64,
        constraints: Constraints,
        priority: f64,
        min_observation_seconds: f64,
        requested_duration_seconds: f64,
        visibility_periods: Vec<Period>,
        scheduled_period: Option<Period>,
    ) -> Self {
        Self {
            id,
            original_block_id: None,  // Not used from Python
            target: ICRS::new(Degrees::new(ra_deg), Degrees::new(dec_deg)),
            constraints,
            priority,
            min_observation: Seconds::new(min_observation_seconds),
            requested_duration: Seconds::new(requested_duration_seconds),
            visibility_periods,
            scheduled_period,
        }
    }

    #[getter]
    pub fn id(&self) -> SchedulingBlockId {
        self.id
    }

    #[getter]
    pub fn target_ra_deg(&self) -> f64 {
        self.target.ra().value()
    }

    #[getter]
    pub fn target_dec_deg(&self) -> f64 {
        self.target.dec().value()
    }

    #[getter]
    pub fn constraints(&self) -> Constraints {
        self.constraints.clone()
    }

    #[getter]
    pub fn priority(&self) -> f64 {
        self.priority
    }

    #[getter]
    pub fn min_observation_seconds(&self) -> f64 {
        self.min_observation.value()
    }

    #[getter]
    pub fn requested_duration_seconds(&self) -> f64 {
        self.requested_duration.value()
    }

    #[getter]
    pub fn visibility_periods(&self) -> Vec<Period> {
        self.visibility_periods.clone()
    }

    #[getter]
    pub fn scheduled_period(&self) -> Option<Period> {
        self.scheduled_period
    }

    pub fn target_coordinates(&self) -> (f64, f64) {
        (self.target.ra().value(), self.target.dec().value())
    }

    fn __repr__(&self) -> String {
        format!(
            "SchedulingBlock(id={}, priority={:.1}, target=({:.3}, {:.3}))",
            self.id.0,
            self.priority,
            self.target.ra().value(),
            self.target.dec().value()
        )
    }
}

/// Core "Schedule" concept:
/// - Metadata (name, checksum, etc.)
/// - Dark periods
/// - Assigned scheduling blocks with optional execution windows
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct Schedule {
    pub id: Option<ScheduleId>,
    pub name: String,
    pub checksum: String,
    pub dark_periods: Vec<Period>,
    pub blocks: Vec<SchedulingBlock>,
}

#[pymethods]
impl Schedule {
    #[new]
    pub fn py_new(
        id: Option<ScheduleId>,
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

    #[getter]
    pub fn id(&self) -> Option<ScheduleId> {
        self.id
    }

    #[getter]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[getter]
    pub fn checksum(&self) -> String {
        self.checksum.clone()
    }

    #[getter]
    pub fn dark_periods(&self) -> Vec<Period> {
        self.dark_periods.clone()
    }

    #[getter]
    pub fn blocks(&self) -> Vec<SchedulingBlock> {
        self.blocks.clone()
    }

    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub fn dark_period_count(&self) -> usize {
        self.dark_periods.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "Schedule(name={}, blocks={}, dark_periods={})",
            self.name,
            self.blocks.len(),
            self.dark_periods.len()
        )
    }
}

/// Lightweight metadata about a schedule (for listings).
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct ScheduleMetadata {
    pub schedule_id: Option<i64>,
    pub schedule_name: String,
    pub upload_timestamp: DateTime<Utc>,
    pub checksum: String,
}

#[pymethods]
impl ScheduleMetadata {
    #[getter]
    pub fn schedule_id(&self) -> Option<i64> {
        self.schedule_id
    }

    #[getter]
    pub fn schedule_name(&self) -> String {
        self.schedule_name.clone()
    }

    pub fn upload_timestamp_iso(&self) -> String {
        self.upload_timestamp.to_rfc3339()
    }

    #[getter]
    pub fn checksum(&self) -> String {
        self.checksum.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "ScheduleMetadata(id={:?}, name={}, uploaded={})",
            self.schedule_id,
            self.schedule_name,
            self.upload_timestamp.to_rfc3339(),
        )
    }
}

/// Extended schedule information including stats.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct ScheduleInfo {
    pub metadata: ScheduleMetadata,
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
    pub unscheduled_blocks: usize,
}

#[pymethods]
impl ScheduleInfo {
    #[getter]
    pub fn metadata(&self) -> ScheduleMetadata {
        self.metadata.clone()
    }

    #[getter]
    pub fn total_blocks(&self) -> usize {
        self.total_blocks
    }

    #[getter]
    pub fn scheduled_blocks(&self) -> usize {
        self.scheduled_blocks
    }

    #[getter]
    pub fn unscheduled_blocks(&self) -> usize {
        self.unscheduled_blocks
    }

    fn __repr__(&self) -> String {
        format!(
            "ScheduleInfo(total={}, scheduled={}, unscheduled={})",
            self.total_blocks, self.scheduled_blocks, self.unscheduled_blocks
        )
    }
}

/// Lightweight scheduling block for sky map visualization.
/// Contains only the essential fields needed for plotting.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct LightweightBlock {
    pub id: SchedulingBlockId,
    pub priority: f64,
    pub priority_bin: String,
    pub requested_duration_seconds: f64,
    pub target_ra_deg: f64,
    pub target_dec_deg: f64,
    pub scheduled_period: Option<Period>,
}

#[pymethods]
impl LightweightBlock {
    #[getter]
    pub fn id(&self) -> SchedulingBlockId {
        self.id
    }

    #[getter]
    pub fn priority(&self) -> f64 {
        self.priority
    }

    #[getter]
    pub fn priority_bin(&self) -> String {
        self.priority_bin.clone()
    }

    #[getter]
    pub fn requested_duration_seconds(&self) -> f64 {
        self.requested_duration_seconds
    }

    #[getter]
    pub fn target_ra_deg(&self) -> f64 {
        self.target_ra_deg
    }

    #[getter]
    pub fn target_dec_deg(&self) -> f64 {
        self.target_dec_deg
    }

    #[getter]
    pub fn scheduled_period(&self) -> Option<Period> {
        self.scheduled_period
    }

    fn __repr__(&self) -> String {
        format!(
            "LightweightBlock(id={}, priority={:.2}, ra={:.2}, dec={:.2}, scheduled={})",
            self.id.0,
            self.priority,
            self.target_ra_deg,
            self.target_dec_deg,
            self.scheduled_period.is_some()
        )
    }
}

/// Computed priority bin with range information.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct PriorityBinInfo {
    pub label: String,
    pub min_priority: f64,
    pub max_priority: f64,
    pub color: String,
}

#[pymethods]
impl PriorityBinInfo {
    #[getter]
    pub fn label(&self) -> String {
        self.label.clone()
    }

    #[getter]
    pub fn min_priority(&self) -> f64 {
        self.min_priority
    }

    #[getter]
    pub fn max_priority(&self) -> f64 {
        self.max_priority
    }

    #[getter]
    pub fn color(&self) -> String {
        self.color.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "PriorityBinInfo(label='{}', range=[{:.2}, {:.2}], color='{}')",
            self.label, self.min_priority, self.max_priority, self.color
        )
    }
}

/// Complete sky map data with blocks and computed metadata.
/// This structure contains everything the frontend needs to render the sky map.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
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

#[pymethods]
impl SkyMapData {
    #[getter]
    pub fn blocks(&self) -> Vec<LightweightBlock> {
        self.blocks.clone()
    }

    #[getter]
    pub fn priority_bins(&self) -> Vec<PriorityBinInfo> {
        self.priority_bins.clone()
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
    pub fn ra_min(&self) -> f64 {
        self.ra_min
    }

    #[getter]
    pub fn ra_max(&self) -> f64 {
        self.ra_max
    }

    #[getter]
    pub fn dec_min(&self) -> f64 {
        self.dec_min
    }

    #[getter]
    pub fn dec_max(&self) -> f64 {
        self.dec_max
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
    pub fn scheduled_time_min(&self) -> Option<f64> {
        self.scheduled_time_min
    }

    #[getter]
    pub fn scheduled_time_max(&self) -> Option<f64> {
        self.scheduled_time_max
    }

    fn __repr__(&self) -> String {
        format!(
            "SkyMapData(blocks={}, bins={}, priority=[{:.2}, {:.2}], scheduled={}/{})",
            self.total_count,
            self.priority_bins.len(),
            self.priority_min,
            self.priority_max,
            self.scheduled_count,
            self.total_count
        )
    }
}

/// Lightweight scheduling block for distribution visualizations.
/// Contains only the fields needed for statistical plots and histograms.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct DistributionBlock {
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub elevation_range_deg: f64,
    pub scheduled: bool,
}

#[pymethods]
impl DistributionBlock {
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

    fn __repr__(&self) -> String {
        format!(
            "DistributionBlock(priority={:.2}, visibility={:.1}h, requested={:.1}h, elevation={:.1}°, scheduled={})",
            self.priority,
            self.total_visibility_hours,
            self.requested_hours,
            self.elevation_range_deg,
            self.scheduled
        )
    }
}

/// Statistical summary for a group of distribution blocks.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct DistributionStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

#[pymethods]
impl DistributionStats {
    #[getter]
    pub fn count(&self) -> usize {
        self.count
    }

    #[getter]
    pub fn mean(&self) -> f64 {
        self.mean
    }

    #[getter]
    pub fn median(&self) -> f64 {
        self.median
    }

    #[getter]
    pub fn std_dev(&self) -> f64 {
        self.std_dev
    }

    #[getter]
    pub fn min(&self) -> f64 {
        self.min
    }

    #[getter]
    pub fn max(&self) -> f64 {
        self.max
    }

    #[getter]
    pub fn sum(&self) -> f64 {
        self.sum
    }

    fn __repr__(&self) -> String {
        format!(
            "DistributionStats(count={}, mean={:.2}, median={:.2}, std={:.2})",
            self.count, self.mean, self.median, self.std_dev
        )
    }
}

/// Complete distribution data with blocks and computed statistics.
/// This structure contains everything the frontend needs for distribution visualizations.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
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

#[pymethods]
impl DistributionData {
    #[getter]
    pub fn blocks(&self) -> Vec<DistributionBlock> {
        self.blocks.clone()
    }

    #[getter]
    pub fn priority_stats(&self) -> DistributionStats {
        self.priority_stats.clone()
    }

    #[getter]
    pub fn visibility_stats(&self) -> DistributionStats {
        self.visibility_stats.clone()
    }

    #[getter]
    pub fn requested_hours_stats(&self) -> DistributionStats {
        self.requested_hours_stats.clone()
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
    pub fn unscheduled_count(&self) -> usize {
        self.unscheduled_count
    }

    #[getter]
    pub fn impossible_count(&self) -> usize {
        self.impossible_count
    }

    fn __repr__(&self) -> String {
        format!(
            "DistributionData(total={}, scheduled={}, impossible={})",
            self.total_count, self.scheduled_count, self.impossible_count
        )
    }
}

// =========================================================
// Visibility Histogram Types
// =========================================================

/// Lightweight block summary for the visibility map.
/// Provides just enough information for filtering and statistics.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct VisibilityBlockSummary {
    pub scheduling_block_id: i64,
    pub priority: f64,
    pub num_visibility_periods: usize,
    pub scheduled: bool,
}

#[pymethods]
impl VisibilityBlockSummary {
    #[getter]
    pub fn scheduling_block_id(&self) -> i64 {
        self.scheduling_block_id
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
            self.scheduling_block_id, self.priority, self.num_visibility_periods, self.scheduled
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
    pub scheduling_block_id: i64,
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
    pub fn scheduling_block_id(&self) -> i64 {
        self.scheduling_block_id
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
            self.scheduling_block_id, self.priority, self.scheduled_start_mjd, self.scheduled_stop_mjd
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
    pub scheduling_block_id: i64,
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
    pub fn scheduling_block_id(&self) -> i64 {
        self.scheduling_block_id
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
            self.scheduling_block_id, self.priority, self.scheduled
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
    pub block_id_1: i64,
    pub block_id_2: i64,
    pub start_time_1: f64,
    pub stop_time_1: f64,
    pub start_time_2: f64,
    pub stop_time_2: f64,
    pub overlap_hours: f64,
}

#[pymethods]
impl ConflictRecord {
    #[getter]
    pub fn block_id_1(&self) -> i64 {
        self.block_id_1
    }

    #[getter]
    pub fn block_id_2(&self) -> i64 {
        self.block_id_2
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
    pub scheduling_block_id: i64,
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub scheduled: bool,
}

#[pymethods]
impl TopObservation {
    #[getter]
    pub fn scheduling_block_id(&self) -> i64 {
        self.scheduling_block_id
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
            self.scheduling_block_id, self.priority, self.total_visibility_hours
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
    pub scheduling_block_id: i64,
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub scheduled: bool,
}

#[pymethods]
impl TrendsBlock {
    #[getter]
    pub fn scheduling_block_id(&self) -> i64 {
        self.scheduling_block_id
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
            self.scheduling_block_id, self.priority, self.scheduled
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
