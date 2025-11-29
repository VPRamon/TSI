//! Python bindings for database operations.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use tokio::runtime::Runtime;

use crate::db::{operations, pool, DbConfig};

/// Initialize database connection pool from environment variables.
#[pyfunction]
pub fn py_init_database() -> PyResult<()> {
    let config = DbConfig::from_env()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    let runtime = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e)))?;

    runtime.block_on(pool::init_pool(&config))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    Ok(())
}

/// Check database connection health.
#[pyfunction]
pub fn py_db_health_check() -> PyResult<bool> {
    let runtime = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e)))?;

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
    use crate::db::models::Schedule;

    // Heavy parsing + DB insert happens without the GIL held to avoid blocking Python.
    let metadata = Python::with_gil(|py| {
        py.allow_threads(|| -> PyResult<_> {
            let dark_periods = std::fs::read_to_string("data/dark_periods.json")
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to read dark_periods.json: {}", e)))?;

            let mut schedule: Schedule = crate::parsing::json_parser::parse_schedule_json_str(
                schedule_json,
                visibility_json,
                dark_periods.as_str(),
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to parse schedule: {}", e)))?;
            schedule.name = schedule_name.to_string();

            let runtime = Runtime::new()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e)))?;

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

/// Fetch a schedule from the database.
/*#[pyfunction]
pub fn py_fetch_schedule(
    schedule_id: Option<i64>,
    schedule_name: Option<&str>,
) -> PyResult<PyDataFrame> {
    let runtime = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e)))?;

    let df = runtime
        .block_on(operations::fetch_schedule(schedule_id, schedule_name))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    Ok(PyDataFrame(df))
}*/

/// List all available schedules in the database.
#[pyfunction]
pub fn py_list_schedules() -> PyResult<PyObject> {
    let runtime = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e)))?;

    let schedules = runtime
        .block_on(operations::list_schedules())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    Python::with_gil(|py| {
        let list = PyList::empty(py);
        for schedule_info in schedules {
            let dict = PyDict::new(py);
            dict.set_item("schedule_id", schedule_info.metadata.schedule_id)?;
            dict.set_item("schedule_name", schedule_info.metadata.schedule_name)?;
            dict.set_item("upload_timestamp", schedule_info.metadata.upload_timestamp.to_rfc3339())?;
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
    let runtime = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e)))?;

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
    let runtime = Runtime::new()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e)))?;

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
