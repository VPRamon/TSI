use polars::prelude::*;
use std::ops::Not;

/// Filter a DataFrame by a single column condition (string value)
pub fn filter_by_column(
    df: &DataFrame,
    column: &str,
    value: &str,
) -> PolarsResult<DataFrame> {
    let series = df.column(column)?;
    let string_ca = series.str()?;
    // Create mask by comparing each element
    let mask: BooleanChunked = string_ca
        .into_iter()
        .map(|opt_val| {
            if let Some(val) = opt_val {
                val == value
            } else {
                false
            }
        })
        .collect();
    df.filter(&mask)
}

/// Filter a DataFrame by numeric range
pub fn filter_by_range(
    df: &DataFrame,
    column: &str,
    min_value: f64,
    max_value: f64,
) -> PolarsResult<DataFrame> {
    let series = df.column(column)?.as_materialized_series();
    
    // Create mask for range [min_value, max_value]
    let ge_mask = series.gt_eq(min_value)?;
    let le_mask = series.lt_eq(max_value)?;
    let mask = &ge_mask & &le_mask;
    
    df.filter(&mask)
}

/// Filter DataFrame by scheduled flag
pub fn filter_by_scheduled(
    df: &DataFrame,
    filter_type: &str, // "All", "Scheduled", "Unscheduled"
) -> PolarsResult<DataFrame> {
    match filter_type {
        "All" => Ok(df.clone()),
        "Scheduled" => {
            let mask = df.column("scheduled_flag")?.bool()?;
            df.filter(mask)
        }
        "Unscheduled" => {
            let mask = df.column("scheduled_flag")?.bool()?;
            let not_mask = mask.not();
            df.filter(&not_mask)
        }
        _ => Err(PolarsError::ComputeError(
            format!("Invalid filter_type: {}. Must be 'All', 'Scheduled', or 'Unscheduled'", filter_type).into()
        )),
    }
}

/// Filter DataFrame by multiple conditions (priority range + scheduled filter)
pub fn filter_dataframe(
    df: &DataFrame,
    priority_min: f64,
    priority_max: f64,
    scheduled_filter: &str,
    priority_bins: Option<Vec<String>>,
    block_ids: Option<Vec<String>>,
) -> PolarsResult<DataFrame> {
    // Start with priority range filter
    let mut filtered = filter_by_range(df, "priority", priority_min, priority_max)?;
    
    // Apply scheduled filter
    filtered = filter_by_scheduled(&filtered, scheduled_filter)?;
    
    // Apply priority bins filter if provided
    if let Some(bins) = priority_bins {
        let series = filtered.column("priority_bin")?;
        let string_ca = series.str()?;
        // Create mask by checking if each value is in the bins list
        let mask: BooleanChunked = string_ca
            .into_iter()
            .map(|opt_val| {
                if let Some(val) = opt_val {
                    bins.contains(&val.to_string())
                } else {
                    false
                }
            })
            .collect();
        filtered = filtered.filter(&mask)?;
    }
    
    // Apply block IDs filter if provided
    if let Some(ids) = block_ids {
        let series = filtered.column("schedulingBlockId")?;
        let string_ca = series.str()?;
        // Create mask by checking if each value is in the ids list
        let mask: BooleanChunked = string_ca
            .into_iter()
            .map(|opt_val| {
                if let Some(val) = opt_val {
                    ids.contains(&val.to_string())
                } else {
                    false
                }
            })
            .collect();
        filtered = filtered.filter(&mask)?;
    }
    
    Ok(filtered)
}

/// Validate DataFrame structure and data quality
pub fn validate_dataframe(df: &DataFrame) -> (bool, Vec<String>) {
    let mut issues: Vec<String> = Vec::new();
    
    // Check for missing schedulingBlockId
    if let Ok(col) = df.column("schedulingBlockId") {
        if col.null_count() > 0 {
            issues.push("Some rows have missing schedulingBlockId".to_string());
        }
    }
    
    // Check for invalid priority values
    if let Ok(col) = df.column("priority") {
        if let Ok(float_col) = col.cast(&DataType::Float64) {
            let null_count = float_col.null_count();
            if null_count > 0 {
                issues.push(format!("{} rows have invalid priority values", null_count));
            }
        }
    }
    
    // Check for invalid declination (must be in [-90, 90])
    if let Ok(col) = df.column("decInDeg") {
        if let Ok(float_col) = col.cast(&DataType::Float64) {
            let series = float_col.f64().unwrap();
            let invalid_count = series.into_iter()
                .filter(|opt| {
                    if let Some(val) = opt {
                        *val < -90.0 || *val > 90.0
                    } else {
                        false
                    }
                })
                .count();
            if invalid_count > 0 {
                issues.push(format!("{} rows have invalid declination", invalid_count));
            }
        }
    }
    
    // Check for invalid right ascension (must be in [0, 360))
    if let Ok(col) = df.column("raInDeg") {
        if let Ok(float_col) = col.cast(&DataType::Float64) {
            let series = float_col.f64().unwrap();
            let invalid_count = series.into_iter()
                .filter(|opt| {
                    if let Some(val) = opt {
                        *val < 0.0 || *val >= 360.0
                    } else {
                        false
                    }
                })
                .count();
            if invalid_count > 0 {
                issues.push(format!("{} rows have invalid right ascension", invalid_count));
            }
        }
    }
    
    (issues.is_empty(), issues)
}

#[cfg(all(test, not(feature = "extension-module")))]
mod tests {
    use super::*;
    
    fn sample_dataframe() -> DataFrame {
        DataFrame::new(vec![
            Series::new("priority".into(), &[5.0, 10.0, 15.0, 20.0]).into(),
            Series::new("scheduled_flag".into(), &[true, false, true, false]).into(),
            Series::new("priority_bin".into(), &["Low", "Medium", "High", "Very High"]).into(),
        ])
        .unwrap()
    }
    
    #[test]
    fn test_filter_by_range() {
        let df = sample_dataframe();
        let filtered = filter_by_range(&df, "priority", 5.0, 15.0).unwrap();
        assert_eq!(filtered.height(), 3);
    }
    
    #[test]
    fn test_filter_by_scheduled() {
        let df = sample_dataframe();
        let scheduled = filter_by_scheduled(&df, "Scheduled").unwrap();
        assert_eq!(scheduled.height(), 2);
        
        let unscheduled = filter_by_scheduled(&df, "Unscheduled").unwrap();
        assert_eq!(unscheduled.height(), 2);
        
        let all = filter_by_scheduled(&df, "All").unwrap();
        assert_eq!(all.height(), 4);
    }
    
    #[test]
    fn test_validate_dataframe() {
        let df = DataFrame::new(vec![
            Series::new("schedulingBlockId".into(), &["SB001", "SB002"]).into(),
            Series::new("priority".into(), &[5.0, 10.0]).into(),
            Series::new("decInDeg".into(), &[45.0, -30.0]).into(),
            Series::new("raInDeg".into(), &[120.0, 270.0]).into(),
        ])
        .unwrap();
        
        let (is_valid, issues) = validate_dataframe(&df);
        assert!(is_valid);
        assert_eq!(issues.len(), 0);
    }
}
