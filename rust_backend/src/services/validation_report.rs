//! Python bindings for validation report functionality.
//!
//! This module exposes validation report data to Python via PyO3.

use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::db::repository::ValidationRepository;

// Import the global repository accessor from python/database module
use crate::db::get_repository;

/// Get validation report data for a schedule (Python binding).
///
/// This function fetches validation results from the database and returns
/// a structured report containing impossible blocks, validation errors, and warnings.
///
/// # Arguments
/// * `schedule_id` - Schedule ID to fetch validation results for
///
/// # Returns
/// * `crate::api::ValidationReport` - Validation report with all issues categorized
///
/// # Example (Python)
/// ```python
/// import tsi_rust
///
/// report = tsi_rust.py_get_validation_report(schedule_id=123)
/// print(f"Total blocks: {report.total_blocks}")
/// print(f"Impossible: {report.impossible_count}")
/// print(f"Errors: {report.errors_count}")
/// print(f"Warnings: {report.warnings_count}")
///
/// for issue in report.impossible_blocks:
///     print(f"Block {issue.block_id}: {issue.description}")
/// ```
// #[pyfunction] - removed, function now internal only
pub fn py_get_validation_report(schedule_id: crate::api::ScheduleId) -> PyResult<crate::api::ValidationReport> {
    // Get the initialized repository
    let repo = get_repository()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    // Create Tokio runtime to bridge async database operations
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    // Fetch validation results from repository
    let report_data = runtime
        .block_on(repo.fetch_validation_results(schedule_id))
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to fetch validation report: {}",
                e
            ))
        })?;

    Ok(report_data)
}
