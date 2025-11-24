use pyo3::prelude::*;

pub mod core;
pub mod parsing;
pub mod preprocessing;
pub mod algorithms;
pub mod transformations;
pub mod io;
pub mod python;

/// TSI Rust Backend - High-performance telescope scheduling analysis
#[pymodule]
fn tsi_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register time conversion functions
    m.add_function(wrap_pyfunction!(python::mjd_to_datetime, m)?)?;
    m.add_function(wrap_pyfunction!(python::datetime_to_mjd, m)?)?;
    m.add_function(wrap_pyfunction!(python::parse_visibility_periods, m)?)?;
    
    // Register data loading functions
    m.add_function(wrap_pyfunction!(python::load_schedule, m)?)?;
    m.add_function(wrap_pyfunction!(python::load_schedule_from_json, m)?)?;
    m.add_function(wrap_pyfunction!(python::load_schedule_from_json_str, m)?)?;
    m.add_function(wrap_pyfunction!(python::load_schedule_from_csv, m)?)?;
    m.add_function(wrap_pyfunction!(python::load_dark_periods, m)?)?;
    
    // Register preprocessing functions
    m.add_function(wrap_pyfunction!(python::py_preprocess_schedule, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_preprocess_schedule_str, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_validate_schedule, m)?)?;
    
    // Register algorithm functions
    m.add_function(wrap_pyfunction!(python::py_compute_metrics, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_compute_correlations, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_get_top_observations, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_find_conflicts, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_greedy_schedule, m)?)?;
    
    // Register transformation functions
    python::transformations::register_transformation_functions(m)?;
    
    // Register classes
    m.add_class::<python::PyValidationResult>()?;
    m.add_class::<python::PyAnalyticsSnapshot>()?;
    m.add_class::<python::PySchedulingConflict>()?;
    m.add_class::<python::PyOptimizationResult>()?;
    
    Ok(())
}
