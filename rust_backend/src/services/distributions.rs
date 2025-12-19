#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::redundant_closure)]

use crate::db::models::{DistributionBlock, DistributionData, DistributionStats};
use crate::db::repository::AnalyticsRepository;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

// Import the global repository accessor
use crate::python::database::get_repository;

/// Compute statistics for a set of values.
/// This is a helper function that calculates mean, median, std dev, min, max, and sum.
fn compute_stats(values: &[f64]) -> DistributionStats {
    if values.is_empty() {
        return DistributionStats {
            count: 0,
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            sum: 0.0,
        };
    }

    let count = values.len();
    let sum: f64 = values.iter().sum();
    let mean = sum / count as f64;

    // Compute median
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if count % 2 == 0 {
        (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
    } else {
        sorted[count / 2]
    };

    // Compute standard deviation
    let variance = values
        .iter()
        .map(|v| {
            let diff = v - mean;
            diff * diff
        })
        .sum::<f64>()
        / count as f64;
    let std_dev = variance.sqrt();

    let min = sorted.first().copied().unwrap_or(0.0);
    let max = sorted.last().copied().unwrap_or(0.0);

    DistributionStats {
        count,
        mean,
        median,
        std_dev,
        min,
        max,
        sum,
    }
}

/// Compute distribution data with statistics from raw blocks.
/// This function takes the blocks and computes all necessary statistics on the Rust side.
pub fn compute_distribution_data(
    blocks: Vec<DistributionBlock>,
) -> Result<DistributionData, String> {
    let total_count = blocks.len();
    let scheduled_count = blocks.iter().filter(|b| b.scheduled).count();
    let unscheduled_count = total_count - scheduled_count;

    // Count impossible observations (those with zero visibility)
    let impossible_count = blocks
        .iter()
        .filter(|b| b.total_visibility_hours == 0.0)
        .count();

    // Collect values for statistics
    let priorities: Vec<f64> = blocks.iter().map(|b| b.priority).collect();
    let visibility_hours: Vec<f64> = blocks.iter().map(|b| b.total_visibility_hours).collect();
    let requested_hours: Vec<f64> = blocks.iter().map(|b| b.requested_hours).collect();

    // Compute statistics
    let priority_stats = compute_stats(&priorities);
    let visibility_stats = compute_stats(&visibility_hours);
    let requested_hours_stats = compute_stats(&requested_hours);

    Ok(DistributionData {
        blocks,
        priority_stats,
        visibility_stats,
        requested_hours_stats,
        total_count,
        scheduled_count,
        unscheduled_count,
        impossible_count,
    })
}

/// Get complete distribution data with computed statistics using ETL analytics.
///
/// This function retrieves blocks from the analytics.schedule_blocks_analytics table
/// which contains pre-computed, denormalized data for optimal performance.
///
/// **Note**: Impossible blocks (zero visibility) are automatically excluded.
pub async fn get_distribution_data(schedule_id: i64) -> Result<DistributionData, String> {
    // Get the initialized repository
    let repo = get_repository().map_err(|e| format!("Failed to get repository: {}", e))?;

    let mut blocks = repo
        .fetch_analytics_blocks_for_distribution(schedule_id)
        .await
        .map_err(|e| format!("Failed to fetch analytics blocks: {}", e))?;

    if blocks.is_empty() {
        return Err(format!(
            "No analytics data available for schedule_id={}. Run populate_schedule_analytics() first.",
            schedule_id
        ));
    }

    // Filter out impossible blocks (zero visibility)
    blocks.retain(|b| b.total_visibility_hours > 0.0);

    compute_distribution_data(blocks)
}

/// Get complete distribution data with computed statistics and metadata.
/// This is the main Python-callable function for the distributions feature.
///
/// **Note**: Impossible blocks are automatically excluded.
#[pyfunction]
pub fn py_get_distribution_data(schedule_id: i64) -> PyResult<DistributionData> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(get_distribution_data(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}

/// Alias for compatibility - uses analytics path.
#[pyfunction]
pub fn py_get_distribution_data_analytics(schedule_id: i64) -> PyResult<DistributionData> {
    py_get_distribution_data(schedule_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_stats() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = compute_stats(&values);

        assert_eq!(stats.count, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.sum, 15.0);
        assert!((stats.std_dev - std::f64::consts::SQRT_2).abs() < 0.001);
    }

    #[test]
    fn test_compute_stats_empty() {
        let values = vec![];
        let stats = compute_stats(&values);

        assert_eq!(stats.count, 0);
        assert_eq!(stats.mean, 0.0);
    }

    #[test]
    fn test_compute_distribution_data() {
        let blocks = vec![
            DistributionBlock {
                priority: 5.0,
                total_visibility_hours: 10.0,
                requested_hours: 2.0,
                elevation_range_deg: 45.0,
                scheduled: true,
            },
            DistributionBlock {
                priority: 3.0,
                total_visibility_hours: 0.0,
                requested_hours: 1.0,
                elevation_range_deg: 30.0,
                scheduled: false,
            },
            DistributionBlock {
                priority: 7.0,
                total_visibility_hours: 15.0,
                requested_hours: 3.0,
                elevation_range_deg: 60.0,
                scheduled: true,
            },
        ];

        let result = compute_distribution_data(blocks).unwrap();

        assert_eq!(result.total_count, 3);
        assert_eq!(result.scheduled_count, 2);
        assert_eq!(result.unscheduled_count, 1);
        assert_eq!(result.impossible_count, 1);
        assert_eq!(result.priority_stats.mean, 5.0);
    }
}
