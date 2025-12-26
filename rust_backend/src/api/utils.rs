//! Streamlit API Functions.
//!
//! This module contains all `#[pyfunction]` exports for the Streamlit Python application.
//! Each function acts as a thin wrapper around internal service/repository calls,
//! converting between API DTOs and internal models at the boundary.
//!
//! ## Design Patterns
//!
//! 1. Accept API DTOs or primitives as parameters
//! 2. Convert to internal types using conversion traits
//! 3. Call internal service/repository methods
//! 4. Convert results back to API DTOs
//! 5. Return to Python with proper error handling

use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde_json::Value;

use crate::api::types as api;
// Re-export landing route functions so they can be registered with the Python module
pub use crate::routes::landing::{list_schedules, store_schedule};
// Re-export route name constants so Python can reference them without hard-coded strings
pub use crate::routes::landing::{LIST_SCHEDULES, POST_SCHEDULE};
// Re-export validation route so it can be registered from routes module
pub use crate::routes::validation::{get_validation_report};
pub use crate::routes::validation::GET_VALIDATION_REPORT;
// Re-export visualization route so it can be registered from routes module
pub use crate::routes::skymap::{get_sky_map_data};
pub use crate::routes::skymap::GET_SKY_MAP_DATA;
// Re-export visibility route and constant
pub use crate::routes::visibility::{get_visibility_map_data};
pub use crate::routes::visibility::GET_VISIBILITY_MAP_DATA;
// Re-export distribution route and constant
pub use crate::routes::distribution::{get_distribution_data};
pub use crate::routes::distribution::GET_DISTRIBUTION_DATA;
// Re-export timeline route and constant
pub use crate::routes::timeline::{get_schedule_timeline_data};
pub use crate::routes::timeline::GET_SCHEDULE_TIMELINE_DATA;
// Re-export insights route and constant
pub use crate::routes::insights::{get_insights_data};
pub use crate::routes::insights::GET_INSIGHTS_DATA;
// Re-export trends route and constant
pub use crate::routes::trends::{get_trends_data};
pub use crate::routes::trends::GET_TRENDS_DATA;
// Re-export compare route and constant
pub use crate::routes::compare::{get_compare_data};
pub use crate::routes::compare::GET_COMPARE_DATA;

/// Register all API functions with the Python module.
///
/// This function is called from lib.rs to populate the tsi_rust_api module
/// with all exported functions and classes.
pub fn register_api_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Database initialization (initialization is now lazy; explicit init function removed)

    // Analytics ETL operations (internalized)

    // Route-specific functions, classes and constants are registered centrally by `routes`
    crate::routes::register_route_functions(m)?;

    // Legacy visibility functions are registered by `routes::visibility`


    // Register all API classes
    m.add_class::<api::Period>()?;
    m.add_class::<api::Constraints>()?;
    m.add_class::<api::SchedulingBlock>()?;
    m.add_class::<api::Schedule>()?;

    Ok(())
}


// =========================================================
// Database Operations
// =========================================================

// Repository initialization is handled lazily by `db::get_repository()`;
// the older explicit `init_database` Python binding has been removed.


// =========================================================
// Analytics ETL Operations
// =========================================================

// Analytics population is internal to the Rust backend; there is no exported
// `populate_analytics` Python binding anymore. Rust routes/services will ensure
// analytics are populated as needed.
// Analytics availability checks are internal; no exported `has_analytics_data` binding.

// =========================================================
// Validation Operations
// =========================================================
// Validation operations are provided by `routes::validation`
// =========================================================
// Transformation Functions (Legacy API)
// =========================================================

/// Register transformation functions for backwards compatibility with tsi_rust module.
pub fn register_transformation_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // `py_filter_dataframe` removed — transformation helpers live in Python now.
    m.add_function(wrap_pyfunction!(py_remove_duplicates, m)?)?;
    m.add_function(wrap_pyfunction!(py_remove_missing_coordinates, m)?)?;
    m.add_function(wrap_pyfunction!(py_validate_dataframe, m)?)?;
    m.add_function(wrap_pyfunction!(mjd_to_datetime, m)?)?;
    m.add_function(wrap_pyfunction!(datetime_to_mjd, m)?)?;
    m.add_function(wrap_pyfunction!(parse_visibility_periods, m)?)?;
    Ok(())
}



/// Convert Modified Julian Date to Python datetime (UTC).
#[pyfunction]
fn mjd_to_datetime(mjd: f64) -> PyResult<Py<PyAny>> {
    Python::attach(|py| {
        let secs = (mjd - 40587.0) * 86400.0;
        let datetime_mod = py.import("datetime")?;
        let datetime_cls = datetime_mod.getattr("datetime")?;
        let timezone_utc = datetime_mod.getattr("timezone")?.getattr("utc")?;
        let dt = datetime_cls.call_method1("fromtimestamp", (secs, timezone_utc))?;
        Ok(dt.into())
    })
}

