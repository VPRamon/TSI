//! Python bindings for database operations.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use tokio::runtime::Runtime;

use crate::db::{
    models::{Schedule, SchedulingBlock},
    operations, pool, DbConfig,
};

/// Initialize database connection pool from environment variables.
#[pyfunction]
pub fn py_init_database() -> PyResult<()> {
    let config =
        DbConfig::from_env().map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(pool::init_pool(&config))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    Ok(())
}

/// Check database connection health.
#[pyfunction]
pub fn py_db_health_check() -> PyResult<bool> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(operations::health_check())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}

/// Store a preprocessed schedule in the database.
#[pyfunction]
pub fn py_store_schedule(
    schedule_name: &str,
    schedule_json: &str,
    visibility_json: Option<&str>,
) -> PyResult<PyObject> {
    // Heavy parsing + DB insert happens without the GIL held to avoid blocking Python.
    let metadata = Python::with_gil(|py| {
        py.allow_threads(|| -> PyResult<_> {
            let dark_periods = std::fs::read_to_string("data/dark_periods.json").map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to read dark_periods.json: {}",
                    e
                ))
            })?;

            let mut schedule: Schedule = crate::parsing::json_parser::parse_schedule_json_str(
                schedule_json,
                visibility_json,
                dark_periods.as_str(),
            )
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to parse schedule: {}",
                    e
                ))
            })?;
            schedule.name = schedule_name.to_string();

            let runtime = Runtime::new().map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to create async runtime: {}",
                    e
                ))
            })?;

            runtime
                .block_on(operations::store_schedule(&schedule))
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
        })
    })?;

    // Convert to Python dict
    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        dict.set_item("schedule_id", metadata.schedule_id)?;
        dict.set_item("schedule_name", metadata.schedule_name)?;
        dict.set_item("upload_timestamp", metadata.upload_timestamp.to_rfc3339())?;
        dict.set_item("checksum", metadata.checksum)?;
        Ok(dict.into())
    })
}

/// Fetch a schedule (metadata + blocks) from the database.
#[pyfunction]
pub fn py_get_schedule(
    schedule_id: Option<i64>,
    schedule_name: Option<&str>,
) -> PyResult<Schedule> {
    if schedule_id.is_none() && schedule_name.is_none() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Either schedule_id or schedule_name must be provided",
        ));
    }

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let owned_name = schedule_name.map(|s| s.to_string());
    runtime
        .block_on(operations::get_schedule(schedule_id, owned_name.as_deref()))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}

/// Fetch all scheduling blocks for a schedule ID.
#[pyfunction]
pub fn py_get_schedule_blocks(schedule_id: i64) -> PyResult<Vec<SchedulingBlock>> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(operations::get_blocks_for_schedule(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}

/// List all available schedules in the database.
#[pyfunction]
pub fn py_list_schedules() -> PyResult<PyObject> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let schedules = runtime
        .block_on(operations::list_schedules())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    Python::with_gil(|py| {
        let list = PyList::empty(py);
        for schedule_info in schedules {
            let dict = PyDict::new(py);
            dict.set_item("schedule_id", schedule_info.metadata.schedule_id)?;
            dict.set_item("schedule_name", schedule_info.metadata.schedule_name)?;
            dict.set_item(
                "upload_timestamp",
                schedule_info.metadata.upload_timestamp.to_rfc3339(),
            )?;
            dict.set_item("checksum", schedule_info.metadata.checksum)?;
            dict.set_item("total_blocks", schedule_info.total_blocks)?;
            dict.set_item("scheduled_blocks", schedule_info.scheduled_blocks)?;
            dict.set_item("unscheduled_blocks", schedule_info.unscheduled_blocks)?;
            list.append(dict)?;
        }
        Ok(list.into())
    })
}

