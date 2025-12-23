//! Streamlit API Functions.
//!
//! This module contains all `#[pyfunction]` exports for the Streamlit Python application.
//! Each function acts as a thin wrapper around internal service/repository calls,
//! converting between API DTOs and internal models at the boundary.
//!
//! ## Design Patterns
//!
//! 1. Accept API DTOs or primitives as parameters
//! 2. Convert to internal types using conversion traits
//! 3. Call internal service/repository methods
//! 4. Convert results back to API DTOs
//! 5. Return to Python with proper error handling

use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde_json::Value;
use tokio::runtime::Runtime;

use crate::api::types as api;
use crate::algorithms;
use crate::db::services as db_services;

/// Register all API functions with the Python module.
///
/// This function is called from lib.rs to populate the tsi_rust_api module
/// with all exported functions and classes.
pub fn register_api_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Time conversion functions
    m.add_function(wrap_pyfunction!(mjd_to_datetime, m)?)?;
    m.add_function(wrap_pyfunction!(datetime_to_mjd, m)?)?;

    // Database initialization
    m.add_function(wrap_pyfunction!(init_database, m)?)?;
    m.add_function(wrap_pyfunction!(db_health_check, m)?)?;

    // Core schedule operations
    m.add_function(wrap_pyfunction!(store_schedule, m)?)?;
    m.add_function(wrap_pyfunction!(list_schedules, m)?)?;
    m.add_function(wrap_pyfunction!(get_schedule, m)?)?;

    // Analytics ETL operations
    m.add_function(wrap_pyfunction!(populate_analytics, m)?)?;
    m.add_function(wrap_pyfunction!(has_analytics_data, m)?)?;

    // Visualization data queries
    m.add_function(wrap_pyfunction!(get_sky_map_data, m)?)?;
    m.add_function(wrap_pyfunction!(get_distribution_data, m)?)?;
    m.add_function(wrap_pyfunction!(get_timeline_data, m)?)?;
    m.add_function(wrap_pyfunction!(get_insights_data, m)?)?;
    m.add_function(wrap_pyfunction!(get_trends_data, m)?)?;
    m.add_function(wrap_pyfunction!(get_compare_data, m)?)?;

    // Algorithm operations
    m.add_function(wrap_pyfunction!(find_conflicts, m)?)?;
    m.add_function(wrap_pyfunction!(get_top_observations, m)?)?;

    // Validation
    m.add_function(wrap_pyfunction!(get_validation_report, m)?)?;

    // Register all API classes
    m.add_class::<api::ScheduleId>()?;
    m.add_class::<api::Period>()?;
    m.add_class::<api::Constraints>()?;
    m.add_class::<api::SchedulingBlock>()?;
    m.add_class::<api::Schedule>()?;
    m.add_class::<api::ScheduleMetadata>()?;
    m.add_class::<api::ScheduleInfo>()?;
    m.add_class::<api::LightweightBlock>()?;
    m.add_class::<api::PriorityBinInfo>()?;
    m.add_class::<api::SkyMapData>()?;
    m.add_class::<api::DistributionBlock>()?;
    m.add_class::<api::DistributionStats>()?;
    m.add_class::<api::DistributionData>()?;
    m.add_class::<api::ScheduleTimelineBlock>()?;
    m.add_class::<api::ScheduleTimelineData>()?;
    m.add_class::<api::InsightsBlock>()?;
    m.add_class::<api::AnalyticsMetrics>()?;
    m.add_class::<api::CorrelationEntry>()?;
    m.add_class::<api::ConflictRecord>()?;
    m.add_class::<api::TopObservation>()?;
    m.add_class::<api::InsightsData>()?;
    m.add_class::<api::TrendsBlock>()?;
    m.add_class::<api::EmpiricalRatePoint>()?;
    m.add_class::<api::SmoothedPoint>()?;
    m.add_class::<api::HeatmapBin>()?;
    m.add_class::<api::TrendsMetrics>()?;
    m.add_class::<api::TrendsData>()?;
    m.add_class::<api::CompareBlock>()?;
    m.add_class::<api::CompareStats>()?;
    m.add_class::<api::SchedulingChange>()?;
    m.add_class::<api::CompareData>()?;
    m.add_class::<api::ScheduleSummary>()?;
    m.add_class::<api::PriorityRate>()?;
    m.add_class::<api::VisibilityBin>()?;
    m.add_class::<api::HeatmapBinData>()?;
    m.add_class::<api::VisibilityTimeMetadata>()?;
    m.add_class::<api::VisibilityTimeBin>()?;
    m.add_class::<api::VisibilityBlockSummary>()?;
    m.add_class::<api::VisibilityMapData>()?;
    m.add_class::<api::ValidationIssue>()?;
    m.add_class::<api::ValidationReport>()?;
    m.add_class::<api::SchedulingConflict>()?;

    Ok(())
}

// =========================================================
// Time Conversion Functions
// =========================================================

