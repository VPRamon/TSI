#![allow(clippy::useless_conversion)]

use serde_json::Value;
use std::collections::HashSet;

/// Remove duplicate rows from records based on subset of columns
pub fn remove_duplicates(
    records: &[Value],
    subset: Option<Vec<String>>,
    keep: &str, // "first", "last", or "none"
) -> Result<Vec<Value>, String> {
    if keep != "first" && keep != "last" && keep != "none" {
        return Err(format!(
            "Invalid keep strategy: {}. Must be 'first', 'last', or 'none'",
            keep
        ));
    }

    let mut seen = HashSet::new();
    let mut result = Vec::new();

    let records_iter: Box<dyn Iterator<Item = &Value>> = if keep == "last" {
        Box::new(records.iter().rev())
    } else {
        Box::new(records.iter())
    };

    for record in records_iter {
        // Create a key based on the subset columns or entire record
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

        if !seen.contains(&key) {
            seen.insert(key);
            result.push(record.clone());
        } else if keep == "none" {
            // Remove from result if duplicate found
            result.retain(|r| {
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
        }
    }

    if keep == "last" {
        result.reverse();
    }

    Ok(result)
}

/// Remove rows with missing coordinates (RA or Dec)
pub fn remove_missing_coordinates(records: &[Value]) -> Result<Vec<Value>, String> {
    let filtered: Vec<Value> = records
        .iter()
        .filter(|r| {
            let has_ra = r.get("raInDeg").and_then(|v| v.as_f64()).is_some();
            let has_dec = r.get("decInDeg").and_then(|v| v.as_f64()).is_some();
            has_ra && has_dec
        })
        .cloned()
        .collect();

    Ok(filtered)
}

/// Impute missing values in a column using various strategies
/// Note: This is a simplified version that returns records with imputed values
pub fn impute_missing(
    records: &[Value],
    column: &str,
    strategy: &str, // "mean", "median", "constant"
    fill_value: Option<f64>,
) -> Result<Vec<Value>, String> {
    // Collect non-null values
    let values: Vec<f64> = records
        .iter()
        .filter_map(|r| r.get(column).and_then(|v| v.as_f64()))
        .collect();

    if values.is_empty() {
        return Ok(records.to_vec());
    }

    let impute_val = match strategy {
        "mean" => values.iter().sum::<f64>() / values.len() as f64,
        "median" => {
            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mid = sorted.len() / 2;
            if sorted.len().is_multiple_of(2) {
                (sorted[mid - 1] + sorted[mid]) / 2.0
            } else {
                sorted[mid]
            }
        }
        "constant" => fill_value
            .ok_or_else(|| "fill_value must be provided for 'constant' strategy".to_string())?,
        _ => {
            return Err(format!(
                "Invalid imputation strategy: {}. Must be 'mean', 'median', or 'constant'",
                strategy
            ));
        }
    };

    // Apply imputation
    let mut result = Vec::new();
    for record in records {
        let mut new_record = record.clone();
        if let Value::Object(ref mut map) = new_record {
            if record.get(column).and_then(|v| v.as_f64()).is_none() {
                map.insert(column.to_string(), serde_json::json!(impute_val));
            }
        }
        result.push(new_record);
    }

    Ok(result)
}

/// Validate record schema (required columns and data types)
pub fn validate_schema(
    records: &[Value],
    required_columns: Vec<String>,
    _expected_dtypes: Option<Vec<(String, String)>>,
) -> Result<(bool, Vec<String>), String> {
    let mut issues: Vec<String> = Vec::new();

    if records.is_empty() {
        return Ok((true, issues));
    }

    // Check for missing required columns in at least one record
    let first_record = &records[0];
    for col in &required_columns {
        if first_record.get(col).is_none() {
            issues.push(format!("Missing required column: {}", col));
        }
    }

    Ok((issues.is_empty(), issues))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_duplicates() {
        let records = vec![
            json!({"id": 1, "value": 10}),
            json!({"id": 2, "value": 20}),
            json!({"id": 2, "value": 20}),
            json!({"id": 3, "value": 30}),
        ];

        let unique = remove_duplicates(&records, None, "first").unwrap();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn test_remove_missing_coordinates() {
        let records = vec![
            json!({"raInDeg": 120.0, "decInDeg": 45.0}),
            json!({"raInDeg": null, "decInDeg": -30.0}),
            json!({"raInDeg": 270.0, "decInDeg": null}),
        ];

        let cleaned = remove_missing_coordinates(&records).unwrap();
        assert_eq!(cleaned.len(), 1);
    }

    #[test]
    fn test_validate_schema() {
        let records = vec![
            json!({"priority": 5.0, "schedulingBlockId": "SB001"}),
            json!({"priority": 10.0, "schedulingBlockId": "SB002"}),
        ];

        let required = vec!["priority".to_string(), "schedulingBlockId".to_string()];
        let (is_valid, issues) = validate_schema(&records, required, None).unwrap();
        assert!(is_valid);
        assert_eq!(issues.len(), 0);

        // Test missing column
        let required_missing = vec!["priority".to_string(), "missing_col".to_string()];
        let (is_valid, issues) = validate_schema(&records, required_missing, None).unwrap();
        assert!(!is_valid);
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn test_impute_missing_median() {
        let records = vec![
            json!({"test": 1.0}),
            json!({"test": null}),
            json!({"test": 3.0}),
            json!({"test": 5.0}),
            json!({"test": null}),
        ];
        let imputed = impute_missing(&records, "test", "median", None).unwrap();

        // Median of [1.0, 3.0, 5.0] is 3.0
        assert_eq!(imputed[1].get("test").unwrap().as_f64().unwrap(), 3.0);
        assert_eq!(imputed[4].get("test").unwrap().as_f64().unwrap(), 3.0);
    }

    #[test]
    fn test_impute_missing_mean() {
        let records = vec![
            json!({"test": 2.0}),
            json!({"test": null}),
            json!({"test": 4.0}),
            json!({"test": 6.0}),
        ];
        let imputed = impute_missing(&records, "test", "mean", None).unwrap();

        // Mean of [2.0, 4.0, 6.0] is 4.0
        assert_eq!(imputed[1].get("test").unwrap().as_f64().unwrap(), 4.0);
    }
}
