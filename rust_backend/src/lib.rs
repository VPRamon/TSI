use pyo3::prelude::*;

pub mod core;
pub mod parsing;
pub mod preprocessing;
pub mod time;
pub mod algorithms;
pub mod transformations;
pub mod io;
pub mod python;

/// TSI Rust Backend - High-performance telescope scheduling analysis
#[pymodule]
fn tsi_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    // Register time conversion functions
    m.add_function(wrap_pyfunction!(time::mjd_to_datetime, m)?)?;
    m.add_function(wrap_pyfunction!(time::datetime_to_mjd, m)?)?;
    m.add_function(wrap_pyfunction!(time::parse_visibility_periods, m)?)?;
    
    // Register data loading functions
    m.add_function(wrap_pyfunction!(python::load_schedule, m)?)?;
    m.add_function(wrap_pyfunction!(python::load_schedule_from_json, m)?)?;
    m.add_function(wrap_pyfunction!(python::load_schedule_from_json_str, m)?)?;
    m.add_function(wrap_pyfunction!(python::load_schedule_from_csv, m)?)?;
    
    Ok(())
}
