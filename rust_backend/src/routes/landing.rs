use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::api::types as api;
use crate::db::services as db_services;

/// Store a schedule in the database with optional analytics population.
///
/// This function mirrors the previous `store_schedule` implementation in the
/// API module but lives under `routes::landing` for routing/landing responsibilities.
#[pyfunction]
pub fn store_schedule(
	schedule_name: String,
	schedule_json: String,
	visibility_json: Option<String>,
) -> PyResult<i64> {
	let visibility_ref = visibility_json.as_deref();
	let schedule = db_services::parse_schedule_from_json(&schedule_name, &schedule_json, visibility_ref)
		.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
	let metadata = db_services::store_schedule_sync(&schedule, true, false)
		.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
	Ok(metadata.schedule_id.unwrap())
}

/// List all schedules in the database.
#[pyfunction]
pub fn list_schedules() -> PyResult<Vec<api::ScheduleInfo>> {
	let runtime = Runtime::new().map_err(|e: std::io::Error| {
		PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create async runtime: {}", e))
	})?;
	let repo = crate::db::get_repository()
		.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
	let schedules = runtime
		.block_on(db_services::list_schedules(repo.as_ref()))
		.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
	Ok(schedules)
}
