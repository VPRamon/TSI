use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_polars::PyDataFrame;
use std::path::PathBuf;

use crate::preprocessing::{preprocess_schedule, PreprocessConfig, PreprocessPipeline, ValidationStats};

/// Python wrapper for ValidationResult
#[pyclass]
#[derive(Clone)]
pub struct PyValidationResult {
    #[pyo3(get)]
    pub is_valid: bool,
    #[pyo3(get)]
    pub errors: Vec<String>,
    #[pyo3(get)]
    pub warnings: Vec<String>,
    // Store stats internally
    stats: ValidationStats,
}

#[pymethods]
impl PyValidationResult {
    fn __repr__(&self) -> String {
        format!(
            "ValidationResult(valid={}, errors={}, warnings={})",
            self.is_valid,
            self.errors.len(),
            self.warnings.len()
        )
    }

    /// Get statistics as a Python dict
    fn get_stats(&self, py: Python) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("total_blocks", self.stats.total_blocks)?;
        dict.set_item("scheduled_blocks", self.stats.scheduled_blocks)?;
        dict.set_item("unscheduled_blocks", self.stats.unscheduled_blocks)?;
        dict.set_item("blocks_with_visibility", self.stats.blocks_with_visibility)?;
        dict.set_item("avg_visibility_periods", self.stats.avg_visibility_periods)?;
        dict.set_item("avg_visibility_hours", self.stats.avg_visibility_hours)?;
        dict.set_item("missing_coordinates", self.stats.missing_coordinates)?;
        dict.set_item("missing_constraints", self.stats.missing_constraints)?;
        dict.set_item("duplicate_ids", self.stats.duplicate_ids)?;
        dict.set_item("invalid_priorities", self.stats.invalid_priorities)?;
        dict.set_item("invalid_durations", self.stats.invalid_durations)?;
        Ok(dict.into())
    }
}

/// Preprocess a schedule file with optional visibility enrichment
///
/// Args:
///     schedule_path: Path to schedule.json or schedule.csv
///     visibility_path: Optional path to possible_periods.json
///     validate: Whether to validate the data (default: True)
///
/// Returns:
///     tuple: (PyDataFrame, PyValidationResult)
///
/// Example:
///     >>> df, validation = tsi_rust.preprocess_schedule(
///     ...     "data/schedule.json",
///     ...     "data/possible_periods.json",
///     ...     validate=True
///     ... )
///     >>> print(f"Loaded {len(df)} blocks")
///     >>> print(f"Valid: {validation.is_valid}")
#[pyfunction]
#[pyo3(signature = (schedule_path, visibility_path=None, validate=true))]
pub fn py_preprocess_schedule(
    schedule_path: &str,
    visibility_path: Option<&str>,
    validate: bool,
) -> PyResult<(PyDataFrame, PyValidationResult)> {
    let schedule_path_buf = PathBuf::from(schedule_path);
    let visibility_path_buf = visibility_path.map(PathBuf::from);

    let result = preprocess_schedule(&schedule_path_buf, visibility_path_buf.as_deref(), validate)
        .map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Preprocessing failed: {}", e))
        })?;

    let py_validation = PyValidationResult {
        is_valid: result.validation.is_valid,
        errors: result.validation.errors,
        warnings: result.validation.warnings,
        stats: result.validation.stats,
    };

    Ok((PyDataFrame(result.dataframe), py_validation))
}

/// Preprocess schedule from JSON string
///
/// Args:
///     schedule_json: JSON string containing schedule data
///     visibility_json: Optional JSON string with visibility data
///     validate: Whether to validate the data (default: True)
///
/// Returns:
///     tuple: (PyDataFrame, PyValidationResult)
#[pyfunction]
#[pyo3(signature = (schedule_json, visibility_json=None, validate=true))]
pub fn py_preprocess_schedule_str(
    schedule_json: &str,
    visibility_json: Option<&str>,
    validate: bool,
) -> PyResult<(PyDataFrame, PyValidationResult)> {
    let config = PreprocessConfig {
        validate,
        enrich_visibility: visibility_json.is_some(),
    };

    let pipeline = PreprocessPipeline::with_config(config);
    let result = pipeline
        .process_json_str(schedule_json, visibility_json)
        .map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Preprocessing failed: {}", e))
        })?;

    let py_validation = PyValidationResult {
        is_valid: result.validation.is_valid,
        errors: result.validation.errors,
        warnings: result.validation.warnings,
        stats: result.validation.stats,
    };

    Ok((PyDataFrame(result.dataframe), py_validation))
}

/// Validate an existing DataFrame
///
/// Args:
///     df: PyDataFrame to validate
///
/// Returns:
///     PyValidationResult
#[pyfunction]
pub fn py_validate_schedule(df: PyDataFrame) -> PyResult<PyValidationResult> {
    use crate::preprocessing::ScheduleValidator;

    let validation = ScheduleValidator::validate_dataframe(&df.0);

    Ok(PyValidationResult {
        is_valid: validation.is_valid,
        errors: validation.errors,
        warnings: validation.warnings,
        stats: validation.stats,
    })
}
