//! Dataset-level analytics and statistical computations.
//!
//! This module provides analytical functions for computing summary statistics,
//! correlations, and extracting insights from telescope scheduling datasets.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Summary statistics snapshot for a scheduling dataset.
///
/// Captures key metrics about observation scheduling including counts,
/// rates, and statistical measures of priority and visibility.
///
/// # Fields
///
/// * `total_observations` - Total number of scheduling blocks in the dataset
/// * `scheduled_count` - Number of blocks with assigned observation times
/// * `unscheduled_count` - Number of blocks without assignments
/// * `scheduling_rate` - Fraction of blocks successfully scheduled (0.0 to 1.0)
/// * `mean_priority` - Average priority across all blocks
/// * `median_priority` - Median priority value
/// * `mean_priority_scheduled` - Average priority of scheduled blocks only
/// * `mean_priority_unscheduled` - Average priority of unscheduled blocks only
/// * `total_visibility_hours` - Sum of all visibility hours across all blocks
/// * `mean_requested_hours` - Average requested observation duration
///
/// # Examples
///
/// ```ignore
/// // This struct is typically created by database-backed analytics functions
/// let snapshot = AnalyticsSnapshot {
///     total_observations: 1000,
///     scheduled_count: 850,
///     unscheduled_count: 150,
///     scheduling_rate: 0.85,
///     mean_priority: 5.2,
///     median_priority: 5.0,
///     mean_priority_scheduled: 5.5,
///     mean_priority_unscheduled: 3.8,
///     total_visibility_hours: 15000.0,
///     mean_requested_hours: 2.5,
/// };
/// println!("Scheduling rate: {:.1}%", snapshot.scheduling_rate * 100.0);
/// println!("Mean priority (scheduled): {:.2}", snapshot.mean_priority_scheduled);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSnapshot {
    pub total_observations: usize,
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub scheduling_rate: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub mean_priority_scheduled: f64,
    pub mean_priority_unscheduled: f64,
    pub total_visibility_hours: f64,
    pub mean_requested_hours: f64,
}

// compute_metrics removed - use database-backed py_get_schedule_summary() instead
// compute_correlations removed - use services::insights::compute_correlations() instead

/// Extracts the top N observations sorted by a specified column.
///
/// Returns a subset of records containing the highest-ranked observations
/// according to the sorting column, with commonly used columns selected.
///
/// # Arguments
///
/// * `records` - Input records as JSON objects
/// * `by` - Column name to sort by (descending order)
/// * `n` - Number of top rows to return
///
/// # Returns
///
/// * `Ok(Vec<Value>)` - Vector with top N records
/// * `Err(String)` - Sort or processing error
///
/// # Selected Columns
///
/// The returned records include:
/// - `schedulingBlockId`
/// - `priority`
/// - `requested_hours`
/// - `total_visibility_hours`
/// - `scheduled_flag`
/// - The sorting column (if different)
///
/// # Edge Cases
///
/// - Returns empty vector if `n` is 0
/// - Returns empty vector if the sorting column doesn't exist in any record
/// - If fewer than N rows exist, returns all available rows
///
/// # Examples
///
/// ```no_run
/// use tsi_rust::algorithms::get_top_observations;
///
/// # fn example(records: &[serde_json::Value]) -> Result<(), String> {
/// // Get top 10 observations by priority
/// let top_priority = get_top_observations(records, "priority", 10)?;
/// println!("Top priority observations: {:?}", top_priority);
///
/// // Get top 5 by visibility hours
/// let top_visibility = get_top_observations(records, "total_visibility_hours", 5)?;
/// # Ok(())
/// # }
/// ```
pub fn get_top_observations(records: &[Value], by: &str, n: usize) -> Result<Vec<Value>, String> {
    if n == 0 {
        return Ok(vec![]);
    }

    // Check if the sort column exists in at least one record
    let has_column = records.iter().any(|r| r.get(by).is_some());
    if !has_column {
        return Ok(vec![]);
    }

    // Create a vector of (index, sort_value) pairs
    let mut indexed_records: Vec<(usize, f64)> = records
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            r.get(by)
                .and_then(|v| v.as_f64())
                .map(|sort_val| (i, sort_val))
        })
        .collect();

    // Sort descending by the sort value
    indexed_records.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top N and extract the corresponding records
    let columns_to_keep = vec![
        "schedulingBlockId",
        "priority",
        "requested_hours",
        "total_visibility_hours",
        "scheduled_flag",
        "priority_bin",
        by,
    ];

    let result: Vec<Value> = indexed_records
        .iter()
        .take(n)
        .map(|(idx, _)| {
            let record = &records[*idx];
            let mut filtered = serde_json::Map::new();

            for col in &columns_to_keep {
                if let Some(val) = record.get(col) {
                    filtered.insert(col.to_string(), val.clone());
                }
            }

            Value::Object(filtered)
        })
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_top_observations() {
        let records = vec![
            json!({
                "schedulingBlockId": "a",
                "priority": 1.0,
                "requested_hours": 1.0,
                "total_visibility_hours": 4.0,
                "scheduled_flag": false,
                "priority_bin": "Low",
                "extra": 1,
            }),
            json!({
                "schedulingBlockId": "b",
                "priority": 3.0,
                "requested_hours": 2.0,
                "total_visibility_hours": 5.0,
                "scheduled_flag": true,
                "priority_bin": "Medium",
                "extra": 2,
            }),
        ];

        let top = get_top_observations(&records, "priority", 1).unwrap();
        assert_eq!(top.len(), 1);
        assert_eq!(
            top[0].get("schedulingBlockId").unwrap().as_str().unwrap(),
            "b"
        );

        // When column missing, returns empty
        assert_eq!(
            get_top_observations(&records, "unknown", 1).unwrap().len(),
            0
        );
        assert_eq!(
            get_top_observations(&records, "priority", 0).unwrap().len(),
            0
        );
    }
}
