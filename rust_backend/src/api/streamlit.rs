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
use pyo3::types::{PyDict, PyList};
use serde_json::Value;
use tokio::runtime::Runtime;

use crate::api::types as api;
use crate::algorithms;
use crate::db::repository::{analytics::AnalyticsRepository, visualization::VisualizationRepository};
use crate::db::services as db_services;
// Re-export landing route functions so they can be registered with the Python module
pub use crate::routes::landing::{list_schedules, store_schedule};
// Re-export route name constants so Python can reference them without hard-coded strings
pub use crate::routes::landing::{LIST_SCHEDULES, POST_SCHEDULE};
// Re-export validation route so it can be registered from routes module
pub use crate::routes::validation::{get_validation_report};
pub use crate::routes::validation::GET_VALIDATION_REPORT;
// Re-export visualization route so it can be registered from routes module
pub use crate::routes::skymap::{get_sky_map_data};
pub use crate::routes::skymap::GET_SKY_MAP_DATA;
// Re-export visibility route and constant
pub use crate::routes::visibility::{get_visibility_map_data};
pub use crate::routes::visibility::GET_VISIBILITY_MAP_DATA;
// Re-export distribution route and constant
pub use crate::routes::distribution::{get_distribution_data};
pub use crate::routes::distribution::GET_DISTRIBUTION_DATA;
// Re-export timeline route and constant
pub use crate::routes::timeline::{get_schedule_timeline_data};
pub use crate::routes::timeline::GET_SCHEDULE_TIMELINE_DATA;
// Re-export insights route and constant
pub use crate::routes::insights::{get_insights_data};
pub use crate::routes::insights::GET_INSIGHTS_DATA;

