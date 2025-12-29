use crate::db;
use crate::db::repository::visualization::VisualizationRepository;
use crate::services;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

// =========================================================
// Visibility Map types + route
// =========================================================

/// Block summary for visibility map.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityBlockSummary {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
    pub priority: f64,
    pub num_visibility_periods: usize,
    pub scheduled: bool,
}

/// Visibility map visualization data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityMapData {
    pub blocks: Vec<VisibilityBlockSummary>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
}

/// Route function name constant for visibility map
pub const GET_VISIBILITY_MAP_DATA: &str = "get_visibility_map_data";
/// Route function name constant for schedule time range
pub const GET_SCHEDULE_TIME_RANGE: &str = "get_schedule_time_range";
/// Route function name constant for visibility histogram
pub const GET_VISIBILITY_HISTOGRAM: &str = "get_visibility_histogram";

/// Get visibility map data (wraps repository call)
#[pyfunction]
pub fn get_visibility_map_data(schedule_id: crate::api::ScheduleId) -> PyResult<VisibilityMapData> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let repo = crate::db::get_repository()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    let data = runtime
        .block_on(repo.fetch_visibility_map_data(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

    Ok(data)
}

/// Register visibility-related functions, classes, and constants.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_visibility_map_data, m)?)?;
    m.add_function(wrap_pyfunction!(get_schedule_time_range, m)?)?;
    m.add_function(wrap_pyfunction!(get_visibility_histogram, m)?)?;
    m.add_class::<VisibilityBlockSummary>()?;
    m.add_class::<VisibilityMapData>()?;
    m.add("GET_VISIBILITY_MAP_DATA", GET_VISIBILITY_MAP_DATA)?;
    m.add("GET_SCHEDULE_TIME_RANGE", GET_SCHEDULE_TIME_RANGE)?;
    m.add("GET_VISIBILITY_HISTOGRAM", GET_VISIBILITY_HISTOGRAM)?;
    Ok(())
}

#[pyfunction]
fn get_schedule_time_range(schedule_id: crate::api::ScheduleId) -> PyResult<Option<(i64, i64)>> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;
    let repo = crate::db::get_repository()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    let time_range_period = runtime
        .block_on(db::services::get_schedule_time_range(
            repo.as_ref(),
            schedule_id,
        ))
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
fn get_visibility_histogram(
    py: Python,
    schedule_id: crate::api::ScheduleId,
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

        services::compute_visibility_histogram_rust(
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
