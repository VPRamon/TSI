//! Public Python API surface for the Rust backend.
//!
//! This file consolidates the Python-facing DTO types and registration
//! functions that were previously split across `src/api/*`.
//!
//! It exposes:
//! - `pub mod types` containing `#[pyclass]` DTOs
//! - `register_api_functions` and `register_transformation_functions`
//!
pub use crate::routes::compare::CompareBlock;
pub use crate::routes::compare::CompareData;
pub use crate::routes::compare::CompareStats;
pub use crate::routes::compare::SchedulingChange;
pub use crate::routes::distribution::DistributionBlock;
pub use crate::routes::distribution::DistributionData;
pub use crate::routes::distribution::DistributionStats;
pub use crate::routes::insights::AnalyticsMetrics;
pub use crate::routes::insights::ConflictRecord;
pub use crate::routes::insights::CorrelationEntry;
pub use crate::routes::insights::InsightsBlock;
pub use crate::routes::insights::InsightsData;
pub use crate::routes::insights::TopObservation;
pub use crate::routes::landing::ScheduleInfo;
pub use crate::routes::skymap::LightweightBlock;
pub use crate::routes::skymap::PriorityBinInfo;
pub use crate::routes::skymap::SkyMapData;
pub use crate::routes::timeline::ScheduleTimelineBlock;
pub use crate::routes::timeline::ScheduleTimelineData;
pub use crate::routes::trends::EmpiricalRatePoint;
pub use crate::routes::trends::HeatmapBin;
pub use crate::routes::trends::SmoothedPoint;
pub use crate::routes::trends::TrendsBlock;
pub use crate::routes::trends::TrendsData;
pub use crate::routes::trends::TrendsMetrics;
pub use crate::routes::validation::ValidationIssue;
pub use crate::routes::validation::ValidationReport;
pub use crate::routes::visibility::VisibilityBlockSummary;
pub use crate::routes::visibility::VisibilityMapData;
use pyo3::prelude::*;

#[pyo3::pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct ScheduleId(pub i64);

#[pyo3::pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TargetId(pub i64);

#[pyo3::pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ConstraintsId(pub i64);

#[pyo3::pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SchedulingBlockId(pub i64);

#[pymethods]
impl ScheduleId {
    #[new]
    pub fn new(value: i64) -> Self {
        ScheduleId(value)
    }

    #[getter]
    pub fn value(&self) -> i64 {
        self.0
    }
}

#[pymethods]
impl TargetId {
    #[new]
    pub fn new(value: i64) -> Self {
        TargetId(value)
    }

    #[getter]
    pub fn value(&self) -> i64 {
        self.0
    }
}

#[pymethods]
impl ConstraintsId {
    #[new]
    pub fn new(value: i64) -> Self {
        ConstraintsId(value)
    }

    #[getter]
    pub fn value(&self) -> i64 {
        self.0
    }
}

#[pymethods]
impl SchedulingBlockId {
    #[new]
    pub fn new(value: i64) -> Self {
        SchedulingBlockId(value)
    }

    #[getter]
    pub fn value(&self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for ScheduleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for TargetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for ConstraintsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for SchedulingBlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ScheduleId> for i64 {
    fn from(id: ScheduleId) -> Self {
        id.0
    }
}
// Python-facing Data Transfer Objects (DTOs) moved to the api root.
use pyo3::types::PyTuple;
use serde::{Deserialize, Serialize};

pub use crate::models::ModifiedJulianDate;

/// Time period in Modified Julian Date (MJD) format.
#[pyclass(module = "tsi_rust_api")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    /// Start time in MJD
    pub start: ModifiedJulianDate,
    /// End time in MJD
    pub stop: ModifiedJulianDate,
}

#[pymethods]
impl Period {
    #[new]
    pub fn py_new(start: f64, stop: f64) -> Self {
        Self {
            start: ModifiedJulianDate::new(start),
            stop: ModifiedJulianDate::new(stop),
        }
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
                start: ModifiedJulianDate::new(start_mjd),
                stop: ModifiedJulianDate::new(stop_mjd),
            })
        })
    }

    #[getter]
    pub fn start_mjd(&self) -> f64 {
        self.start.value()
    }

    #[getter]
    pub fn stop_mjd(&self) -> f64 {
        self.stop.value()
    }

    pub fn contains_mjd(&self, mjd: f64) -> bool {
        let min_mjd = self.start.value().min(self.stop.value());
        let max_mjd = self.start.value().max(self.stop.value());
        mjd >= min_mjd && mjd <= max_mjd
    }

    pub fn to_datetime<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        // Convert MJD -> seconds since UNIX epoch then use Python's datetime
        let s_secs = (self.start.value() - 40587.0) * 86400.0;
        let e_secs = (self.stop.value() - 40587.0) * 86400.0;

        let datetime_mod = py.import("datetime")?;
        let datetime_cls = datetime_mod.getattr("datetime")?;
        let timezone_utc = datetime_mod.getattr("timezone")?.getattr("utc")?;

        let s_dt = datetime_cls.call_method1("fromtimestamp", (s_secs, timezone_utc.clone()))?;
        let e_dt = datetime_cls.call_method1("fromtimestamp", (e_secs, timezone_utc))?;

        PyTuple::new(py, [s_dt, e_dt])
    }
}