/// Convert Modified Julian Date to Python datetime.
///
/// Args:
///     mjd: Modified Julian Date value
///
/// Returns:
///     Python datetime object (UTC timezone)
#[pyfunction]
fn mjd_to_datetime(py: Python<'_>, mjd: f64) -> PyResult<Py<PyAny>> {
    // Convert MJD -> seconds since UNIX epoch then use Python's datetime
    let secs = (mjd - 40587.0) * 86400.0;
    let dt = py
        .import("datetime")?
        .getattr("datetime")?
        .call1((secs,))?;
    Ok(dt.into())
}

/// Convert Python datetime to Modified Julian Date.
///
/// Args:
///     dt: Python datetime object
///
/// Returns:
///     Modified Julian Date as float
#[pyfunction]
fn datetime_to_mjd(dt: Py<PyAny>) -> PyResult<f64> {
    Python::with_gil(|py| {
        let dt_obj = dt.as_ref();
        let datetime_mod = py.import("datetime")?;
        let timezone = datetime_mod.getattr("timezone")?.getattr("utc")?;

        let tzinfo = dt_obj.getattr(py, "tzinfo")?;

        let timestamp = if tzinfo.is_none(py) {
            let kwargs = PyDict::new(py);
            kwargs.set_item("tzinfo", timezone)?;
            let aware = dt_obj.call_method(py, "replace", (), Some(&kwargs))?;
            aware.call_method0(py, "timestamp")?.extract::<f64>(py)?
        } else {
            dt_obj.call_method0(py, "timestamp")?.extract::<f64>(py)?
        };

        let mjd = timestamp / 86400.0 + 40587.0;
        Ok(mjd)
    })
}

// =========================================================
// Database Operations
// =========================================================

/// Initialize the database repository (local or Azure SQL).
///
/// This function must be called before any other database operations.
/// It sets up the global repository singleton based on configuration.
///
/// Returns:
///     Success message string
#[pyfunction]
fn init_database() -> PyResult<()> {
    crate::python::py_init_database()
}

/// Check database health and connectivity.
///
/// Returns:
///     Health status message
#[pyfunction]
fn db_health_check() -> PyResult<bool> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;
    let repo = crate::python::get_repository()?;
    runtime
        .block_on(db_services::health_check(repo.as_ref()))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Store a schedule in the database with optional analytics population.
///
/// Args:
///     schedule_json: JSON string of schedule data
///     possible_periods_json: JSON string of visibility periods
///     dark_periods_json: JSON string of dark periods
///     schedule_name: Human-readable name for the schedule
///     populate_analytics: Whether to run Phase 1 analytics ETL
///     skip_time_bins: Whether to skip Phase 3 time bin population
///
/// Returns:
///     ScheduleMetadata with database ID and checksum
#[pyfunction]
#[pyo3(signature = (schedule_name, schedule_json, visibility_json=None))]
fn store_schedule(
    schedule_name: String,
    schedule_json: String,
    visibility_json: Option<String>,
) -> PyResult<String> {
    // py_store_schedule returns Py<PyAny> (a Python dict), not a Rust struct
    // We return the schedule name as confirmation
    let visibility_ref = visibility_json.as_deref();
    let schedule = crate::python::parse_schedule_from_json(&schedule_name, &schedule_json, visibility_ref)?;
    let _metadata = crate::python::store_schedule_in_db(&schedule, true, false)?;
    Ok(schedule_name)
}

/// List all schedules in the database.
///
/// Returns:
///     List of ScheduleInfo objects with metadata and counts
#[pyfunction]
fn list_schedules() -> PyResult<Vec<api::ScheduleInfo>> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e))
    })?;
    let repo = crate::python::get_repository()?;
    let schedules = runtime
        .block_on(db_services::list_schedules(repo.as_ref()))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
    Ok(schedules.iter().map(|s| api::ScheduleInfo::from(s)).collect())
}

