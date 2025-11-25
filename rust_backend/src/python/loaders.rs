use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;
use std::path::PathBuf;

use crate::io::loaders::{ScheduleLoader, DarkPeriodsLoader};

/// Load schedule data from a file (JSON or CSV)
/// 
/// Automatically detects the file format based on extension and returns
/// a Polars DataFrame that can be converted to pandas.
/// 
/// Args:
///     file_path: Path to the schedule file (.json or .csv)
/// 
/// Returns:
///     PyDataFrame: Polars DataFrame with schedule data
/// 
/// Example:
///     >>> import tsi_rust
///     >>> df = tsi_rust.load_schedule("data/schedule.json")
///     >>> pandas_df = df.to_pandas()
#[pyfunction]
pub fn load_schedule(file_path: &str) -> PyResult<PyDataFrame> {
    let path = PathBuf::from(file_path);
    
    let result = ScheduleLoader::load_from_file(&path)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to load schedule: {}", e)))?;
    
    Ok(PyDataFrame(result.dataframe))
}

/// Load schedule data from a JSON file
/// 
/// Args:
///     json_path: Path to the schedule.json file
/// 
/// Returns:
///     PyDataFrame: Polars DataFrame with schedule data
#[pyfunction]
pub fn load_schedule_from_json(json_path: &str) -> PyResult<PyDataFrame> {
    let path = PathBuf::from(json_path);
    
    let result = ScheduleLoader::load_from_json(&path)
        .map_err(|e| {
            // Format the full error chain for better debugging
            let error_msg = format!("Failed to load JSON from {}:\n{:?}", json_path, e);
            pyo3::exceptions::PyRuntimeError::new_err(error_msg)
        })?;
    
    Ok(PyDataFrame(result.dataframe))
}

/// Load schedule data from a JSON string
/// 
/// Args:
///     json_str: JSON string containing schedule data
/// 
/// Returns:
///     PyDataFrame: Polars DataFrame with schedule data
#[pyfunction]
pub fn load_schedule_from_json_str(json_str: &str) -> PyResult<PyDataFrame> {
    let result = ScheduleLoader::load_from_json_str(json_str)
        .map_err(|e| {
            // Format the full error chain for better debugging
            let error_msg = format!("Failed to parse JSON:\n{:?}", e);
            pyo3::exceptions::PyRuntimeError::new_err(error_msg)
        })?;
    
    Ok(PyDataFrame(result.dataframe))
}

/// Load schedule data from a CSV file
/// 
/// Args:
///     csv_path: Path to the schedule.csv file
/// 
/// Returns:
///     PyDataFrame: Polars DataFrame with schedule data
#[pyfunction]
pub fn load_schedule_from_csv(csv_path: &str) -> PyResult<PyDataFrame> {
    let path = PathBuf::from(csv_path);
    
    let result = ScheduleLoader::load_from_csv(&path)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to load CSV: {}", e)))?;
    
    Ok(PyDataFrame(result.dataframe))
}

/// Load dark periods from a JSON file
/// 
/// Supports flexible JSON formats with various key names (dark_periods, darkPeriods, etc.)
/// and value formats (MJD floats, strings, ISO timestamps, nested dicts).
/// 
/// Args:
///     file_path: Path to the dark_periods.json file
/// 
/// Returns:
///     PyDataFrame: Polars DataFrame with columns:
///         - start_dt: Start datetime (UTC)
///         - stop_dt: Stop datetime (UTC)
///         - start_mjd: Start Modified Julian Date
///         - stop_mjd: Stop Modified Julian Date
///         - duration_hours: Duration in hours
///         - months: List of months (YYYY-MM) touched by the period
/// 
/// Example:
///     >>> import tsi_rust
///     >>> df = tsi_rust.load_dark_periods("data/dark_periods.json")
///     >>> pandas_df = df.to_pandas()
#[pyfunction]
pub fn load_dark_periods(file_path: &str) -> PyResult<PyDataFrame> {
    let path = PathBuf::from(file_path);
    
    let dataframe = DarkPeriodsLoader::load_from_file(&path)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to load dark periods: {}", e)))?;
    
    Ok(PyDataFrame(dataframe))
}