impl Period {
    pub fn new(start: ModifiedJulianDate, stop: ModifiedJulianDate) -> Option<Self> {
        if start.value() < stop.value() {
            Some(Self { start, stop })
        } else {
            None
        }
    }

    /// Length of the interval in days.
    pub fn duration(&self) -> qtty::Days {
        qtty::Days::new(self.stop.value() - self.start.value())
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

/// Observing constraints for a scheduling block.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    /// Minimum altitude in degrees
    #[serde(with = "qtty::serde_f64")]
    pub min_alt: qtty::Degrees,
    /// Maximum altitude in degrees
    #[serde(with = "qtty::serde_f64")]
    pub max_alt: qtty::Degrees,
    /// Minimum azimuth in degrees
    #[serde(with = "qtty::serde_f64")]
    pub min_az: qtty::Degrees,
    /// Maximum azimuth in degrees
    #[serde(with = "qtty::serde_f64")]
    pub max_az: qtty::Degrees,
    /// Fixed observation time window in MJD
    pub fixed_time: Option<Period>,
}

#[pymethods]
impl Constraints {
    #[new]
    #[pyo3(signature = (min_alt, max_alt, min_az, max_az, fixed_time=None))]
    pub fn new(
        min_alt: qtty::Degrees,
        max_alt: qtty::Degrees,
        min_az: qtty::Degrees,
        max_az: qtty::Degrees,
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
            self.min_alt.value(), self.max_alt.value(), self.min_az.value(), self.max_az.value(), self.fixed_time
        )
    }
}

/// Individual scheduling block (observation request).
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingBlock {
    /// Database ID for the block
    pub id: SchedulingBlockId,
    /// Original ID from JSON (shown to user)
    pub original_block_id: Option<String>,
    /// Right Ascension in degrees (ICRS)
    #[serde(with = "qtty::serde_f64")]
    pub target_ra: qtty::Degrees,
    /// Declination in degrees (ICRS)
    #[serde(with = "qtty::serde_f64")]
    pub target_dec: qtty::Degrees,
    /// Observing constraints
    pub constraints: Constraints,
    /// Observation priority
    pub priority: f64,
    /// Minimum observation duration in seconds
    #[serde(with = "qtty::serde_f64")]
    pub min_observation: qtty::Seconds,
    /// Requested observation duration in seconds
    #[serde(with = "qtty::serde_f64")]
    pub requested_duration: qtty::Seconds,
    /// Visibility periods in MJD
    #[serde(default)]
    pub visibility_periods: Vec<Period>,
    /// Scheduled time window in MJD (if scheduled)
    pub scheduled_period: Option<Period>,
}

#[pymethods]
impl SchedulingBlock {
    #[new]
    #[pyo3(signature = (id, original_block_id, target_ra, target_dec, constraints, priority, min_observation, requested_duration, visibility_periods=None, scheduled_period=None))]
    pub fn new(
        id: SchedulingBlockId,
        original_block_id: Option<String>,
        target_ra: qtty::Degrees,
        target_dec: qtty::Degrees,
        constraints: Constraints,
        priority: f64,
        min_observation: qtty::Seconds,
        requested_duration: qtty::Seconds,
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
            self.id.0, self.target_ra.value(), self.target_dec.value(), self.priority
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
    #[serde(default)]
    pub name: String,
    /// SHA256 checksum of schedule data
    #[serde(default)]
    pub checksum: String,
    /// Dark periods (observing windows)
    #[serde(default)]
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

/// Register all API functions with the Python module.
pub fn register_api_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Route-specific functions, classes and constants are registered centrally by `routes`
    crate::routes::register_route_functions(m)?;

    // Register all API classes
    m.add_class::<ModifiedJulianDate>()?;
    m.add_class::<Period>()?;
    m.add_class::<Constraints>()?;
    m.add_class::<SchedulingBlock>()?;
    m.add_class::<Schedule>()?;

    m.add_class::<ScheduleId>()?;
    m.add_class::<TargetId>()?;
    m.add_class::<ConstraintsId>()?;
    m.add_class::<SchedulingBlockId>()?;

    Ok(())
}

/// Register transformation functions (no-op for now).
///
/// Historically transformation helpers were registered separately; keep a
/// stub here to preserve the public API expected by `lib.rs`.
pub fn register_transformation_functions(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    // No transformation functions to register in this consolidated API file.
    Ok(())
}

// Re-export types at the `api` root so other crates can refer to
// `crate::api::Period` instead of `crate::api::types::Period`.
// Types now live in this module; no re-export needed.
