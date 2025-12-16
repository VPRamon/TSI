//! Dataset-level analytics and statistical computations.
//!
//! This module provides analytical functions for computing summary statistics,
//! correlations, and extracting insights from telescope scheduling datasets.

use polars::frame::DataFrame;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

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
/// ```no_run
/// use tsi_rust::algorithms::compute_metrics;
/// use polars::prelude::*;
///
/// # fn example(df: &DataFrame) -> Result<(), PolarsError> {
/// let snapshot = compute_metrics(df)?;
/// println!("Scheduling rate: {:.1}%", snapshot.scheduling_rate * 100.0);
/// println!("Mean priority (scheduled): {:.2}", snapshot.mean_priority_scheduled);
/// # Ok(())
/// # }
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
/// Returns a subset DataFrame containing the highest-ranked observations
/// according to the sorting column, with commonly used columns selected.
///
/// # Arguments
///
/// * `df` - Input DataFrame
/// * `by` - Column name to sort by (descending order)
/// * `n` - Number of top rows to return
///
/// # Returns
///
/// * `Ok(DataFrame)` - DataFrame with top N rows and selected columns
/// * `Err(PolarsError)` - Sort or select error
///
/// # Selected Columns
///
/// The returned DataFrame includes:
/// - `schedulingBlockId`
/// - `priority`
/// - `requested_hours`
/// - `total_visibility_hours`
/// - `scheduled_flag`
/// - The sorting column (if different)
///
/// # Edge Cases
///
/// - Returns empty DataFrame if `n` is 0
/// - Returns empty DataFrame if the sorting column doesn't exist
/// - If fewer than N rows exist, returns all available rows
///
/// # Examples
///
/// ```no_run
/// use tsi_rust::algorithms::get_top_observations;
/// use polars::prelude::*;
///
/// # fn example(df: &DataFrame) -> Result<(), PolarsError> {
/// // Get top 10 observations by priority
/// let top_priority = get_top_observations(df, "priority", 10)?;
/// println!("Top priority observations:");
/// println!("{:?}", top_priority);
///
/// // Get top 5 by visibility hours
/// let top_visibility = get_top_observations(df, "total_visibility_hours", 5)?;
/// # Ok(())
/// # }
/// ```
pub fn get_top_observations(df: &DataFrame, by: &str, n: usize) -> Result<DataFrame, PolarsError> {
    if df.column(by).is_err() || n == 0 {
        return Ok(DataFrame::empty());
    }

    // Select relevant columns
    let columns = [
        "schedulingBlockId",
        "priority",
        "requested_hours",
        "total_visibility_hours",
        "scheduled_flag",
        "priority_bin",
    ];

    let existing_cols: Vec<&str> = columns
        .iter()
        .filter(|&&col| df.column(col).is_ok())
        .copied()
        .collect();

    // Sort descending and take top n
    let sorted = df.sort(
        [by],
        SortMultipleOptions::default().with_order_descending_multi([true]),
    )?;
    let top = sorted.head(Some(n));

    // Select only relevant columns
    top.select(existing_cols)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_metrics_empty() {
        let _df = DataFrame::empty();
        // Should handle empty DataFrame gracefully
    }

    #[test]
    fn test_compute_metrics_with_data() {
        let df = df!(
            "priority" => &[5.0, 10.0, 15.0],
            "scheduled_flag" => &[true, false, true],
            "total_visibility_hours" => &[1.0, 2.0, 3.0],
            "requested_hours" => &[0.5, 1.0, 2.0],
        )
        .unwrap();

        let metrics = compute_metrics(&df).unwrap();
        assert_eq!(metrics.total_observations, 3);
        assert_eq!(metrics.scheduled_count, 2);
        assert_eq!(metrics.unscheduled_count, 1);
        assert_eq!(metrics.mean_priority, 10.0);
        assert_eq!(metrics.mean_priority_scheduled, 10.0);
        assert_eq!(metrics.mean_priority_unscheduled, 10.0);
        assert_eq!(metrics.total_visibility_hours, 6.0);
        assert_eq!(metrics.mean_requested_hours, 1.1666666666666667);
    }

    #[test]
    fn test_get_top_observations() {
        let df = df!(
            "schedulingBlockId" => &["a", "b"],
            "priority" => &[1.0, 3.0],
            "requested_hours" => &[1.0, 2.0],
            "total_visibility_hours" => &[4.0, 5.0],
            "scheduled_flag" => &[false, true],
            "priority_bin" => &["Low", "Medium"],
            "extra" => &[1, 2],
        )
        .unwrap();

        let top = get_top_observations(&df, "priority", 1).unwrap();
        assert_eq!(top.height(), 1);
        assert_eq!(
            top.column("schedulingBlockId")
                .unwrap()
                .str()
                .unwrap()
                .get(0),
            Some("b")
        );

        // When column missing, returns empty
        assert_eq!(get_top_observations(&df, "unknown", 1).unwrap().height(), 0);
        assert_eq!(
            get_top_observations(&df, "priority", 0).unwrap().height(),
            0
        );
    }
}