/// Register all API functions with the Python module.
///
/// This function is called from lib.rs to populate the tsi_rust_api module
/// with all exported functions and classes.
pub fn register_api_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
    m.add_function(wrap_pyfunction!(get_schedule_timeline_data, m)?)?;
    m.add_function(wrap_pyfunction!(get_insights_data, m)?)?;
    m.add_function(wrap_pyfunction!(get_trends_data, m)?)?;
    m.add_function(wrap_pyfunction!(get_compare_data, m)?)?;
    m.add_function(wrap_pyfunction!(get_visibility_map_data, m)?)?;

    // Legacy visibility histogram functions (expected by Python services)
    m.add_function(wrap_pyfunction!(py_get_schedule_time_range, m)?)?;
    m.add_function(wrap_pyfunction!(py_get_visibility_histogram, m)?)?;
    m.add_function(wrap_pyfunction!(py_get_visibility_histogram_analytics, m)?)?;

    // Algorithm operations
    m.add_function(wrap_pyfunction!(find_conflicts, m)?)?;
    m.add_function(wrap_pyfunction!(get_top_observations, m)?)?;

    // Validation
    m.add_function(wrap_pyfunction!(get_validation_report, m)?)?;

    // Register all API classes
    m.add_class::<api::ScheduleId>()?;
    m.add_class::<api::TargetId>()?;
    m.add_class::<api::ConstraintsId>()?;
    m.add_class::<api::SchedulingBlockId>()?;
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
    m.add_class::<api::VisibilityBinData>()?;
    m.add_class::<api::BlockHistogramData>()?;
    m.add_class::<api::HeatmapBinData>()?;
    m.add_class::<api::VisibilityTimeMetadata>()?;
    m.add_class::<api::VisibilityTimeBin>()?;
    m.add_class::<api::VisibilityBlockSummary>()?;
    m.add_class::<api::VisibilityMapData>()?;
    m.add_class::<api::ValidationIssue>()?;
    m.add_class::<api::ValidationReport>()?;
    m.add_class::<api::SchedulingConflict>()?;

    // Expose route name constants to Python to avoid stringly-typed calls
    m.add("LIST_SCHEDULES", crate::routes::landing::LIST_SCHEDULES)?;
    m.add("POST_SCHEDULE", crate::routes::landing::POST_SCHEDULE)?;
    m.add("GET_VALIDATION_REPORT", crate::routes::validation::GET_VALIDATION_REPORT)?;
    m.add("GET_SKY_MAP_DATA", crate::routes::skymap::GET_SKY_MAP_DATA)?;
    m.add("GET_DISTRIBUTION_DATA", crate::routes::distribution::GET_DISTRIBUTION_DATA)?;
    m.add("GET_SCHEDULE_TIMELINE_DATA", crate::routes::timeline::GET_SCHEDULE_TIMELINE_DATA)?;
    m.add("GET_INSIGHTS_DATA", crate::routes::insights::GET_INSIGHTS_DATA)?;
    m.add("GET_VISIBILITY_MAP_DATA", crate::routes::visibility::GET_VISIBILITY_MAP_DATA)?;

    Ok(())
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
    crate::db::init_repository()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
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
    let repo = crate::db::get_repository()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    runtime
        .block_on(db_services::health_check(repo.as_ref()))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
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
    let repo = crate::db::get_repository()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
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
    let repo = crate::db::get_repository()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
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
    let repo = crate::db::get_repository()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    runtime
        .block_on(db_services::has_analytics_data(repo.as_ref(), schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

// =========================================================
// Visualization Data Queries
// =========================================================

// Sky map visualization provided by `routes::visualization`

/// Get distribution visualization data (ETL-based).
/// This is provided by `routes::distribution`.

/// Get timeline visualization data.
///
/// Args:
///     schedule_id: Database ID of the schedule
///
/// Returns:
///     ScheduleTimelineData with scheduled blocks
/// Get timeline visualization data (provided by `routes::timeline`).

/// Get insights analysis data (provided by `routes::insights`).

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

#[pyfunction]
fn py_get_schedule_time_range(schedule_id: i64) -> PyResult<Option<(i64, i64)>> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;
    let repo = crate::db::get_repository()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    let time_range_period = runtime
        .block_on(db_services::get_schedule_time_range(repo.as_ref(), schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

    if let Some(period) = time_range_period {
        const MJD_EPOCH_UNIX: i64 = -3506716800;
        let start_unix = MJD_EPOCH_UNIX + (period.start.value() * 86400.0) as i64;
        let end_unix = MJD_EPOCH_UNIX + (period.stop.value() * 86400.0) as i64;
        Ok(Some((start_unix, end_unix)))
    } else {
        Ok(None)
    }
}

#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn py_get_visibility_histogram(
    py: Python,
    schedule_id: i64,
    start_unix: i64,
    end_unix: i64,
    bin_duration_minutes: i64,
    priority_min: Option<i32>,
    priority_max: Option<i32>,
    block_ids: Option<Vec<i64>>,
) -> PyResult<Py<PyAny>> {
    if start_unix >= end_unix {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "start_unix must be less than end_unix",
        ));
    }
    if bin_duration_minutes <= 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "bin_duration_minutes must be positive",
        ));
    }

    let bin_duration_seconds = bin_duration_minutes * 60;

    let bins = py.detach(|| -> PyResult<_> {
        let repo = crate::db::get_repository()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        let runtime = Runtime::new().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create async runtime: {}",
                e
            ))
        })?;

        let blocks = runtime
            .block_on(repo.fetch_blocks_for_histogram(
                schedule_id,
                priority_min,
                priority_max,
                block_ids,
            ))
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to fetch blocks: {}",
                    e
                ))
            })?;

        crate::services::compute_visibility_histogram_rust(
            blocks.into_iter(),
            start_unix,
            end_unix,
            bin_duration_seconds,
            priority_min,
            priority_max,
        )
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to compute histogram: {}",
                e
            ))
        })
    })?;

    let list = PyList::empty(py);
    for bin in bins {
        let dict = PyDict::new(py);
        dict.set_item("bin_start_unix", bin.bin_start_unix)?;
        dict.set_item("bin_end_unix", bin.bin_end_unix)?;
        dict.set_item("count", bin.visible_count)?;
        list.append(dict)?;
    }

    Ok(list.into())
}