/// Convert Python datetime to Modified Julian Date (assumes UTC for naive datetimes).
#[pyfunction]
fn datetime_to_mjd(dt: Py<PyAny>) -> PyResult<f64> {
    Python::attach(|py| {
        let datetime_mod = py.import("datetime")?;
        let timezone_utc = datetime_mod.getattr("timezone")?.getattr("utc")?;
        let dt_obj = dt.bind(py);
        let tzinfo = dt_obj.getattr("tzinfo")?;

        let timestamp = if tzinfo.is_none() {
            let kwargs = PyDict::new(py);
            kwargs.set_item("tzinfo", &timezone_utc)?;
            let aware = dt_obj.call_method("replace", (), Some(&kwargs))?;
            aware.call_method0("timestamp")?.extract::<f64>()?
        } else {
            dt_obj.call_method0("timestamp")?.extract::<f64>()?
        };

        Ok(timestamp / 86400.0 + 40587.0)
    })
}

// `py_filter_dataframe` intentionally removed: filtering helpers are now
// implemented in the Python layer (`src/tsi/backend/utils.py`).

/// Remove duplicate rows from DataFrame.
#[pyfunction]
#[pyo3(signature = (json_str, subset=None, keep=None))]
fn py_remove_duplicates(
    json_str: String,
    subset: Option<Vec<String>>,
    keep: Option<String>,
) -> PyResult<String> {
    let keep = keep.unwrap_or_else(|| "first".to_string());
    let records: Vec<Value> = serde_json::from_str(&json_str).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e))
    })?;
    
    let mut unique_records = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();
    
    for record in records.iter() {
        // Generate key from subset columns or entire record
        let key = if let Some(ref cols) = subset {
            let mut key_parts = Vec::new();
            for col in cols {
                if let Some(val) = record.get(col) {
                    key_parts.push(val.to_string());
                }
            }
            key_parts.join("|")
        } else {
            record.to_string()
        };
        
        match keep.as_str() {
            "first" => {
                if !seen_keys.contains(&key) {
                    seen_keys.insert(key);
                    unique_records.push(record.clone());
                }
            }
            "last" => {
                seen_keys.insert(key.clone());
                unique_records.retain(|r| {
                    let r_key = if let Some(ref cols) = subset {
                        let mut key_parts = Vec::new();
                        for col in cols {
                            if let Some(val) = r.get(col) {
                                key_parts.push(val.to_string());
                            }
                        }
                        key_parts.join("|")
                    } else {
                        r.to_string()
                    };
                    r_key != key
                });
                unique_records.push(record.clone());
            }
            "none" => {
                if !seen_keys.contains(&key) {
                    seen_keys.insert(key);
                    unique_records.push(record.clone());
                }
            }
            _ => {
                if !seen_keys.contains(&key) {
                    seen_keys.insert(key);
                    unique_records.push(record.clone());
                }
            }
        }
    }
    
    serde_json::to_string(&unique_records).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Serialization failed: {}", e))
    })
}

/// Remove observations with missing RA or Dec coordinates.
#[pyfunction]
fn py_remove_missing_coordinates(json_str: String) -> PyResult<String> {
    let records: Vec<Value> = serde_json::from_str(&json_str).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e))
    })?;
    
    let filtered: Vec<Value> = records
        .into_iter()
        .filter(|record| {
            let has_ra = record.get("raDeg").and_then(|v| v.as_f64()).is_some();
            let has_dec = record.get("decDeg").and_then(|v| v.as_f64()).is_some();
            has_ra && has_dec
        })
        .collect();
    
    serde_json::to_string(&filtered).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Serialization failed: {}", e))
    })
}

/// Validate DataFrame data quality.
#[pyfunction]
fn py_validate_dataframe(json_str: String) -> PyResult<(bool, Vec<String>)> {
    let records: Vec<Value> = serde_json::from_str(&json_str).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse JSON: {}", e))
    })?;
    
    let mut issues = Vec::new();
    
    // Check for missing coordinates
    let missing_coords = records.iter().filter(|r| {
        let has_ra = r.get("raDeg").and_then(|v| v.as_f64()).is_some();
        let has_dec = r.get("decDeg").and_then(|v| v.as_f64()).is_some();
        !has_ra || !has_dec
    }).count();
    
    if missing_coords > 0 {
        issues.push(format!("{} observations with missing coordinates", missing_coords));
    }
    
    // Check for invalid priorities
    let invalid_priorities = records.iter().filter(|r| {
        if let Some(p) = r.get("priority").and_then(|v| v.as_f64()) {
            p < 0.0 || p > 100.0
        } else {
            true
        }
    }).count();
    
    if invalid_priorities > 0 {
        issues.push(format!("{} observations with invalid priorities", invalid_priorities));
    }
    
    let is_valid = issues.is_empty();
    Ok((is_valid, issues))
}

/// Parse visibility periods from list of dicts to datetime tuples.
#[pyfunction]
fn parse_visibility_periods(periods: Vec<(String, String)>) -> PyResult<Vec<(String, String)>> {
    // Simply return as-is since Python will handle datetime parsing
    // This is a no-op shim for backwards compatibility
    Ok(periods)
}
