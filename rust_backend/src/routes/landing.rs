use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::db::services as db_services;
use serde::{Deserialize, Serialize};

/// Schedule information with block counts.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleInfo {
	pub schedule_id: i64,
	pub schedule_name: String,
}

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
pub fn list_schedules() -> PyResult<Vec<crate::api_tmp::ScheduleInfo>> {
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

pub const LIST_SCHEDULES: &str = "list_schedules";
pub const POST_SCHEDULE: &str = "store_schedule";

/// Register landing route functions and constants with the Python module.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
	m.add_function(wrap_pyfunction!(store_schedule, m)?)?;
	m.add_function(wrap_pyfunction!(list_schedules, m)?)?;
	m.add("LIST_SCHEDULES", LIST_SCHEDULES)?;
	m.add("POST_SCHEDULE", POST_SCHEDULE)?;
	m.add_class::<ScheduleInfo>()?;
	Ok(())
}
