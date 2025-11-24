use pyo3::prelude::*;
use siderust::astro::ModifiedJulianDate;

use crate::core::domain::Period;

/// Convert Modified Julian Date to ModifiedJulianDate
/// 
/// # Arguments
/// * `mjd` - Modified Julian Date value
/// 
/// # Returns
/// * `ModifiedJulianDate` - ModifiedJulianDate in UTC
/// 
/// # Example
/// ```
/// use tsi_rust::time::mjd_to_epoch;
/// let epoch = mjd_to_epoch(59580.5);
/// ```
pub fn mjd_to_epoch(mjd: f64) -> ModifiedJulianDate {
    ModifiedJulianDate::new(mjd)
}

/// Convert ModifiedJulianDate to Modified Julian Date
/// 
/// # Arguments
/// * `epoch` - ModifiedJulianDate in UTC
/// 
/// # Returns
/// * `f64` - Modified Julian Date value
pub fn epoch_to_mjd(epoch: &ModifiedJulianDate) -> f64 {
    epoch.value()
}

/// Parse a Python list of visibility periods (as string) into Period structs
/// 
/// Expected format: "[(start_mjd, stop_mjd), ...]"
/// where start_mjd and stop_mjd are MJD floats
/// 
/// # Arguments
/// * `visibility_str` - String representation of visibility periods
/// 
/// # Returns
/// * `Vec<Period>` - Parsed visibility periods
pub fn parse_visibility_string(visibility_str: &str) -> Result<Vec<Period>, String> {
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
                            periods.push(Period::new(
                                mjd_to_epoch(start),
                                mjd_to_epoch(stop),
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
pub fn mjd_to_datetime(py: Python, mjd: f64) -> PyResult<Py<PyAny>> {
    use chrono::Datelike;
    use chrono::Timelike;
    
    let epoch = mjd_to_epoch(mjd);
    
    // Convert to Python datetime using chrono DateTime
    let datetime_utc = epoch.to_utc()
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid MJD value"))?;
    
    let datetime_module = py.import("datetime")?;
    let datetime_cls = datetime_module.getattr("datetime")?;
    let timezone_cls = datetime_module.getattr("timezone")?;
    let utc = timezone_cls.getattr("utc")?;
    
    let year = datetime_utc.year();
    let month = datetime_utc.month();
    let day = datetime_utc.day();
    let hour = datetime_utc.hour();
    let minute = datetime_utc.minute();
    let second = datetime_utc.second();
    let microsecond = datetime_utc.timestamp_subsec_micros();
    
    let py_dt = datetime_cls.call1((
        year,
        month,
        day,
        hour,
        minute,
        second,
        microsecond,
        utc,
    ))?;
    
    Ok(py_dt.unbind())
}

/// Convert Python datetime to Modified Julian Date (PyO3 binding)
#[pyfunction]
pub fn datetime_to_mjd(dt: &Bound<'_, PyAny>) -> PyResult<f64> {
    use chrono::prelude::*;
    
    // Get datetime components from Python
    let year = dt.getattr("year")?.extract::<i32>()?;
    let month = dt.getattr("month")?.extract::<u32>()?;
    let day = dt.getattr("day")?.extract::<u32>()?;
    let hour = dt.getattr("hour")?.extract::<u32>()?;
    let minute = dt.getattr("minute")?.extract::<u32>()?;
    let second = dt.getattr("second")?.extract::<u32>()?;
    let microsecond = dt.getattr("microsecond")?.extract::<u32>()?;
    
    // Create chrono DateTime
    let datetime_utc = Utc
        .with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid datetime"))?
        .checked_add_signed(chrono::Duration::microseconds(microsecond as i64))
        .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid datetime"))?;
    
    // Create epoch and convert to MJD
    let epoch = ModifiedJulianDate::from_utc(datetime_utc);
    Ok(epoch_to_mjd(&epoch))
}

/// Parse visibility periods from string (PyO3 binding)
/// 
/// Returns a list of tuples (start_datetime, stop_datetime)
#[pyfunction]
pub fn parse_visibility_periods(py: Python, visibility_str: &str) -> PyResult<Py<PyAny>> {
    use chrono::Datelike;
    use chrono::Timelike;
    
    let periods = parse_visibility_string(visibility_str)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
    
    let datetime_module = py.import("datetime")?;
    let datetime_cls = datetime_module.getattr("datetime")?;
    let timezone_cls = datetime_module.getattr("timezone")?;
    let utc = timezone_cls.getattr("utc")?;
    
    let py_list = pyo3::types::PyList::empty(py);
    for period in periods {
        // Convert start using to_utc()
        let start_utc = period.start.to_utc()
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid start MJD"))?;
        let start_py = datetime_cls.call1((
            start_utc.year(),
            start_utc.month(),
            start_utc.day(),
            start_utc.hour(),
            start_utc.minute(),
            start_utc.second(),
            start_utc.timestamp_subsec_micros(),
            &utc,
        ))?;
        
        // Convert stop using to_utc()
        let stop_utc = period.stop.to_utc()
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid stop MJD"))?;
        let stop_py = datetime_cls.call1((
            stop_utc.year(),
            stop_utc.month(),
            stop_utc.day(),
            stop_utc.hour(),
            stop_utc.minute(),
            stop_utc.second(),
            stop_utc.timestamp_subsec_micros(),
            &utc,
        ))?;
        
        let tuple = pyo3::types::PyTuple::new(py, vec![start_py, stop_py])?;
        py_list.append(tuple)?;
    }
    
    Ok(py_list.unbind().into())
}

#[cfg(all(test, not(feature = "extension-module")))]
mod tests {
    use super::*;
    
    #[test]
    fn test_mjd_roundtrip() {
        let original_mjd = 59580.5;
        let epoch = mjd_to_epoch(original_mjd);
        let back_to_mjd = epoch_to_mjd(&epoch);
        
        assert!((original_mjd - back_to_mjd).abs() < 1e-6);
    }
    
    #[test]
    fn test_known_mjd_conversion() {
        use chrono::Datelike;
        
        // MJD 0 = 1858-11-17 00:00:00 UTC
        // MJD 59580.0 = 2022-01-01 00:00:00 UTC (approximately)
        let mjd = 59580.0;
        let epoch = mjd_to_epoch(mjd);
        let datetime = epoch.to_utc().unwrap();
        
        assert_eq!(datetime.year(), 2022);
        assert_eq!(datetime.month(), 1);
        assert_eq!(datetime.day(), 1);
    }
    
    #[test]
    fn test_parse_empty_visibility() {
        assert_eq!(parse_visibility_string("").unwrap().len(), 0);
        assert_eq!(parse_visibility_string("[]").unwrap().len(), 0);
        assert_eq!(parse_visibility_string("  []  ").unwrap().len(), 0);
    }
    
    #[test]
    fn test_parse_single_visibility_period() {
        use chrono::Datelike;
        
        let input = "[(59580.0, 59581.0)]";
        let periods = parse_visibility_string(input).unwrap();
        
        assert_eq!(periods.len(), 1);
        let period = &periods[0];
        let datetime = period.start.to_utc().unwrap();
        assert_eq!(datetime.year(), 2022);
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
        let epoch = mjd_to_epoch(mjd);
        let back = epoch_to_mjd(&epoch);
        
        // Should be accurate to microseconds
        assert!((mjd - back).abs() < 1e-9);
    }
}
