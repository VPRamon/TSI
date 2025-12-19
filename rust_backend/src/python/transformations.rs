use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;

use crate::transformations::{cleaning, filtering};

/// Remove duplicate rows from a DataFrame
#[pyfunction]
#[pyo3(signature = (df, subset=None, keep="first"))]
pub fn py_remove_duplicates(
    df: PyDataFrame,
    subset: Option<Vec<String>>,
    keep: &str,
) -> PyResult<PyDataFrame> {
    let dataframe = &df.0;
    let result = cleaning::remove_duplicates(dataframe, subset, keep)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(PyDataFrame(result))
}

/// Remove rows with missing coordinates (RA or Dec)
#[pyfunction]
pub fn py_remove_missing_coordinates(df: PyDataFrame) -> PyResult<PyDataFrame> {
    let dataframe = &df.0;
    let result = cleaning::remove_missing_coordinates(dataframe)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(PyDataFrame(result))
}

/// Filter DataFrame by numeric range
#[pyfunction]
pub fn py_filter_by_range(
    df: PyDataFrame,
    column: &str,
    min_value: f64,
    max_value: f64,
) -> PyResult<PyDataFrame> {
    let dataframe = &df.0;
    let result = filtering::filter_by_range(dataframe, column, min_value, max_value)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(PyDataFrame(result))
}

/// Filter DataFrame by scheduled flag
#[pyfunction]
pub fn py_filter_by_scheduled(df: PyDataFrame, filter_type: &str) -> PyResult<PyDataFrame> {
    let dataframe = &df.0;
    let result = filtering::filter_by_scheduled(dataframe, filter_type)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(PyDataFrame(result))
}

/// Filter DataFrame by multiple conditions
#[pyfunction]
#[pyo3(signature = (df, priority_min, priority_max, scheduled_filter="All", priority_bins=None, block_ids=None))]
pub fn py_filter_dataframe(
    df: PyDataFrame,
    priority_min: f64,
    priority_max: f64,
    scheduled_filter: &str,
    priority_bins: Option<Vec<String>>,
    block_ids: Option<Vec<String>>,
) -> PyResult<PyDataFrame> {
    let dataframe = &df.0;
    let result = filtering::filter_dataframe(
        dataframe,
        priority_min,
        priority_max,
        scheduled_filter,
        priority_bins,
        block_ids,
    )
    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(PyDataFrame(result))
}

/// Validate DataFrame structure and data quality
#[pyfunction]
pub fn py_validate_dataframe(df: PyDataFrame) -> PyResult<(bool, Vec<String>)> {
    let dataframe = &df.0;
    Ok(filtering::validate_dataframe(dataframe))
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
