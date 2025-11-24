use pyo3::prelude::*;
use pyo3_polars::PyDataFrame;
use std::path::PathBuf;

use crate::io::loaders::ScheduleLoader;

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
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to load JSON: {}", e)))?;
    
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
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to parse JSON: {}", e)))?;
    
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
