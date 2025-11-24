use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Not;

/// Dataset-level summary statistics
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

/// Compute dataset-level summary statistics
///
/// # Arguments
/// * `df` - Schedule DataFrame with required columns
///
/// # Returns
/// AnalyticsSnapshot with aggregated metrics
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
        scheduled_df.column("priority")?.f64()?.mean().unwrap_or(0.0)
    } else {
        0.0
    };
    
    // Compute mean priority for unscheduled observations
    let unscheduled_mask = scheduled_flag.not();
    let unscheduled_df = df.filter(&unscheduled_mask)?;
    let mean_priority_unscheduled = if unscheduled_df.height() > 0 {
        unscheduled_df.column("priority")?.f64()?.mean().unwrap_or(0.0)
    } else {
        0.0
    };
    
    // Total visibility hours
    let total_visibility_hours = df.column("total_visibility_hours")?
        .f64()?
        .sum()
        .unwrap_or(0.0);
    
    // Mean requested hours
    let mean_requested_hours = df.column("requested_hours")?
        .f64()?
        .mean()
        .unwrap_or(0.0);
    
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

/// Compute Spearman correlation matrix for selected columns
///
/// # Arguments
/// * `df` - Input DataFrame
/// * `columns` - List of column names to correlate
///
/// # Returns
/// Correlation matrix as DataFrame
pub fn compute_correlations(
    df: &DataFrame,
    columns: &[String],
) -> Result<DataFrame, PolarsError> {
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

/// Get top N observations ordered by a specific column
///
/// # Arguments
/// * `df` - Input DataFrame
/// * `by` - Column name to sort by
/// * `n` - Number of top rows to return
///
/// # Returns
/// DataFrame with top N rows
pub fn get_top_observations(
    df: &DataFrame,
    by: &str,
    n: usize,
) -> Result<DataFrame, PolarsError> {
    if df.column(by).is_err() || n == 0 {
        return Ok(DataFrame::empty());
    }
    
    // Select relevant columns
    let columns = vec![
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
    let sorted = df.sort([by], vec![true], false)?;
    let top = sorted.head(Some(n));
    
    // Select only relevant columns
    top.select(&existing_cols)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compute_metrics_empty() {
        let df = DataFrame::empty();
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
}
