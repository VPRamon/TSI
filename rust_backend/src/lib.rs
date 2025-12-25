//! # TSI Rust Backend
//!
//! High-performance telescope scheduling analysis engine with Python bindings.
//!
//! This crate provides a Rust-based backend for the Telescope Scheduling Intelligence (TSI)
//! system, offering efficient parsing, preprocessing, validation, and analysis of astronomical
//! observation schedules. The core functionality is exposed to Python via PyO3 bindings.
//!
//! ## Features
//!
//! - **Data Loading**: Parse observation schedules from JSON format
//! - **Preprocessing**: Validate, enrich, and transform scheduling data
//! - **Analysis**: Compute metrics, correlations, and identify scheduling conflicts
//! - **Optimization**: Greedy scheduling algorithms for observation planning
//! - **Time Handling**: Modified Julian Date (MJD) conversions and time period management
//! - **Visibility Computation**: Integration with visibility period data
//!
//! ## Architecture
//!
//! The crate is organized into several logical modules:
//!
//! - [`algorithms`]: Analytics, conflict detection, and optimization routines
//! - [`api`]: PyO3 bindings exposing Rust functionality to Python (DTOs, conversions, exports)
//! - [`db`]: Database operations, repository pattern, and persistence layer
//! - [`services`]: High-level business logic and visualization services
//! - [`transformations`]: Data transformation utilities
//!
//! ## Python API Example
//!
//! ```python
//! import tsi_rust
//!
//! # Initialize database connection
//! tsi_rust.py_init_database()
//!
//! # Store schedule and populate analytics
//! metadata = tsi_rust.py_store_schedule(
//!     schedule_json,
//!     possible_periods_json,
//!     dark_periods_json,
//!     "My Schedule",
//!     populate_analytics=True,
//!     skip_time_bins=False
//! )
//!
//! # Get pre-computed analytics summary
//! summary = tsi_rust.py_get_schedule_summary(metadata.schedule_id)
//! print(f"Scheduling rate: {summary.scheduling_rate:.2%}")
//! ```
//!
//! ## Performance
//!
//! This Rust backend is designed for high-performance batch processing of large
//! observation schedules. Key optimizations include:
//!
//! - Zero-copy parsing where possible
//! - Efficient JSON-based data processing with serde_json
//! - Parallel batch operations
//! - Minimal allocations in hot paths

use pyo3::prelude::*;

pub mod routes;
pub mod algorithms;
pub mod api;
pub mod db;
pub mod services;
pub mod transformations;

pub mod api_tmp {
    pub use crate::routes::landing::ScheduleInfo;
    pub use crate::routes::skymap::SkyMapData;
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

    #[pyo3::pyclass(module = "tsi_rust_api")]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    pub struct ScheduleId(pub i64);

    /// Strongly-typed identifier for a target record.
    #[pyo3::pyclass(module = "tsi_rust_api")]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    pub struct TargetId(pub i64);

}

/// Python module entry point for the new TSI Rust API.
///
/// This is the new stable API module that isolates Python bindings from internal implementations.
/// All PyO3 types and conversions are handled in the `api` module, allowing internal models
/// to evolve independently.
///
/// # Module Contents
///
/// - Time conversion utilities
/// - Database operations (store, retrieve, analytics)
/// - Visualization data queries (sky map, distributions, timeline, insights, trends, compare)
/// - Algorithm operations (conflict detection, top observations)
/// - Validation reports
///
/// # Usage
///
/// ```python
/// import tsi_rust_api
///
/// # Initialize database
/// tsi_rust_api.init_database()
///
/// # Store schedule
/// metadata = tsi_rust_api.store_schedule(
///     schedule_json, possible_periods_json, dark_periods_json,
///     "My Schedule", populate_analytics=True
/// )
///
/// # Get visualization data
/// sky_map = tsi_rust_api.get_sky_map_data(metadata.schedule_id)
/// ```
#[pymodule]
fn tsi_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Re-export all functions from the API module
    api::register_api_functions(m)?;

    // Add transformation functions that Python code expects
    api::register_transformation_functions(m)?;

    m.add_class::<api_tmp::ScheduleId>()?;
    m.add_class::<api_tmp::TargetId>()?;

    Ok(())
}

/// Python module: tsi_rust_api
///
/// **NEW API** - This is the new high-level API module.
/// The `tsi_rust` module above is for backwards compatibility.
///
/// Example usage:
/// ```python
/// from tsi_rust_api import TSIBackend
///
/// # Initialize backend
/// backend = TSIBackend()
///
/// # Store schedule with ETL
/// metadata = backend.store_schedule(
///     schedule_json, possible_periods_json, dark_periods_json,
///     "My Schedule", populate_analytics=True
/// )
///
/// # Get visualization data
/// sky_map = tsi_rust_api.get_sky_map_data(metadata.schedule_id)
/// ```
#[pymodule]
fn tsi_rust_api(m: &Bound<'_, PyModule>) -> PyResult<()> {
    api::register_api_functions(m)
}

