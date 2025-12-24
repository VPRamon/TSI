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

pub mod algorithms;
pub mod api;
pub mod db;
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
/// - `load_schedule`: Load schedule from JSON file
/// - `load_schedule_from_json`: Load schedule from JSON file
/// - `load_schedule_from_json_str`: Load schedule from JSON string
/// - `load_schedule_from_iteration`: Load schedule from iteration directory
/// - `load_dark_periods`: Load dark period constraints
///
/// ## Preprocessing Functions
/// - `py_preprocess_schedule`: Comprehensive schedule preprocessing pipeline
/// - `py_preprocess_schedule_str`: Preprocess schedule from JSON string
/// - `py_validate_schedule`: Validate schedule data without enrichment
///
/// ## Analysis Functions
/// - `py_get_top_observations`: Get top N observations by criteria
/// - `py_find_conflicts`: Detect scheduling conflicts
/// - `py_get_schedule_summary`: Get pre-computed schedule metrics (preferred over DataFrame-based metrics)
///
/// ## Classes
/// - `PyValidationResult`: Validation results with errors and warnings
/// - `PySchedulingConflict`: Detected scheduling conflict information
///
/// # Example
///
/// ```python
/// import tsi_rust
///
/// # The module is automatically initialized when imported
/// df = tsi_rust.load_schedule("schedule.json")
/// ```
///
/// **DEPRECATED**: This module is deprecated. Use `tsi_rust_api` instead.
/// Internal models no longer have PyO3 derives, so this module cannot be compiled.
/// All functionality is available in the new `tsi_rust_api` module.
/* DEPRECATED - commented out due to removal of PyO3 from internal models
#[pymodule]
fn tsi_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Note: Time conversion, data loading, and preprocessing functions are now
    // exposed through the TSIBackend Python wrapper class (tsi_rust_api.py) which
    // provides a more ergonomic, high-level API. Direct function exports below
    // focus on core algorithms, transformations, and database operations.

    // Register algorithm functions
    m.add_function(wrap_pyfunction!(python::py_get_top_observations, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_find_conflicts, m)?)?;

    // Register transformation functions
    python::transformations::register_transformation_functions(m)?;

    // Register time conversion functions
    m.add_function(wrap_pyfunction!(python::mjd_to_datetime, m)?)?;
    m.add_function(wrap_pyfunction!(python::datetime_to_mjd, m)?)?;

    // Register database functions
    m.add_function(wrap_pyfunction!(python::py_init_database, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_db_health_check, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_store_schedule, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_store_schedule_with_options, m)?)?;
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
    m.add_function(wrap_pyfunction!(
        python::py_populate_visibility_time_bins,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(python::py_has_visibility_time_bins, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_delete_visibility_time_bins, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_visibility_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(
        python::py_get_visibility_histogram_analytics,
        m
    )?)?;

    // Register service functions (ETL-based)
    m.add_function(wrap_pyfunction!(services::py_get_sky_map_data, m)?)?;
    m.add_function(wrap_pyfunction!(
        services::py_get_sky_map_data_analytics,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(services::py_get_distribution_data, m)?)?;
    m.add_function(wrap_pyfunction!(
        services::py_get_distribution_data_analytics,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        services::py_get_schedule_timeline_data,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(services::py_get_insights_data, m)?)?;
    m.add_function(wrap_pyfunction!(services::py_get_trends_data, m)?)?;
    m.add_function(wrap_pyfunction!(services::py_get_compare_data, m)?)?;

    // Validation report function
    m.add_function(wrap_pyfunction!(services::py_get_validation_report, m)?)?;

    // Register classes (PyO3 data structures exposed to Python)
    m.add_class::<python::PySchedulingConflict>()?;
    m.add_class::<db::models::Schedule>()?;
    // Note: Classes are now registered via the tsi_rust_api module
    // The old tsi_rust module only exposes functions, not classes
    // This keeps backward compatibility for function calls while
    // the new API provides proper type isolation
    
    Ok(())
}
*/

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

