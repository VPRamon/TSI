use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use polars::prelude::*;
use serde_json::Value;
use siderust::astro::ModifiedJulianDate;
use std::path::Path;

use crate::core::domain::Period;

/// Candidate keys that may contain the list of dark periods in the JSON payload
const PERIOD_KEYS: &[&str] = &[
    "dark_periods",
    "darkPeriods",
    "dark_period",
    "darkPeriod",
    "periods",
    "DarkPeriods",
];

/// Candidate keys for start timestamps inside each period
const START_KEYS: &[&str] = &[
    "start",
    "startMjd",
    "start_mjd",
    "startTime",
    "start_time",
    "startTimeUtc",
    "startUTC",
    "startUtc",
];

/// Candidate keys for stop timestamps inside each period
const STOP_KEYS: &[&str] = &[
    "stop",
    "stopMjd",
    "stop_mjd",
    "end",
    "endMjd",
    "end_mjd",
    "stopTime",
    "stop_time",
    "stopTimeUtc",
    "stopUTC",
    "stopUtc",
    "endTime",
    "end_time",
];

/// Enumerate all months (YYYY-MM format) touched by a period
fn enumerate_months(period: &Period) -> Vec<String> {
    let mut months = Vec::new();
    
    let start_dt = period.start.to_utc().expect("Valid start date");
    let stop_dt = period.stop.to_utc().expect("Valid stop date");
    
    let mut current = NaiveDate::from_ymd_opt(
        start_dt.year(),
        start_dt.month(),
        1
    ).expect("Valid date");
    
    let end_month = NaiveDate::from_ymd_opt(
        stop_dt.year(),
        stop_dt.month(),
        1
    ).expect("Valid date");
    
    while current <= end_month {
        months.push(format!("{:04}-{:02}", current.year(), current.month()));
        
        // Advance one month
        current = if current.month() == 12 {
            NaiveDate::from_ymd_opt(current.year() + 1, 1, 1).expect("Valid date")
        } else {
            NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1).expect("Valid date")
        };
    }
    
    months
}

/// Parse dark periods from JSON file
pub fn parse_dark_periods_file(path: &Path) -> Result<Vec<Period>> {
    let json_content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read dark periods file: {}", path.display()))?;
    
    parse_dark_periods_str(&json_content)
}

/// Parse dark periods from JSON string
pub fn parse_dark_periods_str(json_str: &str) -> Result<Vec<Period>> {
    let value: Value = serde_json::from_str(json_str)
        .context("Failed to parse dark periods JSON")?;
    
    extract_periods(&value)
}

/// Extract periods from a JSON value
fn extract_periods(payload: &Value) -> Result<Vec<Period>> {
    let raw_periods = find_periods_array(payload)
        .context("Could not find dark periods array in JSON")?;
    
    let mut periods = Vec::new();
    
    for period_value in raw_periods {
        if let Some(period) = parse_period(period_value) {
            // Validate: stop must be after start
            if period.stop.value() > period.start.value() {
                periods.push(period);
            }
        }
    }
    
    Ok(periods)
}

/// Find the array of periods in the JSON payload
fn find_periods_array(payload: &Value) -> Option<&Vec<Value>> {
    // If payload is an object, search for known keys
    if let Some(obj) = payload.as_object() {
        // Try known period keys first
        for key in PERIOD_KEYS {
            if let Some(value) = obj.get(*key) {
                if let Some(arr) = value.as_array() {
                    return Some(arr);
                }
            }
        }
        
        // Fallback: find the first array value in the object
        for value in obj.values() {
            if let Some(arr) = value.as_array() {
                return Some(arr);
            }
        }
    }
    
    // If payload is already an array
    if let Some(arr) = payload.as_array() {
        return Some(arr);
    }
    
    None
}

/// Parse a single period from JSON
fn parse_period(period: &Value) -> Option<Period> {
    let (start_mjd, stop_mjd) = if let Some(obj) = period.as_object() {
        // Object format: {"start": ..., "stop": ...}
        let start_value = find_value_by_keys(obj, START_KEYS)?;
        let stop_value = find_value_by_keys(obj, STOP_KEYS)?;
        
        let start = parse_time_value(start_value)?;
        let stop = parse_time_value(stop_value)?;
        
        (start, stop)
    } else if let Some(arr) = period.as_array() {
        // Array format: [start, stop]
        if arr.len() >= 2 {
            let start = parse_time_value(&arr[0])?;
            let stop = parse_time_value(&arr[1])?;
            (start, stop)
        } else {
            return None;
        }
    } else {
        return None;
    };
    
    // Convert MJD values to ModifiedJulianDate
    let start_epoch = ModifiedJulianDate::new(start_mjd);
    let stop_epoch = ModifiedJulianDate::new(stop_mjd);
    
    // Validate that we can convert to UTC (basic validation)
    start_epoch.to_utc()?;
    stop_epoch.to_utc()?;
    
    Some(Period::new(start_epoch, stop_epoch))
}

/// Find a value in an object by trying multiple keys
fn find_value_by_keys<'a>(obj: &'a serde_json::Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    for key in keys {
        if let Some(value) = obj.get(*key) {
            return Some(value);
        }
    }
    None
}