#[pyfunction]
fn py_get_visibility_histogram_analytics(
    py: Python,
    schedule_id: i64,
    start_unix: i64,
    end_unix: i64,
    bin_duration_minutes: i64,
) -> PyResult<Py<PyAny>> {
    if start_unix >= end_unix {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "start_unix must be less than end_unix",
        ));
    }
    if bin_duration_minutes <= 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "bin_duration_minutes must be positive",
        ));
    }

    let bin_duration_seconds = bin_duration_minutes * 60;

    let bins = py.detach(|| -> PyResult<_> {
        let repo = crate::db::get_repository()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        let runtime = Runtime::new().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create async runtime: {}",
                e
            ))
        })?;

        runtime
            .block_on(repo.fetch_visibility_histogram_from_analytics(
                schedule_id,
                start_unix,
                end_unix,
                bin_duration_seconds,
            ))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    })?;

    let list = PyList::empty(py);
    for bin in bins {
        let dict = PyDict::new(py);
        dict.set_item("bin_start_unix", bin.bin_start_unix)?;
        dict.set_item("bin_end_unix", bin.bin_end_unix)?;
        dict.set_item("count", bin.visible_count)?;
        list.append(dict)?;
    }

    Ok(list.into())
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
// Validation operations are provided by `routes::validation`
// =========================================================
// Transformation Functions (Legacy API)
// =========================================================

/// Register transformation functions for backwards compatibility with tsi_rust module.
pub fn register_transformation_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_filter_by_range, m)?)?;
    m.add_function(wrap_pyfunction!(py_filter_by_scheduled, m)?)?;
    m.add_function(wrap_pyfunction!(py_filter_dataframe, m)?)?;
    m.add_function(wrap_pyfunction!(py_remove_duplicates, m)?)?;
    m.add_function(wrap_pyfunction!(py_remove_missing_coordinates, m)?)?;
    m.add_function(wrap_pyfunction!(py_validate_dataframe, m)?)?;
    m.add_function(wrap_pyfunction!(mjd_to_datetime, m)?)?;
    m.add_function(wrap_pyfunction!(datetime_to_mjd, m)?)?;
    m.add_function(wrap_pyfunction!(parse_visibility_periods, m)?)?;
    Ok(())
}

/// Filter DataFrame by numeric range on a column.
#[pyfunction]
#[pyo3(signature = (json_str, column, min_val, max_val))]
fn py_filter_by_range(
    json_str: String,
    column: String,
    min_val: f64,
    max_val: f64,
) -> PyResult<String> {
    let records: Vec<Value> = serde_json::from_str(&json_str).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e))
    })?;
    
    let filtered: Vec<Value> = records
        .into_iter()
        .filter(|record| {
            if let Some(val) = record.get(&column).and_then(|v| v.as_f64()) {
                val >= min_val && val <= max_val
            } else {
                false
            }
        })
        .collect();
    
    serde_json::to_string(&filtered).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Serialization failed: {}", e))
    })
}

/// Filter DataFrame by scheduled status.
#[pyfunction]
#[pyo3(signature = (json_str, filter_type))]
fn py_filter_by_scheduled(json_str: String, filter_type: String) -> PyResult<String> {
    let records: Vec<Value> = serde_json::from_str(&json_str).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e))
    })?;
    
    let filtered: Vec<Value> = match filter_type.as_str() {
        "All" => records,
        "Scheduled" => records.into_iter().filter(|r| {
            r.get("wasScheduled").and_then(|v| v.as_bool()).unwrap_or(false)
        }).collect(),
        "Unscheduled" => records.into_iter().filter(|r| {
            !r.get("wasScheduled").and_then(|v| v.as_bool()).unwrap_or(false)
        }).collect(),
        _ => records,
    };
    
    serde_json::to_string(&filtered).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Serialization failed: {}", e))
    })
}

