use chrono::{DateTime, Datelike, Timelike, TimeZone, Utc};
use pyo3::prelude::*;

use crate::core::domain::VisibilityPeriod;

const SECONDS_PER_DAY: f64 = 86_400.0;
const MJD_UNIX_EPOCH: f64 = 40_587.0; // Unix epoch (1970-01-01) in MJD

/// Convert Modified Julian Date to UTC DateTime
/// 
/// # Arguments
/// * `mjd` - Modified Julian Date value
/// 
/// # Returns
/// * `DateTime<Utc>` - UTC timestamp
/// 
/// # Example
/// ```
/// use tsi_rust::time::mjd_to_datetime;
/// let dt = mjd_to_datetime(59580.5);
/// ```
pub fn mjd_to_datetime_rust(mjd: f64) -> DateTime<Utc> {
    let seconds_since_epoch = (mjd - MJD_UNIX_EPOCH) * SECONDS_PER_DAY;
    let timestamp_secs = seconds_since_epoch.floor() as i64;
    let nanos = ((seconds_since_epoch - timestamp_secs as f64) * 1_000_000_000.0) as u32;
    
    Utc.timestamp_opt(timestamp_secs, nanos)
        .single()
        .expect("Invalid timestamp")
}

/// Convert UTC DateTime to Modified Julian Date
/// 
/// # Arguments
/// * `dt` - UTC DateTime
/// 
/// # Returns
/// * `f64` - Modified Julian Date value
pub fn datetime_to_mjd_rust(dt: &DateTime<Utc>) -> f64 {
    let timestamp = dt.timestamp() as f64 + (dt.timestamp_subsec_nanos() as f64 / 1_000_000_000.0);
    (timestamp / SECONDS_PER_DAY) + MJD_UNIX_EPOCH
}

/// Parse a Python list of visibility periods (as string) into VisibilityPeriod structs
/// 
/// Expected format: "[(start_mjd, stop_mjd), ...]"
/// where start_mjd and stop_mjd are MJD floats
/// 
/// # Arguments
/// * `visibility_str` - String representation of visibility periods
/// 
/// # Returns
/// * `Vec<VisibilityPeriod>` - Parsed visibility periods
pub fn parse_visibility_string(visibility_str: &str) -> Result<Vec<VisibilityPeriod>, String> {
    if visibility_str.trim().is_empty() || visibility_str == "[]" {
        return Ok(Vec::new());
    }
    
    // Parse string like "[(mjd1, mjd2), (mjd3, mjd4)]"
    let cleaned = visibility_str
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']');
    
    if cleaned.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut periods = Vec::new();
    let mut current_tuple = String::new();
    let mut paren_depth = 0;
    
    for ch in cleaned.chars() {
        match ch {
            '(' => {
                paren_depth += 1;
                if paren_depth == 1 {
                    current_tuple.clear();
                }
            }
            ')' => {
                paren_depth -= 1;
                if paren_depth == 0 && !current_tuple.is_empty() {
                    // Parse tuple (start, stop)
                    let parts: Vec<&str> = current_tuple.split(',').collect();
                    if parts.len() == 2 {
                        if let (Ok(start), Ok(stop)) = (
                            parts[0].trim().parse::<f64>(),
                            parts[1].trim().parse::<f64>(),
                        ) {
                            periods.push(VisibilityPeriod::new(
                                mjd_to_datetime_rust(start),
                                mjd_to_datetime_rust(stop),
                            ));
                        }
                    }
                }
            }
            _ => {
                if paren_depth > 0 {
                    current_tuple.push(ch);
                }
            }
        }
    }
    
    Ok(periods)
}

// ============ PyO3 Python Bindings ============

/// Convert Modified Julian Date to Python datetime (PyO3 binding)
#[pyfunction]
pub fn mjd_to_datetime(py: Python, mjd: f64) -> PyResult<PyObject> {
    let dt = mjd_to_datetime_rust(mjd);
    
    // Create Python datetime using direct construction
    let datetime_module = py.import("datetime")?;
    let datetime_cls = datetime_module.getattr("datetime")?;
    let timezone_cls = datetime_module.getattr("timezone")?;
    let utc = timezone_cls.getattr("utc")?;
    
    let timestamp = dt.timestamp() as f64 + (dt.timestamp_subsec_micros() as f64 / 1_000_000.0);
    
    let py_dt = datetime_cls.call_method1("fromtimestamp", (timestamp, utc))?;
    
    Ok(py_dt.to_object(py))
}

