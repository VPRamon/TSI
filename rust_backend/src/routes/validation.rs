use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// Validation issue.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
	pub block_id: i64,
	pub original_block_id: Option<String>,
	pub issue_type: String,
	pub category: String,
	pub criticality: String,
	pub field_name: Option<String>,
	pub current_value: Option<String>,
	pub expected_value: Option<String>,
	pub description: String,
}

/// Validation report data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
	pub schedule_id: i64,
	pub total_blocks: usize,
	pub valid_blocks: usize,
	pub impossible_blocks: Vec<ValidationIssue>,
	pub validation_errors: Vec<ValidationIssue>,
	pub validation_warnings: Vec<ValidationIssue>,
}

/// Validation route function name constant
pub const GET_VALIDATION_REPORT: &str = "get_validation_report";

/// Get validation report for a schedule.
#[pyfunction]
pub fn get_validation_report(schedule_id: i64) -> PyResult<ValidationReport> {
	let report = crate::services::py_get_validation_report(schedule_id)
		.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
	Ok(report)
}