/// Convert Modified Julian Date to Python datetime (UTC).
#[pyfunction]
fn mjd_to_datetime(mjd: f64) -> PyResult<Py<PyAny>> {
    Python::attach(|py| {
        let secs = (mjd - 40587.0) * 86400.0;
        let datetime_mod = py.import("datetime")?;
        let datetime_cls = datetime_mod.getattr("datetime")?;
        let timezone_utc = datetime_mod.getattr("timezone")?.getattr("utc")?;
        let dt = datetime_cls.call_method1("fromtimestamp", (secs, timezone_utc))?;
        Ok(dt.into())
    })
}

/// Convert Python datetime to Modified Julian Date (assumes UTC for naive datetimes).
#[pyfunction]
fn datetime_to_mjd(dt: Py<PyAny>) -> PyResult<f64> {
    Python::attach(|py| {
        let datetime_mod = py.import("datetime")?;
        let timezone_utc = datetime_mod.getattr("timezone")?.getattr("utc")?;
        let dt_obj = dt.bind(py);
        let tzinfo = dt_obj.getattr("tzinfo")?;

        let timestamp = if tzinfo.is_none() {
            let kwargs = PyDict::new(py);
            kwargs.set_item("tzinfo", &timezone_utc)?;
            let aware = dt_obj.call_method("replace", (), Some(&kwargs))?;
            aware.call_method0("timestamp")?.extract::<f64>()?
        } else {
            dt_obj.call_method0("timestamp")?.extract::<f64>()?
        };

        Ok(timestamp / 86400.0 + 40587.0)
    })
}

/// Filter DataFrame with multiple filter criteria.
#[pyfunction]
#[pyo3(signature = (json_str, priority_min, priority_max, scheduled_filter, priority_bins=None, block_ids=None))]
fn py_filter_dataframe(
    json_str: String,
    priority_min: f64,
    priority_max: f64,
    scheduled_filter: String,
    priority_bins: Option<Vec<String>>,
    block_ids: Option<Vec<String>>,
) -> PyResult<String> {
    let mut records: Vec<Value> = serde_json::from_str(&json_str).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e))
    })?;
    
    // Filter by priority range
    records.retain(|record| {
        if let Some(priority) = record.get("priority").and_then(|v| v.as_f64()) {
            priority >= priority_min && priority <= priority_max
        } else {
            false
        }
    });
    
    // Filter by scheduled status
    records = match scheduled_filter.as_str() {
        "Scheduled" => records.into_iter().filter(|r| {
            r.get("wasScheduled").and_then(|v| v.as_bool()).unwrap_or(false)
        }).collect(),
        "Unscheduled" => records.into_iter().filter(|r| {
            !r.get("wasScheduled").and_then(|v| v.as_bool()).unwrap_or(false)
        }).collect(),
        _ => records,
    };
    
    // Filter by priority bins
    if let Some(bins) = priority_bins {
        if !bins.is_empty() {
            records.retain(|record| {
                if let Some(bin) = record.get("priorityBin").and_then(|v| v.as_str()) {
                    bins.contains(&bin.to_string())
                } else {
                    false
                }
            });
        }
    }
    
    // Filter by block IDs
    if let Some(ids) = block_ids {
        if !ids.is_empty() {
            records.retain(|record| {
                if let Some(id) = record.get("schedulingBlockId").and_then(|v| v.as_str()) {
                    ids.contains(&id.to_string())
                } else {
                    false
                }
            });
        }
    }
    
    serde_json::to_string(&records).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Serialization failed: {}", e))
    })
}

