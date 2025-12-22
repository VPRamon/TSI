use pyo3::prelude::*;
use serde_json::Value;

use crate::transformations::{cleaning, filtering};

/// Remove duplicate rows from records
#[pyfunction]
#[pyo3(signature = (records_json, subset=None, keep="first"))]
pub fn py_remove_duplicates(
    records_json: String,
    subset: Option<Vec<String>>,
    keep: &str,
) -> PyResult<String> {
    let records: Vec<Value> = serde_json::from_str(&records_json).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse JSON: {}", e))
    })?;

    let result = cleaning::remove_duplicates(&records, subset, keep)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    serde_json::to_string(&result).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to serialize result: {}", e))
    })
}

/// Remove rows with missing coordinates (RA or Dec)
#[pyfunction]
pub fn py_remove_missing_coordinates(records_json: String) -> PyResult<String> {
    let records: Vec<Value> = serde_json::from_str(&records_json).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse JSON: {}", e))
    })?;

    let result = cleaning::remove_missing_coordinates(&records)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    serde_json::to_string(&result).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to serialize result: {}", e))
    })
}

/// Filter records by numeric range
#[pyfunction]
pub fn py_filter_by_range(
    records_json: String,
    column: &str,
    min_value: f64,
    max_value: f64,
) -> PyResult<String> {
    let records: Vec<Value> = serde_json::from_str(&records_json).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse JSON: {}", e))
    })?;

    let result = filtering::filter_by_range(&records, column, min_value, max_value)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    serde_json::to_string(&result).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to serialize result: {}", e))
    })
}

/// Filter records by scheduled flag
#[pyfunction]
pub fn py_filter_by_scheduled(records_json: String, filter_type: &str) -> PyResult<String> {
    let records: Vec<Value> = serde_json::from_str(&records_json).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse JSON: {}", e))
    })?;

    let result = filtering::filter_by_scheduled(&records, filter_type)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    serde_json::to_string(&result).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to serialize result: {}", e))
    })
}

/// Filter records by multiple conditions
#[pyfunction]
#[pyo3(signature = (records_json, priority_min, priority_max, scheduled_filter="All", priority_bins=None, block_ids=None))]
pub fn py_filter_dataframe(
    records_json: String,
    priority_min: f64,
    priority_max: f64,
    scheduled_filter: &str,
    priority_bins: Option<Vec<String>>,
    block_ids: Option<Vec<String>>,
) -> PyResult<String> {
    let records: Vec<Value> = serde_json::from_str(&records_json).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse JSON: {}", e))
    })?;

    let result = filtering::filter_dataframe(
        &records,
        priority_min,
        priority_max,
        scheduled_filter,
        priority_bins,
        block_ids,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;

    serde_json::to_string(&result).map_err(|e| {
        pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to serialize result: {}", e))
    })
}

/// Validate records structure and data quality
#[pyfunction]
pub fn py_validate_dataframe(records_json: String) -> PyResult<(bool, Vec<String>)> {
    let records: Vec<Value> = serde_json::from_str(&records_json).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Failed to parse JSON: {}", e))
    })?;

    Ok(filtering::validate_dataframe(&records))
}

pub fn register_transformation_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_remove_duplicates, m)?)?;
    m.add_function(wrap_pyfunction!(py_remove_missing_coordinates, m)?)?;
    m.add_function(wrap_pyfunction!(py_filter_by_range, m)?)?;
    m.add_function(wrap_pyfunction!(py_filter_by_scheduled, m)?)?;
    m.add_function(wrap_pyfunction!(py_filter_dataframe, m)?)?;
    m.add_function(wrap_pyfunction!(py_validate_dataframe, m)?)?;
    Ok(())
}
