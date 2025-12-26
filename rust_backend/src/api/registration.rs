//! API registration helpers.
//!
//! This module contains the functions that register Python-callable
//! functions and the minimal transformation shims that remain in Rust.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde_json::Value;

use crate::api::types as api;

// Re-export route registration — routes register their own Python functions
// when `register_route_functions` is called.

/// Register all API functions with the Python module.
pub fn register_api_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Route-specific functions, classes and constants are registered centrally by `routes`
    crate::routes::register_route_functions(m)?;

    // Register all API classes
    m.add_class::<api::Period>()?;
    m.add_class::<api::Constraints>()?;
    m.add_class::<api::SchedulingBlock>()?;
    m.add_class::<api::Schedule>()?;

    Ok(())
}


/// Register minimal transformation shims expected by legacy Python code.
pub fn register_transformation_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Transformation shims removed — functionality lives in Python now.
    Ok(())
}


// `mjd_to_datetime` and `datetime_to_mjd` removed — implemented in Python.

// `parse_visibility_periods` removed — implemented in Python.
