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
//! - **Data Loading**: Parse observation schedules from JSON and CSV formats
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
//! - [`core`]: Domain models for scheduling blocks and time periods
//! - [`parsing`]: JSON/CSV parsers and visibility period handling
//! - [`preprocessing`]: Data validation, enrichment, and pipeline orchestration
//! - [`algorithms`]: Analytics, conflict detection, and optimization routines
//! - [`transformations`]: Data transformation utilities
//! - [`io`]: High-level loaders combining parsing and domain logic
//! - [`python`]: PyO3 bindings exposing Rust functionality to Python
//!
//! ## Python API Example
//!
//! ```python
//! import tsi_rust
//!
//! # Load and preprocess schedule data
//! df, validation = tsi_rust.py_preprocess_schedule(
//!     "data/schedule.json",
//!     "data/possible_periods.json",
//!     validate=True
//! )
//!
//! # Convert MJD to datetime
//! dt = tsi_rust.mjd_to_datetime(59000.0)
//!
//! # Compute analytics
//! metrics = tsi_rust.py_compute_metrics(df)
//! print(f"Scheduling rate: {metrics.scheduling_rate:.2%}")
//! ```
//!
//! ## Performance
//!
//! This Rust backend is designed for high-performance batch processing of large
//! observation schedules. Key optimizations include:
//!
//! - Zero-copy parsing where possible
//! - Polars DataFrames for columnar data processing
//! - Parallel batch operations
//! - Minimal allocations in hot paths

use pyo3::prelude::*;

pub mod algorithms;
pub mod core;
pub mod db;
pub mod parsing;
pub mod python;
pub mod services;
pub mod transformations;

