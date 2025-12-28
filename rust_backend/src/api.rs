//! Public Python API surface for the Rust backend.
//!
//! This file consolidates the Python-facing DTO types and registration
//! functions that were previously split across `src/api/*`.
//!
//! It exposes:
//! - `pub mod types` containing `#[pyclass]` DTOs
//! - `register_api_functions` and `register_transformation_functions`
//!
use pyo3::prelude::*;
pub use crate::routes::landing::ScheduleInfo;
pub use crate::routes::skymap::PriorityBinInfo;
pub use crate::routes::skymap::LightweightBlock;
pub use crate::routes::skymap::SkyMapData;
pub use crate::routes::visibility::VisibilityBlockSummary;
pub use crate::routes::visibility::VisibilityMapData;
pub use crate::routes::distribution::DistributionBlock;
pub use crate::routes::distribution::DistributionStats;
pub use crate::routes::distribution::DistributionData;
pub use crate::routes::timeline::ScheduleTimelineBlock;
pub use crate::routes::timeline::ScheduleTimelineData;
pub use crate::routes::insights::InsightsBlock;
pub use crate::routes::insights::AnalyticsMetrics;
pub use crate::routes::insights::CorrelationEntry;
pub use crate::routes::insights::ConflictRecord;
pub use crate::routes::insights::TopObservation;
pub use crate::routes::insights::InsightsData;
pub use crate::routes::trends::TrendsBlock;
pub use crate::routes::trends::EmpiricalRatePoint;
pub use crate::routes::trends::SmoothedPoint;
pub use crate::routes::trends::HeatmapBin;
pub use crate::routes::trends::TrendsMetrics;
pub use crate::routes::trends::TrendsData;
pub use crate::routes::compare::CompareBlock;
pub use crate::routes::compare::CompareStats;
pub use crate::routes::compare::SchedulingChange;
pub use crate::routes::compare::CompareData;
pub use crate::routes::validation::ValidationIssue;
pub use crate::routes::validation::ValidationReport;
pub use crate::models::*;

#[pyo3::pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TargetId(pub i64);

#[pyo3::pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ConstraintsId(pub i64);

#[pyo3::pyclass(module = "tsi_rust_api")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SchedulingBlockId(pub i64);
// Python-facing Data Transfer Objects (DTOs) moved to the api root.
use serde::{Deserialize, Serialize};


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
    pub id: Option<ScheduleId>,
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