/// Fetch dark periods for a schedule.
#[pyfunction]
pub fn py_fetch_dark_periods(schedule_id: Option<i64>) -> PyResult<PyObject> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let periods = runtime
        .block_on(operations::fetch_dark_periods_public(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    Python::with_gil(|py| {
        let list = PyList::empty(py);
        for (start, stop) in periods {
            list.append((start, stop))?;
        }
        Ok(list.into())
    })
}

/// Fetch possible (visibility) periods for a schedule.
#[pyfunction]
pub fn py_fetch_possible_periods(schedule_id: i64) -> PyResult<PyObject> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let periods = runtime
        .block_on(operations::fetch_possible_periods(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    Python::with_gil(|py| {
        let list = PyList::empty(py);
        for (sb_id, start, stop) in periods {
            list.append((sb_id, start, stop))?;
        }
        Ok(list.into())
    })
}

/// Compute visibility histogram for a schedule with filters.
///
/// This function fetches minimal block data from the database and computes
/// a time-binned histogram showing how many unique scheduling blocks are
/// visible in each time interval.
///
/// ## Arguments
/// * `schedule_id` - Schedule ID to analyze
/// * `start_unix` - Start of time range (Unix timestamp seconds)
/// * `end_unix` - End of time range (Unix timestamp seconds)
/// * `bin_duration_minutes` - Duration of each histogram bin in minutes
/// * `priority_min` - Optional minimum priority filter (inclusive)
/// * `priority_max` - Optional maximum priority filter (inclusive)
/// * `block_ids` - Optional list of specific block IDs to include
///
/// ## Returns
/// List of dictionaries with keys:
/// - `bin_start_unix`: Start of bin (Unix timestamp)
/// - `bin_end_unix`: End of bin (Unix timestamp)
/// - `count`: Number of unique blocks visible in this bin
///
/// ## Example
/// ```python
/// import tsi_rust
/// from datetime import datetime, timezone
///
/// start = int(datetime(2024, 1, 1, tzinfo=timezone.utc).timestamp())
/// end = int(datetime(2024, 1, 2, tzinfo=timezone.utc).timestamp())
///
/// bins = tsi_rust.py_get_visibility_histogram(
///     schedule_id=1,
///     start_unix=start,
///     end_unix=end,
///     bin_duration_minutes=60,
///     priority_min=5,
///     priority_max=10,
///     block_ids=None
/// )
///
/// for bin in bins:
///     print(f"Time: {bin['bin_start_unix']}, Visible: {bin['count']}")
/// ```
#[pyfunction]
#[pyo3(signature = (schedule_id, start_unix, end_unix, bin_duration_minutes, priority_min=None, priority_max=None, block_ids=None))]
pub fn py_get_visibility_histogram(
    py: Python,
    schedule_id: i64,
    start_unix: i64,
    end_unix: i64,
    bin_duration_minutes: i64,
    priority_min: Option<i32>,
    priority_max: Option<i32>,
    block_ids: Option<Vec<i64>>,
) -> PyResult<PyObject> {
    // Validate inputs
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

    // Release GIL for database and compute operations
    let bins = py.allow_threads(|| -> PyResult<_> {
        let runtime = Runtime::new().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create async runtime: {}",
                e
            ))
        })?;

        // Fetch blocks from database
        let blocks = runtime
            .block_on(crate::db::operations::fetch_blocks_for_histogram(
                schedule_id,
                priority_min,
                priority_max,
                block_ids.as_deref(),
            ))
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to fetch blocks: {}",
                    e
                ))
            })?;

        // Compute histogram
        crate::services::visibility::compute_visibility_histogram_rust(
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

    // Convert to Python list of dicts (JSON-serializable)
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

/// Get the time range (min/max timestamps) for a schedule's visibility periods.
///
/// This function queries all visibility periods for a schedule and returns
/// the minimum start time and maximum stop time as Unix timestamps.
///
/// ## Arguments
/// * `schedule_id` - Schedule ID to analyze
///
/// ## Returns
/// Tuple of (start_unix, end_unix) as Option. Returns None if no
/// visibility periods exist or if schedule not found.
///
/// ## Example
/// ```python
/// import tsi_rust
///
/// time_range = tsi_rust.py_get_schedule_time_range(schedule_id=1)
/// if time_range:
///     start_unix, end_unix = time_range
///     print(f"Schedule spans from {start_unix} to {end_unix}")
/// else:
///     print("No visibility periods found")
/// ```
#[pyfunction]
pub fn py_get_schedule_time_range(schedule_id: i64) -> PyResult<Option<(i64, i64)>> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let time_range_mjd = runtime
        .block_on(operations::get_schedule_time_range(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    // Convert MJD to Unix timestamps
    if let Some((min_mjd, max_mjd)) = time_range_mjd {
        // MJD epoch (1858-11-17 00:00:00 UTC) as Unix timestamp
        const MJD_EPOCH_UNIX: i64 = -3506716800;
        let start_unix = MJD_EPOCH_UNIX + (min_mjd * 86400.0) as i64;
        let end_unix = MJD_EPOCH_UNIX + (max_mjd * 86400.0) as i64;
        Ok(Some((start_unix, end_unix)))
    } else {
        Ok(None)
    }
}

/// Fetch visibility map data (priority range, block metadata) in one backend call.
#[pyfunction]
pub fn py_get_visibility_map_data(
    schedule_id: i64,
) -> PyResult<crate::db::models::VisibilityMapData> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(operations::fetch_visibility_map_data(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}
