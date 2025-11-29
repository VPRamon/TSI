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
    pub target: ICRS,
    pub constraints: Constraints,
    pub priority: f32, // NUMERIC(4,1) as f32
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
        priority: f32,
        min_observation_seconds: f64,
        requested_duration_seconds: f64,
        visibility_periods: Vec<Period>,
        scheduled_period: Option<Period>,
    ) -> Self {
        Self {
            id,
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
    pub fn priority(&self) -> f32 {
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