/// Python module entry point for TSI Rust Backend.
///
/// This function initializes the `tsi_rust` Python module, registering all functions
/// and classes that form the Python API. The module provides high-performance
/// telescope scheduling operations to Python applications.
///
/// # Module Contents
///
/// ## Time Conversion Functions
/// - `mjd_to_datetime`: Convert Modified Julian Date to Python datetime
/// - `datetime_to_mjd`: Convert Python datetime to Modified Julian Date
/// - `parse_visibility_periods`: Parse visibility period strings
///
/// ## Data Loading Functions
/// - `load_schedule`: Auto-detect and load schedule from JSON or CSV
/// - `load_schedule_from_json`: Load schedule from JSON file
/// - `load_schedule_from_json_str`: Load schedule from JSON string
/// - `load_schedule_from_csv`: Load schedule from CSV file
/// - `load_schedule_from_iteration`: Load schedule from iteration directory
/// - `load_dark_periods`: Load dark period constraints
///
/// ## Preprocessing Functions
/// - `py_preprocess_schedule`: Comprehensive schedule preprocessing pipeline
/// - `py_preprocess_schedule_str`: Preprocess schedule from JSON string
/// - `py_validate_schedule`: Validate schedule data without enrichment
///
/// ## Analysis Functions
/// - `py_compute_metrics`: Compute dataset-level summary statistics
/// - `py_compute_correlations`: Compute correlation matrices
/// - `py_get_top_observations`: Get top N observations by criteria
/// - `py_find_conflicts`: Detect scheduling conflicts
/// - `py_greedy_schedule`: Run greedy scheduling optimization
///
/// ## Classes
/// - `PyValidationResult`: Validation results with errors and warnings
/// - `PyAnalyticsSnapshot`: Dataset-level analytics summary
/// - `PySchedulingConflict`: Detected scheduling conflict information
/// - `PyOptimizationResult`: Results from scheduling optimization
///
/// # Example
///
/// ```python
/// import tsi_rust
///
/// # The module is automatically initialized when imported
/// df = tsi_rust.load_schedule("schedule.json")
/// ```
#[pymodule]
fn tsi_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Note: Time conversion, data loading, and preprocessing functions are now
    // exposed through the TSIBackend Python wrapper class (tsi_rust_api.py) which
    // provides a more ergonomic, high-level API. Direct function exports below
    // focus on core algorithms, transformations, and database operations.

    // Register algorithm functions
    m.add_function(wrap_pyfunction!(python::py_compute_metrics, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_compute_correlations, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_top_observations, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_find_conflicts, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_greedy_schedule, m)?)?;

    // Register transformation functions
    python::transformations::register_transformation_functions(m)?;

    // Register time conversion functions
    m.add_function(wrap_pyfunction!(python::mjd_to_datetime, m)?)?;
    m.add_function(wrap_pyfunction!(python::datetime_to_mjd, m)?)?;

    // Register database functions
    m.add_function(wrap_pyfunction!(python::py_init_database, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_db_health_check, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_store_schedule, m)?)?;
    //m.add_function(wrap_pyfunction!(python::py_fetch_schedule, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_list_schedules, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_fetch_dark_periods, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_fetch_possible_periods, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_fetch_compare_blocks, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_schedule, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_schedule_blocks, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_visibility_histogram, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_schedule_time_range, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_visibility_map_data, m)?)?;
    
    // Phase 1: Block-level analytics ETL functions
    m.add_function(wrap_pyfunction!(python::py_populate_analytics, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_has_analytics_data, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_delete_analytics, m)?)?;
    
    // Phase 2: Summary analytics ETL functions
    m.add_function(wrap_pyfunction!(python::py_populate_summary_analytics, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_has_summary_analytics, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_delete_summary_analytics, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_schedule_summary, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_priority_rates, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_visibility_bins, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_heatmap_bins, m)?)?;
    
    // Phase 3: Visibility time bins (pre-computed histogram) functions
    m.add_function(wrap_pyfunction!(python::py_populate_visibility_time_bins, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_has_visibility_time_bins, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_delete_visibility_time_bins, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_visibility_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_visibility_histogram_analytics, m)?)?;

    // Register service functions
    m.add_function(wrap_pyfunction!(services::py_get_sky_map_data, m)?)?;
    m.add_function(wrap_pyfunction!(services::py_get_distribution_data, m)?)?;
    m.add_function(wrap_pyfunction!(services::py_get_schedule_timeline_data, m)?)?;
    m.add_function(wrap_pyfunction!(services::py_get_insights_data, m)?)?;
    m.add_function(wrap_pyfunction!(services::py_get_trends_data, m)?)?;
    m.add_function(wrap_pyfunction!(services::py_get_compare_data, m)?)?;
    m.add_function(wrap_pyfunction!(services::py_compute_compare_data, m)?)?;

    // Register classes (PyO3 data structures exposed to Python)
    m.add_class::<python::PyAnalyticsSnapshot>()?;
    m.add_class::<python::PySchedulingConflict>()?;
    m.add_class::<python::PyOptimizationResult>()?;
    m.add_class::<db::models::Schedule>()?;
    m.add_class::<db::models::SchedulingBlock>()?;
    m.add_class::<db::models::Constraints>()?;
    m.add_class::<db::models::Period>()?;
    m.add_class::<db::models::ScheduleMetadata>()?;
    m.add_class::<db::models::ScheduleInfo>()?;
    m.add_class::<db::models::ScheduleId>()?;
    m.add_class::<db::models::TargetId>()?;
    m.add_class::<db::models::ConstraintsId>()?;
    m.add_class::<db::models::SchedulingBlockId>()?;
    m.add_class::<db::models::LightweightBlock>()?;
    m.add_class::<db::models::PriorityBinInfo>()?;
    m.add_class::<db::models::SkyMapData>()?;
    m.add_class::<db::models::DistributionBlock>()?;
    m.add_class::<db::models::DistributionStats>()?;
    m.add_class::<db::models::DistributionData>()?;
    m.add_class::<db::models::VisibilityBlockSummary>()?;
    m.add_class::<db::models::VisibilityMapData>()?;
    m.add_class::<db::models::ScheduleTimelineBlock>()?;
    m.add_class::<db::models::ScheduleTimelineData>()?;
    m.add_class::<db::models::InsightsBlock>()?;
    m.add_class::<db::models::AnalyticsMetrics>()?;
    m.add_class::<db::models::CorrelationEntry>()?;
    m.add_class::<db::models::ConflictRecord>()?;
    m.add_class::<db::models::TopObservation>()?;
    m.add_class::<db::models::InsightsData>()?;
    m.add_class::<db::models::TrendsBlock>()?;
    m.add_class::<db::models::EmpiricalRatePoint>()?;
    m.add_class::<db::models::SmoothedPoint>()?;
    m.add_class::<db::models::HeatmapBin>()?;
    m.add_class::<db::models::TrendsMetrics>()?;
    m.add_class::<db::models::TrendsData>()?;
    m.add_class::<db::models::CompareBlock>()?;
    m.add_class::<db::models::CompareStats>()?;
    m.add_class::<db::models::SchedulingChange>()?;
    m.add_class::<db::models::CompareData>()?;
    
    // Phase 2: Summary analytics classes
    m.add_class::<db::ScheduleSummary>()?;
    m.add_class::<db::PriorityRate>()?;
    m.add_class::<db::VisibilityBin>()?;
    m.add_class::<db::HeatmapBinData>()?;
    
    // Phase 3: Visibility time bins classes
    m.add_class::<db::VisibilityTimeMetadata>()?;
    m.add_class::<db::VisibilityTimeBin>()?;

    Ok(())
}
