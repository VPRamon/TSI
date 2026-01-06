use serde_json::Value;

/// Filter records by a single column condition (string value)
pub fn filter_by_column(
    records: &[Value],
    column: &str,
    value: &str,
) -> Result<Vec<Value>, String> {
    let filtered: Vec<Value> = records
        .iter()
        .filter(|r| {
            r.get(column)
                .and_then(|v| v.as_str())
                .map(|s| s == value)
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    Ok(filtered)
}

/// Filter records by numeric range
pub fn filter_by_range(
    records: &[Value],
    column: &str,
    min_value: f64,
    max_value: f64,
) -> Result<Vec<Value>, String> {
    let filtered: Vec<Value> = records
        .iter()
        .filter(|r| {
            r.get(column)
                .and_then(|v| v.as_f64())
                .map(|val| val >= min_value && val <= max_value)
                .unwrap_or(false)
        })
        .cloned()
        .collect();
    Ok(filtered)
}

/// Filter records by scheduled flag
pub fn filter_by_scheduled(
    records: &[Value],
    filter_type: &str, // "All", "Scheduled", "Unscheduled"
) -> Result<Vec<Value>, String> {
    match filter_type {
        "All" => Ok(records.to_vec()),
        "Scheduled" => {
            let filtered: Vec<Value> = records
                .iter()
                .filter(|r| {
                    r.get("scheduled_flag")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
            Ok(filtered)
        }
        "Unscheduled" => {
            let filtered: Vec<Value> = records
                .iter()
                .filter(|r| {
                    !r.get("scheduled_flag")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
            Ok(filtered)
        }
        _ => Err(format!(
            "Invalid filter_type: {}. Must be 'All', 'Scheduled', or 'Unscheduled'",
            filter_type
        )),
    }
}

/// Filter records by multiple conditions (priority range + scheduled filter)
pub fn filter_dataframe(
    records: &[Value],
    priority_min: f64,
    priority_max: f64,
    scheduled_filter: &str,
    priority_bins: Option<Vec<String>>,
    block_ids: Option<Vec<String>>,
) -> Result<Vec<Value>, String> {
    // Start with priority range filter
    let mut filtered = filter_by_range(records, "priority", priority_min, priority_max)?;

    // Apply scheduled filter
    filtered = filter_by_scheduled(&filtered, scheduled_filter)?;

    // Apply priority bins filter if provided
    if let Some(bins) = priority_bins {
        filtered.retain(|r| {
                r.get("priority_bin")
                    .and_then(|v| v.as_str())
                    .map(|s| bins.contains(&s.to_string()))
                    .unwrap_or(false)
            });
    }

    // Apply block IDs filter if provided
    if let Some(ids) = block_ids {
        filtered.retain(|r| {
                r.get("schedulingBlockId")
                    .and_then(|v| v.as_str())
                    .map(|s| ids.contains(&s.to_string()))
                    .unwrap_or(false)
            });
    }

    Ok(filtered)
}

/// Validate record structure and data quality
pub fn validate_dataframe(records: &[Value]) -> (bool, Vec<String>) {
    let mut issues: Vec<String> = Vec::new();

    // Check for missing schedulingBlockId
    let missing_id_count = records
        .iter()
        .filter(|r| r.get("schedulingBlockId").is_none())
        .count();
    if missing_id_count > 0 {
        issues.push("Some rows have missing schedulingBlockId".to_string());
    }

    // Check for invalid priority values
    let invalid_priority_count = records
        .iter()
        .filter(|r| r.get("priority").and_then(|v| v.as_f64()).is_none())
        .count();
    if invalid_priority_count > 0 {
        issues.push(format!(
            "{} rows have invalid priority values",
            invalid_priority_count
        ));
    }

    // Check for invalid declination (must be in [-90, 90])
    let invalid_dec_count = records
        .iter()
        .filter(|r| {
            r.get("decInDeg")
                .and_then(|v| v.as_f64())
                .map(|val| !(-90.0..=90.0).contains(&val))
                .unwrap_or(false)
        })
        .count();
    if invalid_dec_count > 0 {
        issues.push(format!(
            "{} rows have invalid declination",
            invalid_dec_count
        ));
    }

    // Check for invalid right ascension (must be in [0, 360))
    let invalid_ra_count = records
        .iter()
        .filter(|r| {
            r.get("raInDeg")
                .and_then(|v| v.as_f64())
                .map(|val| !(0.0..360.0).contains(&val))
                .unwrap_or(false)
        })
        .count();
    if invalid_ra_count > 0 {
        issues.push(format!(
            "{} rows have invalid right ascension",
            invalid_ra_count
        ));
    }

    (issues.is_empty(), issues)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_records() -> Vec<Value> {
        vec![
            json!({
                "priority": 5.0,
                "scheduled_flag": true,
                "priority_bin": "Low",
            }),
            json!({
                "priority": 10.0,
                "scheduled_flag": false,
                "priority_bin": "Medium",
            }),
            json!({
                "priority": 15.0,
                "scheduled_flag": true,
                "priority_bin": "High",
            }),
            json!({
                "priority": 20.0,
                "scheduled_flag": false,
                "priority_bin": "Very High",
            }),
        ]
    }

    #[test]
    fn test_filter_by_range() {
        let records = sample_records();
        let filtered = filter_by_range(&records, "priority", 8.0, 18.0).unwrap();
        assert_eq!(filtered.len(), 2);
        // Should include 10.0 and 15.0
        assert_eq!(filtered[0].get("priority").unwrap().as_f64().unwrap(), 10.0);
        assert_eq!(filtered[1].get("priority").unwrap().as_f64().unwrap(), 15.0);
    }

    #[test]
    fn test_filter_by_scheduled() {
        let records = sample_records();

        let all = filter_by_scheduled(&records, "All").unwrap();
        assert_eq!(all.len(), 4);

        let scheduled = filter_by_scheduled(&records, "Scheduled").unwrap();
        assert_eq!(scheduled.len(), 2);

        let unscheduled = filter_by_scheduled(&records, "Unscheduled").unwrap();
        assert_eq!(unscheduled.len(), 2);
    }

    #[test]
    fn test_filter_dataframe() {
        let records = sample_records();

        // Priority 8-18, scheduled only
        let filtered = filter_dataframe(&records, 8.0, 18.0, "Scheduled", None, None).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].get("priority").unwrap().as_f64().unwrap(), 15.0);
    }

    #[test]
    fn test_validate_dataframe() {
        let good_records = vec![json!({
            "schedulingBlockId": "SB001",
            "priority": 5.0,
            "raInDeg": 120.0,
            "decInDeg": 45.0,
        })];
        let (is_valid, issues) = validate_dataframe(&good_records);
        assert!(is_valid);
        assert_eq!(issues.len(), 0);

        let bad_records = vec![json!({
            "priority": 5.0,
            "raInDeg": 400.0,  // Invalid
            "decInDeg": -100.0, // Invalid
        })];
        let (is_valid, issues) = validate_dataframe(&bad_records);
        assert!(!is_valid);
        assert!(issues.len() >= 2);
    }
}