/// Convert Python datetime to Modified Julian Date (PyO3 binding)
#[pyfunction]
pub fn datetime_to_mjd(_py: Python, dt: &PyAny) -> PyResult<f64> {
    // Get timestamp from Python datetime
    let timestamp = dt.call_method0("timestamp")?.extract::<f64>()?;
    
    // Convert timestamp to MJD
    let mjd = (timestamp / SECONDS_PER_DAY) + MJD_UNIX_EPOCH;
    Ok(mjd)
}

/// Parse visibility periods from string (PyO3 binding)
/// 
/// Returns a list of tuples (start_datetime, stop_datetime)
#[pyfunction]
pub fn parse_visibility_periods(py: Python, visibility_str: &str) -> PyResult<PyObject> {
    let periods = parse_visibility_string(visibility_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
    
    let datetime_module = py.import("datetime")?;
    let datetime_cls = datetime_module.getattr("datetime")?;
    let timezone_cls = datetime_module.getattr("timezone")?;
    let utc = timezone_cls.getattr("utc")?;
    
    let py_list = pyo3::types::PyList::empty(py);
    for period in periods {
        // Convert start
        let start_timestamp = period.start.timestamp() as f64 + (period.start.timestamp_subsec_micros() as f64 / 1_000_000.0);
        let start_py = datetime_cls.call_method1("fromtimestamp", (start_timestamp, utc))?;
        
        // Convert stop
        let stop_timestamp = period.stop.timestamp() as f64 + (period.stop.timestamp_subsec_micros() as f64 / 1_000_000.0);
        let stop_py = datetime_cls.call_method1("fromtimestamp", (stop_timestamp, utc))?;
        
        let tuple = pyo3::types::PyTuple::new(py, &[start_py, stop_py]);
        py_list.append(tuple)?;
    }
    
    Ok(py_list.to_object(py))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mjd_roundtrip() {
        let original_mjd = 59580.5;
        let dt = mjd_to_datetime_rust(original_mjd);
        let back_to_mjd = datetime_to_mjd_rust(&dt);
        
        assert!((original_mjd - back_to_mjd).abs() < 1e-6);
    }
    
    #[test]
    fn test_known_mjd_conversion() {
        // MJD 0 = 1858-11-17 00:00:00 UTC
        // MJD 59580.0 = 2022-01-01 00:00:00 UTC (approximately)
        let mjd = 59580.0;
        let dt = mjd_to_datetime_rust(mjd);
        
        assert_eq!(dt.year(), 2022);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }
    
    #[test]
    fn test_parse_empty_visibility() {
        assert_eq!(parse_visibility_string("").unwrap().len(), 0);
        assert_eq!(parse_visibility_string("[]").unwrap().len(), 0);
        assert_eq!(parse_visibility_string("  []  ").unwrap().len(), 0);
    }
    
    #[test]
    fn test_parse_single_visibility_period() {
        let input = "[(59580.0, 59581.0)]";
        let periods = parse_visibility_string(input).unwrap();
        
        assert_eq!(periods.len(), 1);
        let period = &periods[0];
        assert_eq!(period.start.year(), 2022);
        assert_eq!(period.stop.year(), 2022);
        assert!((period.duration_hours() - 24.0).abs() < 0.1);
    }
    
    #[test]
    fn test_parse_multiple_visibility_periods() {
        let input = "[(59580.0, 59580.5), (59581.0, 59581.25)]";
        let periods = parse_visibility_string(input).unwrap();
        
        assert_eq!(periods.len(), 2);
        assert!((periods[0].duration_hours() - 12.0).abs() < 0.1);
        assert!((periods[1].duration_hours() - 6.0).abs() < 0.1);
    }
    
    #[test]
    fn test_precision() {
        let mjd = 59580.123456789;
        let dt = mjd_to_datetime_rust(mjd);
        let back = datetime_to_mjd_rust(&dt);
        
        // Should be accurate to microseconds
        assert!((mjd - back).abs() < 1e-9);
    }
}
