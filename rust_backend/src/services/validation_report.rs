//! Python bindings for validation report functionality.
//!
//! This module exposes validation report data to Python via PyO3.

use pyo3::prelude::*;
use tokio::runtime::Runtime;

use crate::db::validation::{fetch_validation_results, ValidationIssue, ValidationReportData};

/// A single validation issue exposed to Python
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct PyValidationIssue {
    #[pyo3(get)]
    pub block_id: i64,
    #[pyo3(get)]
    pub issue_type: String,
    #[pyo3(get)]
    pub category: String,
    #[pyo3(get)]
    pub criticality: String,
    #[pyo3(get)]
    pub field_name: Option<String>,
    #[pyo3(get)]
    pub current_value: Option<String>,
    #[pyo3(get)]
    pub expected_value: Option<String>,
    #[pyo3(get)]
    pub description: String,
}

impl From<ValidationIssue> for PyValidationIssue {
    fn from(issue: ValidationIssue) -> Self {
        Self {
            block_id: issue.block_id,
            issue_type: issue.issue_type,
            category: issue.category,
            criticality: issue.criticality,
            field_name: issue.field_name,
            current_value: issue.current_value,
            expected_value: issue.expected_value,
            description: issue.description,
        }
    }
}

/// Validation report data exposed to Python
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct PyValidationReportData {
    #[pyo3(get)]
    pub schedule_id: i64,
    #[pyo3(get)]
    pub total_blocks: usize,
    #[pyo3(get)]
    pub valid_blocks: usize,
    #[pyo3(get)]
    pub impossible_blocks: Vec<PyValidationIssue>,
    #[pyo3(get)]
    pub validation_errors: Vec<PyValidationIssue>,
    #[pyo3(get)]
    pub validation_warnings: Vec<PyValidationIssue>,
}

#[pymethods]
impl PyValidationReportData {
    /// Get the number of impossible blocks
    #[getter]
    pub fn impossible_count(&self) -> usize {
        self.impossible_blocks.len()
    }

    /// Get the number of validation errors
    #[getter]
    pub fn errors_count(&self) -> usize {
        self.validation_errors.len()
    }

    /// Get the number of validation warnings
    #[getter]
    pub fn warnings_count(&self) -> usize {
        self.validation_warnings.len()
    }

    /// Get the total number of issues (impossible + errors + warnings)
    #[getter]
    pub fn total_issues(&self) -> usize {
        self.impossible_blocks.len() + self.validation_errors.len() + self.validation_warnings.len()
    }

    fn __repr__(&self) -> String {
        format!(
            "ValidationReportData(schedule_id={}, total_blocks={}, valid={}, impossible={}, errors={}, warnings={})",
            self.schedule_id,
            self.total_blocks,
            self.valid_blocks,
            self.impossible_blocks.len(),
            self.validation_errors.len(),
            self.validation_warnings.len()
        )
    }
}

impl From<ValidationReportData> for PyValidationReportData {
    fn from(data: ValidationReportData) -> Self {
        Self {
            schedule_id: data.schedule_id,
            total_blocks: data.total_blocks,
            valid_blocks: data.valid_blocks,
            impossible_blocks: data
                .impossible_blocks
                .into_iter()
                .map(PyValidationIssue::from)
                .collect(),
            validation_errors: data
                .validation_errors
                .into_iter()
                .map(PyValidationIssue::from)
                .collect(),
            validation_warnings: data
                .validation_warnings
                .into_iter()
                .map(PyValidationIssue::from)
                .collect(),
        }
    }
}

/// Get validation report data for a schedule (Python binding).
///
/// This function fetches validation results from the database and returns
/// a structured report containing impossible blocks, validation errors, and warnings.
///
/// # Arguments
/// * `schedule_id` - Schedule ID to fetch validation results for
///
/// # Returns
/// * `PyValidationReportData` - Validation report with all issues categorized
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
#[pyfunction]
pub fn py_get_validation_report(schedule_id: i64) -> PyResult<PyValidationReportData> {
    // Create Tokio runtime to bridge async database operations
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    // Fetch validation results from database
    let report_data = runtime
        .block_on(fetch_validation_results(schedule_id))
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to fetch validation report: {}",
                e
            ))
        })?;

    Ok(PyValidationReportData::from(report_data))
}