/// Remove duplicate rows from DataFrame.
#[pyfunction]
#[pyo3(signature = (json_str, subset=None, keep=None))]
fn py_remove_duplicates(
    json_str: String,
    subset: Option<Vec<String>>,
    keep: Option<String>,
) -> PyResult<String> {
    let keep = keep.unwrap_or_else(|| "first".to_string());
    let records: Vec<Value> = serde_json::from_str(&json_str).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e))
    })?;
    
    let mut unique_records = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();
    
    for record in records.iter() {
        // Generate key from subset columns or entire record
        let key = if let Some(ref cols) = subset {
            let mut key_parts = Vec::new();
            for col in cols {
                if let Some(val) = record.get(col) {
                    key_parts.push(val.to_string());
                }
            }
            key_parts.join("|")
        } else {
            record.to_string()
        };
        
        match keep.as_str() {
            "first" => {
                if !seen_keys.contains(&key) {
                    seen_keys.insert(key);
                    unique_records.push(record.clone());
                }
            }
            "last" => {
                seen_keys.insert(key.clone());
                unique_records.retain(|r| {
                    let r_key = if let Some(ref cols) = subset {
                        let mut key_parts = Vec::new();
                        for col in cols {
                            if let Some(val) = r.get(col) {
                                key_parts.push(val.to_string());
                            }
                        }
                        key_parts.join("|")
                    } else {
                        r.to_string()
                    };
                    r_key != key
                });
                unique_records.push(record.clone());
            }
            "none" => {
                if !seen_keys.contains(&key) {
                    seen_keys.insert(key);
                    unique_records.push(record.clone());
                }
            }
            _ => {
                if !seen_keys.contains(&key) {
                    seen_keys.insert(key);
                    unique_records.push(record.clone());
                }
            }
        }
    }
    
    serde_json::to_string(&unique_records).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Serialization failed: {}", e))
    })
}

/// Remove observations with missing RA or Dec coordinates.
#[pyfunction]
fn py_remove_missing_coordinates(json_str: String) -> PyResult<String> {
    let records: Vec<Value> = serde_json::from_str(&json_str).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e))
    })?;
    
    let filtered: Vec<Value> = records
        .into_iter()
        .filter(|record| {
            let has_ra = record.get("raDeg").and_then(|v| v.as_f64()).is_some();
            let has_dec = record.get("decDeg").and_then(|v| v.as_f64()).is_some();
            has_ra && has_dec
        })
        .collect();
    
    serde_json::to_string(&filtered).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Serialization failed: {}", e))
    })
}

/// Validate DataFrame data quality.
#[pyfunction]
fn py_validate_dataframe(json_str: String) -> PyResult<(bool, Vec<String>)> {
    let records: Vec<Value> = serde_json::from_str(&json_str).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e))
    })?;
    
    let mut issues = Vec::new();
    
    // Check for missing coordinates
    let missing_coords = records.iter().filter(|r| {
        let has_ra = r.get("raDeg").and_then(|v| v.as_f64()).is_some();
        let has_dec = r.get("decDeg").and_then(|v| v.as_f64()).is_some();
        !has_ra || !has_dec
    }).count();
    
    if missing_coords > 0 {
        issues.push(format!("{} observations with missing coordinates", missing_coords));
    }
    
    // Check for invalid priorities
    let invalid_priorities = records.iter().filter(|r| {
        if let Some(p) = r.get("priority").and_then(|v| v.as_f64()) {
            p < 0.0 || p > 100.0
        } else {
            true
        }
    }).count();
    
    if invalid_priorities > 0 {
        issues.push(format!("{} observations with invalid priorities", invalid_priorities));
    }
    
    let is_valid = issues.is_empty();
    Ok((is_valid, issues))
}

/// Parse visibility periods from list of dicts to datetime tuples.
#[pyfunction]
fn parse_visibility_periods(periods: Vec<(String, String)>) -> PyResult<Vec<(String, String)>> {
    // Simply return as-is since Python will handle datetime parsing
    // This is a no-op shim for backwards compatibility
    Ok(periods)
}