/// Parse a time value from various formats
fn parse_time_value(value: &Value) -> Option<f64> {
    // Handle nested dictionary format (e.g., {"format": "MJD", "scale": "UTC", "value": 61771.0})
    if let Some(obj) = value.as_object() {
        if let Some(val) = obj.get("value") {
            return parse_time_value(val);
        } else if let Some(val) = obj.get("mjd") {
            return parse_time_value(val);
        } else if let Some(val) = obj.get("MJD") {
            return parse_time_value(val);
        }
        return None;
    }
    
    // Handle numeric values (MJD as float or int)
    if let Some(num) = value.as_f64() {
        return Some(num);
    }
    
    // Handle string values
    if let Some(s) = value.as_str() {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        
        // Try parsing as MJD float
        if let Ok(mjd) = trimmed.parse::<f64>() {
            return Some(mjd);
        }
        
        // Try parsing as ISO timestamp
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
            let utc_dt = dt.with_timezone(&chrono::Utc);
            let epoch = ModifiedJulianDate::from_utc(utc_dt);
            return Some(epoch.value());
        }
        
        // Try pandas-compatible ISO format without timezone
        if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
            let utc_dt = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                naive_dt,
                chrono::Utc,
            );
            let epoch = ModifiedJulianDate::from_utc(utc_dt);
            return Some(epoch.value());
        }
    }
    
    None
}

/// Convert dark periods to a Polars DataFrame
pub fn periods_to_dataframe(periods: Vec<Period>) -> Result<DataFrame> {
    if periods.is_empty() {
        // Return empty DataFrame with correct schema and UTC timezone
        let start_dts: Vec<i64> = vec![];
        let stop_dts: Vec<i64> = vec![];
        let start_mjds: Vec<f64> = vec![];
        let stop_mjds: Vec<f64> = vec![];
        let duration_hours: Vec<f64> = vec![];
        let months_list: Vec<Series> = vec![];
        
        // Create datetime series with UTC timezone
        let start_dt_series = Int64Chunked::from_vec("start_dt".into(), start_dts)
            .into_datetime(TimeUnit::Microseconds, Some(TimeZone::UTC))
            .into_series();
        
        let stop_dt_series = Int64Chunked::from_vec("stop_dt".into(), stop_dts)
            .into_datetime(TimeUnit::Microseconds, Some(TimeZone::UTC))
            .into_series();
        
        let df = df! {
            "start_dt" => start_dt_series,
            "stop_dt" => stop_dt_series,
            "start_mjd" => start_mjds,
            "stop_mjd" => stop_mjds,
            "duration_hours" => duration_hours,
            "months" => months_list,
        }?;
        
        return Ok(df);
    }
    
    let n = periods.len();
    
    // Collect data
    let mut start_dts = Vec::with_capacity(n);
    let mut stop_dts = Vec::with_capacity(n);
    let mut start_mjds = Vec::with_capacity(n);
    let mut stop_mjds = Vec::with_capacity(n);
    let mut duration_hours = Vec::with_capacity(n);
    let mut months_list = Vec::with_capacity(n);
    
    for period in periods {
        let start_dt = period.start.to_utc().expect("Valid start datetime");
        let stop_dt = period.stop.to_utc().expect("Valid stop datetime");
        
        // Convert to microseconds since epoch for Polars datetime
        start_dts.push(start_dt.timestamp_micros());
        stop_dts.push(stop_dt.timestamp_micros());
        start_mjds.push(period.start.value());
        stop_mjds.push(period.stop.value());
        duration_hours.push(period.duration_hours());
        
        // Convert months vec to Series for list column
        let months = enumerate_months(&period);
        months_list.push(Series::from_iter(months));
    }
    
    // Create datetime series with UTC timezone
    let start_dt_series = Int64Chunked::from_vec("start_dt".into(), start_dts)
        .into_datetime(TimeUnit::Microseconds, Some(TimeZone::UTC))
        .into_series();
    
    let stop_dt_series = Int64Chunked::from_vec("stop_dt".into(), stop_dts)
        .into_datetime(TimeUnit::Microseconds, Some(TimeZone::UTC))
        .into_series();
    
    let df = df! {
        "start_dt" => start_dt_series,
        "stop_dt" => stop_dt_series,
        "start_mjd" => start_mjds,
        "stop_mjd" => stop_mjds,
        "duration_hours" => duration_hours,
        "months" => months_list,
    }?;
    
    Ok(df)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_value_mjd() {
        let value = serde_json::json!(61771.0);
        let mjd = parse_time_value(&value).unwrap();
        assert_eq!(mjd, 61771.0);
    }

    #[test]
    fn test_parse_time_value_nested() {
        let value = serde_json::json!({
            "format": "MJD",
            "scale": "UTC",
            "value": 61771.0
        });
        let mjd = parse_time_value(&value).unwrap();
        assert_eq!(mjd, 61771.0);
    }

    #[test]
    fn test_parse_time_value_string() {
        let value = serde_json::json!("61771.0");
        let mjd = parse_time_value(&value).unwrap();
        assert_eq!(mjd, 61771.0);
    }

    #[test]
    fn test_parse_dark_periods() {
        let json = r#"{
            "dark_periods": [
                {
                    "startTime": {
                        "format": "MJD",
                        "scale": "UTC",
                        "value": 61771.0
                    },
                    "stopTime": {
                        "format": "MJD",
                        "scale": "UTC",
                        "value": 61771.276910532266
                    }
                }
            ]
        }"#;

        let periods = parse_dark_periods_str(json).unwrap();
        assert_eq!(periods.len(), 1);
        assert_eq!(periods[0].start.value(), 61771.0);
        assert_eq!(periods[0].stop.value(), 61771.276910532266);
    }

    #[test]
    fn test_enumerate_months() {
        let json = r#"{
            "dark_periods": [
                {
                    "startTime": {"value": 61771.0},
                    "stopTime": {"value": 61802.0}
                }
            ]
        }"#;

        let periods = parse_dark_periods_str(json).unwrap();
        let months = enumerate_months(&periods[0]);
        
        // MJD 61771.0 = 2028-01-01, MJD 61802.0 = 2028-02-01
        // Should cover January through February 2028
        assert!(months.contains(&"2028-01".to_string()));
        assert!(months.contains(&"2028-02".to_string()));
    }
}
