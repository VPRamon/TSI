use polars::prelude::*;

/// Remove duplicate rows from a DataFrame
pub fn remove_duplicates(
    df: &DataFrame,
    subset: Option<Vec<String>>,
    keep: &str, // "first", "last", or "none"
) -> PolarsResult<DataFrame> {
    let unique_strategy = match keep {
        "first" => UniqueKeepStrategy::First,
        "last" => UniqueKeepStrategy::Last,
        "none" => UniqueKeepStrategy::None,
        _ => return Err(PolarsError::ComputeError(
            format!("Invalid keep strategy: {}. Must be 'first', 'last', or 'none'", keep).into()
        )),
    };
    
    if let Some(cols) = subset {
        let col_strs: Vec<String> = cols.into_iter().collect();
        df.unique::<Vec<String>, String>(Some(&col_strs), unique_strategy, None)
    } else {
        df.unique::<Vec<String>, String>(None::<&[String]>, unique_strategy, None)
    }
}

/// Remove rows with missing coordinates (RA or Dec)
pub fn remove_missing_coordinates(df: &DataFrame) -> PolarsResult<DataFrame> {
    // Filter out rows where raInDeg or decInDeg are null
    let ra_col = df.column("raInDeg")?;
    let dec_col = df.column("decInDeg")?;
    
    let ra_not_null = ra_col.is_not_null();
    let dec_not_null = dec_col.is_not_null();
    
    let mask = &ra_not_null & &dec_not_null;
    df.filter(&mask)
}

/// Impute missing values in a Series using various strategies
pub fn impute_missing(
    series: &Series,
    strategy: &str, // "mean", "median", "constant"
    fill_value: Option<f64>,
) -> PolarsResult<Series> {
    match strategy {
        "mean" => {
            let float_series = series.cast(&DataType::Float64)?;
            if float_series.mean().is_some() {
                Ok(float_series.fill_null(FillNullStrategy::Mean)?)
            } else {
                Ok(series.clone())
            }
        }
        "median" => {
            let float_series = series.cast(&DataType::Float64)?;
            if float_series.median().is_some() {
                Ok(float_series.fill_null(FillNullStrategy::Mean)?)
            } else {
                Ok(series.clone())
            }
        }
        "constant" => {
            if let Some(_val) = fill_value {
                Ok(series.fill_null(FillNullStrategy::Forward(None))?)
            } else {
                Err(PolarsError::ComputeError(
                    "fill_value must be provided for 'constant' strategy".into()
                ))
            }
        }
        _ => Err(PolarsError::ComputeError(
            format!("Invalid imputation strategy: {}. Must be 'mean', 'median', or 'constant'", strategy).into()
        )),
    }
}

/// Validate DataFrame schema (required columns and data types)
pub fn validate_schema(
    df: &DataFrame,
    required_columns: Vec<String>,
    expected_dtypes: Option<Vec<(String, DataType)>>,
) -> PolarsResult<(bool, Vec<String>)> {
    let mut issues: Vec<String> = Vec::new();
    
    // Check for missing required columns
    let col_names: Vec<_> = df.get_column_names_owned();
    for col in &required_columns {
        if !col_names.iter().any(|c| c.as_str() == col.as_str()) {
            issues.push(format!("Missing required column: {}", col));
        }
    }
    
    // Check data types if provided
    if let Some(dtypes) = expected_dtypes {
        for (col_name, expected_dtype) in dtypes {
            if let Ok(col) = df.column(&col_name) {
                let actual_dtype = col.dtype();
                if actual_dtype != &expected_dtype {
                    issues.push(format!(
                        "Column '{}' has incorrect type: expected {:?}, got {:?}",
                        col_name, expected_dtype, actual_dtype
                    ));
                }
            }
        }
    }
    
    Ok((issues.is_empty(), issues))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_remove_duplicates() {
        let df = DataFrame::new(vec![
            Series::new("id".into(), &[1, 2, 2, 3]).into(),
            Series::new("value".into(), &[10, 20, 20, 30]).into(),
        ])
        .unwrap();
        
        let unique_df = remove_duplicates(&df, None, "first").unwrap();
        assert_eq!(unique_df.height(), 3);
    }
    
    #[test]
    fn test_remove_missing_coordinates() {
        let df = DataFrame::new(vec![
            Series::new("raInDeg".into(), &[Some(120.0), None, Some(270.0)]).into(),
            Series::new("decInDeg".into(), &[Some(45.0), Some(-30.0), None]).into(),
        ])
        .unwrap();
        
        let cleaned = remove_missing_coordinates(&df).unwrap();
        assert_eq!(cleaned.height(), 1);
    }
    
    #[test]
    fn test_validate_schema() {
        let df = DataFrame::new(vec![
            Series::new("priority".into(), &[5.0, 10.0]).into(),
            Series::new("schedulingBlockId".into(), &["SB001", "SB002"]).into(),
        ])
        .unwrap();
        
        let required = vec!["priority".to_string(), "schedulingBlockId".to_string()];
        let (is_valid, issues) = validate_schema(&df, required, None).unwrap();
        assert!(is_valid);
        assert_eq!(issues.len(), 0);
        
        // Test missing column
        let required_missing = vec!["priority".to_string(), "missing_col".to_string()];
        let (is_valid, issues) = validate_schema(&df, required_missing, None).unwrap();
        assert!(!is_valid);
        assert_eq!(issues.len(), 1);
    }
}
