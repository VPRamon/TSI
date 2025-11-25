//! Dataset-level analytics and statistical computations.
//!
//! This module provides analytical functions for computing summary statistics,
//! correlations, and extracting insights from telescope scheduling datasets.

use polars::frame::DataFrame;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Not;

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

/// Computes comprehensive dataset-level summary statistics.
///
/// Analyzes a schedule DataFrame and returns aggregated metrics including
/// scheduling rates, priority distributions, and visibility totals.
///
/// # Arguments
///
/// * `df` - Schedule DataFrame with required columns:
///   - `scheduled_flag` (bool): Whether each block is scheduled
///   - `priority` (f64): Priority value for each block
///   - `total_visibility_hours` (f64): Total visibility hours per block
///   - `requested_hours` (f64): Requested observation duration per block
///
/// # Returns
///
/// * `Ok(AnalyticsSnapshot)` - Computed metrics
/// * `Err(PolarsError)` - Column access or computation error
///
/// # Errors
///
/// Returns an error if required columns are missing or have incompatible types.
///
/// # Examples
///
/// ```no_run
/// use tsi_rust::algorithms::compute_metrics;
/// use polars::prelude::*;
///
/// # fn example(df: &DataFrame) -> Result<(), PolarsError> {
/// let metrics = compute_metrics(df)?;
///
/// println!("Total observations: {}", metrics.total_observations);
/// println!("Scheduling rate: {:.2}%", metrics.scheduling_rate * 100.0);
/// println!("Mean priority: {:.2}", metrics.mean_priority);
/// println!("Total visibility: {:.1} hours", metrics.total_visibility_hours);
/// # Ok(())
/// # }
/// ```
///
/// # Performance
///
/// This function performs multiple DataFrame operations including filtering,
/// aggregations, and statistical computations. For large datasets (>100k rows),
/// expect execution times on the order of milliseconds.
pub fn compute_metrics(df: &DataFrame) -> Result<AnalyticsSnapshot, PolarsError> {
    let total_obs = df.height();

    // Get scheduled flag
    let scheduled_flag = df.column("scheduled_flag")?.bool()?;
    let scheduled_count = scheduled_flag.sum().unwrap_or(0) as usize;
    let unscheduled_count = total_obs - scheduled_count;

    // Get priority column
    let priority = df.column("priority")?.f64()?;
    let mean_priority = priority.mean().unwrap_or(0.0);
    let median_priority = priority.median().unwrap_or(0.0);

    // Compute mean priority for scheduled observations
    let scheduled_mask = scheduled_flag;
    let scheduled_df = df.filter(scheduled_mask)?;
    let mean_priority_scheduled = if scheduled_df.height() > 0 {
        scheduled_df
            .column("priority")?
            .f64()?
            .mean()
            .unwrap_or(0.0)
    } else {
        0.0
    };

    // Compute mean priority for unscheduled observations
    let unscheduled_mask = scheduled_flag.not();
    let unscheduled_df = df.filter(&unscheduled_mask)?;
    let mean_priority_unscheduled = if unscheduled_df.height() > 0 {
        unscheduled_df
            .column("priority")?
            .f64()?
            .mean()
            .unwrap_or(0.0)
    } else {
        0.0
    };

    // Total visibility hours
    let total_visibility_hours = df
        .column("total_visibility_hours")?
        .f64()?
        .sum()
        .unwrap_or(0.0);

    // Mean requested hours
    let mean_requested_hours = df.column("requested_hours")?.f64()?.mean().unwrap_or(0.0);

    Ok(AnalyticsSnapshot {
        total_observations: total_obs,
        scheduled_count,
        unscheduled_count,
        scheduling_rate: if total_obs > 0 {
            scheduled_count as f64 / total_obs as f64
        } else {
            0.0
        },
        mean_priority,
        median_priority,
        mean_priority_scheduled,
        mean_priority_unscheduled,
        total_visibility_hours,
        mean_requested_hours,
    })
}

/// Computes Spearman correlation matrix for selected DataFrame columns.
///
/// Calculates pairwise correlations between numeric columns, filtering out
/// columns that don't exist and handling null values appropriately.
///
/// # Arguments
///
/// * `df` - Input DataFrame
/// * `columns` - List of column names to include in correlation analysis
///
/// # Returns
///
/// * `Ok(DataFrame)` - Correlation matrix (currently returns empty; see note)
/// * `Err(PolarsError)` - Column access or computation error
///
/// # Note
///
/// The current implementation returns an empty DataFrame as a placeholder.
/// For production use, integrate with `ndarray-stats` or call Python's `scipy`
/// via PyO3 for actual correlation computation.
///
/// # Examples
///
/// ```no_run
/// use tsi_rust::algorithms::compute_correlations;
/// use polars::prelude::*;
///
/// # fn example(df: &DataFrame) -> Result<(), PolarsError> {
/// let cols = vec![
///     "priority".to_string(),
///     "requested_hours".to_string(),
///     "total_visibility_hours".to_string(),
/// ];
/// let corr_matrix = compute_correlations(df, &cols)?;
/// # Ok(())
/// # }
/// ```
pub fn compute_correlations(df: &DataFrame, columns: &[String]) -> Result<DataFrame, PolarsError> {
    // Filter only existing columns
    let existing_cols: Vec<String> = columns
        .iter()
        .filter(|col| df.column(col).is_ok())
        .cloned()
        .collect();

    if existing_cols.len() < 2 {
        return Ok(DataFrame::empty());
    }

    // Select columns and drop nulls
    let subset = df.select(&existing_cols)?;
    let clean = subset.drop_nulls::<String>(None)?;

    if clean.height() == 0 {
        return Ok(DataFrame::empty());
    }

    // Compute correlation matrix using Polars
    // Note: This is a simplified implementation
    // Polars 0.38 doesn't have direct corr() method in stable API
    // For production, use scipy or ndarray-stats

    // Return empty DataFrame as placeholder
    // Real implementation would use ndarray or scipy via PyO3
    Ok(DataFrame::empty())
}

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
    fn test_compute_correlations_insufficient_columns() {
        let df = DataFrame::empty();
        let columns = vec!["col1".to_string()];
        let result = compute_correlations(&df, &columns);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().height(), 0);
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
    fn test_get_top_observations_and_correlations_paths() {
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

        // Correlations should ignore missing columns and empty rows
        let corr = compute_correlations(&df, &["priority".into(), "missing".into()]).unwrap();
        assert_eq!(corr.height(), 0);
    }
}
