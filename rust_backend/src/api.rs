//! Public Python API surface for the Rust backend.
//!
//! This file consolidates the Python-facing DTO types and registration
//! functions that were previously split across `src/api/*`.
//!
//! It exposes:
//! - `pub mod types` containing `#[pyclass]` DTOs
//! - `register_api_functions` and `register_transformation_functions`
//!
pub use crate::models::*;
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
