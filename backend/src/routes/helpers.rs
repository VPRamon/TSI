use pyo3::prelude::*;
use tokio::runtime::Runtime;

/// Get full schedule by ID and return the `Schedule` DTO to Python.
#[pyfunction]
pub fn get_schedule(schedule_id: crate::api::ScheduleId) -> PyResult<crate::api::Schedule> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let repo = crate::db::get_repository()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    let schedule = runtime
        .block_on(crate::db::services::get_schedule(repo.as_ref(), schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

    Ok(schedule)
}

pub const GET_SCHEDULE: &str = "get_schedule";

pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_schedule, m)?)?;
    m.add("GET_SCHEDULE", GET_SCHEDULE)?;
    Ok(())
}