/// Get a specific schedule by ID.
///
/// Args:
///     schedule_id: Database ID of the schedule
///
/// Returns:
///     Schedule object with all blocks and periods
#[pyfunction]
fn get_schedule(_schedule_id: i64) -> PyResult<api::Schedule> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e))
    })?;
    let repo = crate::python::get_repository()?;
    let schedule = runtime
        .block_on(db_services::get_schedule(repo.as_ref(), _schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
    Ok((&schedule).into())
}

// =========================================================
// Analytics ETL Operations
// =========================================================

/// Populate Phase 1 analytics (block-level denormalized data).
///
/// Args:
///     schedule_id: Database ID of the schedule
///
/// Returns:
///     Number of blocks processed
#[pyfunction]
fn populate_analytics(schedule_id: i64) -> PyResult<usize> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e))
    })?;
    let repo = crate::python::get_repository()?;
    runtime
        .block_on(db_services::ensure_analytics(repo.as_ref(), schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
    Ok(0)
}

/// Check if Phase 1 analytics exist for a schedule.
///
/// Args:
///     schedule_id: Database ID of the schedule
///
/// Returns:
///     True if analytics data exists
#[pyfunction]
fn has_analytics_data(schedule_id: i64) -> PyResult<bool> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e))
    })?;
    let repo = crate::python::get_repository()?;
    runtime
        .block_on(db_services::has_analytics_data(repo.as_ref(), schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

// =========================================================
// Visualization Data Queries
// =========================================================

/// Get sky map visualization data (ETL-based).
///
/// Args:
///     schedule_id: Database ID of the schedule
///
/// Returns:
///     SkyMapData with blocks and priority bins
#[pyfunction]
fn get_sky_map_data(schedule_id: i64) -> PyResult<api::SkyMapData> {
    let data = crate::services::py_get_sky_map_data_analytics(schedule_id)?;
    Ok((&data).into())
}

/// Get distribution visualization data (ETL-based).
///
/// Args:
///     schedule_id: Database ID of the schedule
///
/// Returns:
///     DistributionData with blocks and statistics
#[pyfunction]
fn get_distribution_data(schedule_id: i64) -> PyResult<api::DistributionData> {
    let data = crate::services::py_get_distribution_data_analytics(schedule_id)?;
    Ok((&data).into())
}

/// Get timeline visualization data.
///
/// Args:
///     schedule_id: Database ID of the schedule
///
/// Returns:
///     ScheduleTimelineData with scheduled blocks
#[pyfunction]
fn get_timeline_data(schedule_id: i64) -> PyResult<api::ScheduleTimelineData> {
    let data = crate::services::py_get_schedule_timeline_data(schedule_id)?;
    Ok((&data).into())
}

/// Get insights analysis data.
///
/// Args:
///     schedule_id: Database ID of the schedule
///
/// Returns:
///     InsightsData with blocks, metrics, correlations, and conflicts
#[pyfunction]
fn get_insights_data(schedule_id: i64) -> PyResult<api::InsightsData> {
    let data = crate::services::py_get_insights_data(schedule_id)?;
    Ok((&data).into())
}

/// Get trends analysis data.
///
/// Args:
///     schedule_id: Database ID of the schedule
///
/// Returns:
///     TrendsData with empirical rates, smoothed trends, and heatmaps
#[pyfunction]
fn get_trends_data(schedule_id: i64) -> PyResult<api::TrendsData> {
    // py_get_trends_data requires: schedule_id, n_priority_bins, smoothing_window, n_time_bins
    let data = crate::services::py_get_trends_data(schedule_id, 10, 0.5, 12)?;
    Ok((&data).into())
}

/// Get schedule comparison data.
///
/// Args:
///     schedule_id_a: Database ID of first schedule
///     schedule_id_b: Database ID of second schedule
///
/// Returns:
///     CompareData with comparison blocks, stats, and changes
#[pyfunction]
fn get_compare_data(schedule_id_a: i64, schedule_id_b: i64) -> PyResult<api::CompareData> {
    // py_get_compare_data requires: schedule_id_a, schedule_id_b, name_a, name_b
    let data = crate::services::py_get_compare_data(
        schedule_id_a,
        schedule_id_b,
        "Schedule A".to_string(),
        "Schedule B".to_string(),
    )?;
    Ok((&data).into())
}

// =========================================================
// Algorithm Operations
// =========================================================

/// Find scheduling conflicts in a schedule JSON.
///
/// Args:
///     schedule_json: JSON string of schedule data
///
/// Returns:
///     List of SchedulingConflict objects
#[pyfunction]
fn find_conflicts(schedule_json: String) -> PyResult<Vec<api::SchedulingConflict>> {
    let records: Vec<Value> = serde_json::from_str(&schedule_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e))
    })?;
    let conflicts = algorithms::find_conflicts(&records).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Conflict detection failed: {}", e))
    })?;
    Ok(conflicts.iter().map(|c| api::SchedulingConflict::from(c)).collect())
}

/// Get top N observations by a specific criterion.
///
/// Args:
///     schedule_json: JSON string of schedule data
///     by: Criterion to sort by ("priority", "visibility", etc.)
///     n: Number of top observations to return
///
/// Returns:
///     List of TopObservation objects
#[pyfunction]
#[pyo3(signature = (schedule_json, by, n=10))]
fn get_top_observations(schedule_json: String, by: String, n: usize) -> PyResult<String> {
    let records: Vec<Value> = serde_json::from_str(&schedule_json).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e))
    })?;
    let top = algorithms::get_top_observations(&records, &by, n).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Top observations failed: {}", e))
    })?;
    serde_json::to_string(&top).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Serialization failed: {}", e)))
}

// =========================================================
// Validation Operations
// =========================================================

/// Get validation report for a schedule.
///
/// Args:
///     schedule_id: Database ID of the schedule
///
/// Returns:
///     ValidationReport with issues and summary
#[pyfunction]
fn get_validation_report(schedule_id: i64) -> PyResult<api::ValidationReport> {
    let report = crate::services::py_get_validation_report(schedule_id)?;
    Ok((&report).into())
}
