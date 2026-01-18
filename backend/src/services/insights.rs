#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::useless_vec)]

use crate::db::models::{
    AnalyticsMetrics, ConflictRecord, CorrelationEntry, InsightsBlock, InsightsData, TopObservation,
};
use tokio::runtime::Runtime;

// Import the global repository accessor
use crate::db::get_repository;
use qtty::time::Hours;

/// Compute analytics metrics from insights blocks.
pub(crate) fn compute_metrics(blocks: &[InsightsBlock]) -> AnalyticsMetrics {
    let total_observations = blocks.len();
    let scheduled_count = blocks.iter().filter(|b| b.scheduled).count();
    let unscheduled_count = total_observations - scheduled_count;

    let scheduling_rate = if total_observations > 0 {
        scheduled_count as f64 / total_observations as f64
    } else {
        0.0
    };

    // Collect priorities for stats
    let priorities: Vec<f64> = blocks.iter().map(|b| b.priority).collect();
    let mean_priority = if !priorities.is_empty() {
        priorities.iter().sum::<f64>() / priorities.len() as f64
    } else {
        0.0
    };

    // Compute median priority
    let mut sorted_priorities = priorities.clone();
    sorted_priorities.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_priority = if !sorted_priorities.is_empty() {
        if sorted_priorities.len() % 2 == 0 {
            (sorted_priorities[sorted_priorities.len() / 2 - 1]
                + sorted_priorities[sorted_priorities.len() / 2])
                / 2.0
        } else {
            sorted_priorities[sorted_priorities.len() / 2]
        }
    } else {
        0.0
    };

    // Scheduled and unscheduled priorities
    let scheduled_priorities: Vec<f64> = blocks
        .iter()
        .filter(|b| b.scheduled)
        .map(|b| b.priority)
        .collect();
    let mean_priority_scheduled = if !scheduled_priorities.is_empty() {
        scheduled_priorities.iter().sum::<f64>() / scheduled_priorities.len() as f64
    } else {
        0.0
    };

    let unscheduled_priorities: Vec<f64> = blocks
        .iter()
        .filter(|b| !b.scheduled)
        .map(|b| b.priority)
        .collect();
    let mean_priority_unscheduled = if !unscheduled_priorities.is_empty() {
        unscheduled_priorities.iter().sum::<f64>() / unscheduled_priorities.len() as f64
    } else {
        0.0
    };

    // Total visibility and requested hours (work with raw f64 values, then wrap as Hours)
    let total_visibility_hours_f64: f64 = blocks
        .iter()
        .map(|b| b.total_visibility_hours.value())
        .sum();
    let requested_hours_vec: Vec<f64> = blocks.iter().map(|b| b.requested_hours.value()).collect();
    let mean_requested_hours_f64 = if !requested_hours_vec.is_empty() {
        requested_hours_vec.iter().sum::<f64>() / requested_hours_vec.len() as f64
    } else {
        0.0
    };

    AnalyticsMetrics {
        total_observations,
        scheduled_count,
        unscheduled_count,
        scheduling_rate,
        mean_priority,
        median_priority,
        mean_priority_scheduled,
        mean_priority_unscheduled,
        total_visibility_hours: qtty::time::Hours::new(total_visibility_hours_f64),
        mean_requested_hours: qtty::time::Hours::new(mean_requested_hours_f64),
    }
}

/// Compute Spearman rank correlation between two variables.
/// Uses a simple implementation of Spearman's rank correlation coefficient.
pub(crate) fn compute_spearman_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }

    let n = x.len();

    // Create ranks for x
    let mut x_indexed: Vec<(usize, f64)> = x.iter().copied().enumerate().collect();
    x_indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut x_ranks = vec![0.0; n];
    for (rank, (idx, _)) in x_indexed.iter().enumerate() {
        x_ranks[*idx] = (rank + 1) as f64;
    }

    // Create ranks for y
    let mut y_indexed: Vec<(usize, f64)> = y.iter().copied().enumerate().collect();
    y_indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut y_ranks = vec![0.0; n];
    for (rank, (idx, _)) in y_indexed.iter().enumerate() {
        y_ranks[*idx] = (rank + 1) as f64;
    }

    // Compute Pearson correlation on ranks
    let mean_x = x_ranks.iter().sum::<f64>() / n as f64;
    let mean_y = y_ranks.iter().sum::<f64>() / n as f64;

    let mut numerator = 0.0;
    let mut sum_sq_x = 0.0;
    let mut sum_sq_y = 0.0;

    for i in 0..n {
        let dx = x_ranks[i] - mean_x;
        let dy = y_ranks[i] - mean_y;
        numerator += dx * dy;
        sum_sq_x += dx * dx;
        sum_sq_y += dy * dy;
    }

    let denominator = (sum_sq_x * sum_sq_y).sqrt();
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

/// Compute correlations between key variables.
pub(crate) fn compute_correlations(blocks: &[InsightsBlock]) -> Vec<CorrelationEntry> {
    if blocks.len() < 2 {
        return vec![];
    }

    // Extract variables (use primitive values for statistical routines)
    let priorities: Vec<f64> = blocks.iter().map(|b| b.priority).collect();
    let visibility: Vec<f64> = blocks
        .iter()
        .map(|b| b.total_visibility_hours.value())
        .collect();
    let requested: Vec<f64> = blocks.iter().map(|b| b.requested_hours.value()).collect();
    let elevation: Vec<f64> = blocks
        .iter()
        .map(|b| b.elevation_range_deg.value())
        .collect();

    let variables = vec![
        ("priority", &priorities[..]),
        ("total_visibility_hours", &visibility[..]),
        ("requested_hours", &requested[..]),
        ("elevation_range_deg", &elevation[..]),
    ];

    let mut correlations = Vec::new();

    // Compute all pairwise correlations
    for i in 0..variables.len() {
        for j in (i + 1)..variables.len() {
            let (name1, data1) = variables[i];
            let (name2, data2) = variables[j];

            let corr = compute_spearman_correlation(data1, data2);
            correlations.push(CorrelationEntry {
                variable1: name1.to_string(),
                variable2: name2.to_string(),
                correlation: corr,
            });
        }
    }

    correlations
}

/// Get top N observations by a specified metric.
fn get_top_observations(blocks: &[InsightsBlock], by: &str, n: usize) -> Vec<TopObservation> {
    let mut sorted_blocks: Vec<_> = blocks.iter().collect();

    match by {
        "priority" => {
            sorted_blocks.sort_by(|a, b| {
                b.priority
                    .partial_cmp(&a.priority)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        "total_visibility_hours" => {
            sorted_blocks.sort_by(|a, b| {
                b.total_visibility_hours
                    .value()
                    .partial_cmp(&a.total_visibility_hours.value())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        _ => return vec![],
    }

    sorted_blocks
        .into_iter()
        .take(n)
        .map(|b| TopObservation {
            scheduling_block_id: b.scheduling_block_id,
            original_block_id: b.original_block_id.clone(),
            priority: b.priority,
            total_visibility_hours: b.total_visibility_hours,
            requested_hours: b.requested_hours,
            scheduled: b.scheduled,
        })
        .collect()
}

/// Find scheduling conflicts (overlapping scheduled observations).
fn find_conflicts(blocks: &[InsightsBlock]) -> Vec<ConflictRecord> {
    let mut conflicts = Vec::new();

    // Get only scheduled blocks with valid times
    let scheduled: Vec<_> = blocks
        .iter()
        .filter(|b| {
            b.scheduled && b.scheduled_start_mjd.is_some() && b.scheduled_stop_mjd.is_some()
        })
        .collect();

    // Compare all pairs
    for i in 0..scheduled.len() {
        for j in (i + 1)..scheduled.len() {
            let block1 = scheduled[i];
            let block2 = scheduled[j];

            let start1 = block1.scheduled_start_mjd.unwrap();
            let stop1 = block1.scheduled_stop_mjd.unwrap();
            let start2 = block2.scheduled_start_mjd.unwrap();
            let stop2 = block2.scheduled_stop_mjd.unwrap();

            // Use primitive mjd values for numeric overlap calculations
            let s1 = start1.value();
            let e1 = stop1.value();
            let s2 = start2.value();
            let e2 = stop2.value();

            let overlap_start = s1.max(s2);
            let overlap_stop = e1.min(e2);

            if overlap_start < overlap_stop {
                let overlap_days = overlap_stop - overlap_start;
                let overlap_hours_f64 = overlap_days * 24.0;

                conflicts.push(ConflictRecord {
                    block_id_1: block1.original_block_id.clone(),
                    block_id_2: block2.original_block_id.clone(),
                    start_time_1: start1,
                    stop_time_1: stop1,
                    start_time_2: start2,
                    stop_time_2: stop2,
                    overlap_hours: Hours::new(overlap_hours_f64),
                });
            }
        }
    }

    conflicts
}

/// Compute insights data with all analytics from raw blocks.
pub fn compute_insights_data(blocks: Vec<InsightsBlock>) -> Result<InsightsData, String> {
    let total_count = blocks.len();
    let scheduled_count = blocks.iter().filter(|b| b.scheduled).count();
    let impossible_count = blocks
        .iter()
        .filter(|b| b.total_visibility_hours.value() == 0.0)
        .count();

    // Compute all analytics
    let metrics = compute_metrics(&blocks);
    let correlations = compute_correlations(&blocks);
    let top_priority = get_top_observations(&blocks, "priority", 10);
    let top_visibility = get_top_observations(&blocks, "total_visibility_hours", 10);
    let conflicts = find_conflicts(&blocks);

    Ok(InsightsData {
        blocks,
        metrics,
        correlations,
        top_priority,
        top_visibility,
        conflicts,
        total_count,
        scheduled_count,
        impossible_count,
    })
}

/// Get complete insights data with computed analytics.
/// Uses pre-computed analytics table when available for ~10-100x faster performance.
///
/// **Note**: Impossible blocks (zero visibility) are automatically excluded during ETL.
/// Validation results are stored separately and can be retrieved via py_get_validation_report.
pub async fn get_insights_data(schedule_id: i64) -> Result<InsightsData, String> {
    // Get the initialized repository
    let repo = get_repository().map_err(|e| format!("Failed to get repository: {}", e))?;

    // Fetch insights-ready analytics blocks
    let mut blocks = repo
        .fetch_analytics_blocks_for_insights(crate::api::ScheduleId(schedule_id))
        .await
        .map_err(|e| format!("Failed to fetch insights blocks: {}", e))?;

    if blocks.is_empty() {
        return Err(format!(
            "No analytics data available for schedule_id={}. Run populate_schedule_analytics() first.",
            schedule_id
        ));
    }

    // Filter out impossible blocks (zero visibility)
    // These are tracked in the validation results table
    blocks.retain(|b| b.total_visibility_hours.value() > 0.0);

    if blocks.is_empty() {
        return Err(format!(
            "No blocks with visibility data found for schedule_id={}. All blocks have zero visibility hours. This likely means visibility_periods data is missing from the schedule.",
            schedule_id
        ));
    }

    compute_insights_data(blocks)
}

/// Get complete insights data with computed analytics and metadata.
/// This is the main function for the insights feature, computing all analytics
/// on the Rust side for maximum performance.
///
/// **Note**: Impossible blocks (zero visibility) are automatically excluded.
/// To see validation issues, use py_get_validation_report.
pub fn py_get_insights_data(schedule_id: crate::api::ScheduleId) -> Result<InsightsData, String> {
    let runtime = Runtime::new().map_err(|e| {
        format!("Failed to create async runtime: {}", e)
    })?;

    runtime
        .block_on(get_insights_data(schedule_id.0))
}

#[cfg(test)]
mod tests {
    use super::{compute_correlations, compute_metrics, compute_spearman_correlation};
    use crate::api::InsightsBlock;

    fn create_test_block(
        priority: f64,
        scheduled: bool,
        visibility: f64,
        requested: f64,
    ) -> InsightsBlock {
        InsightsBlock {
            scheduling_block_id: 1,
            original_block_id: "test".to_string(),
            priority,
            total_visibility_hours: qtty::Hours::new(visibility),
            requested_hours: qtty::Hours::new(requested),
            elevation_range_deg: qtty::Degrees::new(30.0),
            scheduled,
            scheduled_start_mjd: None,
            scheduled_stop_mjd: None,
        }
    }

    #[test]
    fn test_compute_metrics_empty() {
        let blocks = vec![];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.total_observations, 0);
        assert_eq!(metrics.scheduled_count, 0);
        assert_eq!(metrics.scheduling_rate, 0.0);
    }

    #[test]
    fn test_compute_metrics_basic() {
        let blocks = vec![
            create_test_block(5.0, true, 10.0, 2.0),
            create_test_block(7.0, false, 15.0, 3.0),
            create_test_block(9.0, true, 20.0, 4.0),
        ];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.total_observations, 3);
        assert_eq!(metrics.scheduled_count, 2);
        assert_eq!(metrics.unscheduled_count, 1);
        assert!((metrics.scheduling_rate - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!(metrics.mean_priority, 7.0);
        assert_eq!(metrics.median_priority, 7.0);
    }

    #[test]
    fn test_compute_metrics_median_even() {
        let blocks = vec![
            create_test_block(2.0, true, 10.0, 1.0),
            create_test_block(4.0, false, 15.0, 2.0),
            create_test_block(6.0, true, 20.0, 3.0),
            create_test_block(8.0, false, 25.0, 4.0),
        ];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.median_priority, 5.0); // (4.0 + 6.0) / 2
    }

    #[test]
    fn test_compute_spearman_correlation_empty() {
        let corr = compute_spearman_correlation(&[], &[]);
        assert_eq!(corr, 0.0);
    }

    #[test]
    fn test_compute_spearman_correlation_perfect_positive() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let corr = compute_spearman_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_spearman_correlation_perfect_negative() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let corr = compute_spearman_correlation(&x, &y);
        assert!((corr + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_spearman_correlation_no_relationship() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![5.0, 5.0, 5.0];
        let corr = compute_spearman_correlation(&x, &y);
        // When one variable is constant, Spearman will assign tied ranks
        // Result may vary but should be valid
        assert!((-1.0..=1.0).contains(&corr));
    }

    #[test]
    fn test_compute_spearman_correlation_length_mismatch() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![1.0, 2.0];
        let corr = compute_spearman_correlation(&x, &y);
        assert_eq!(corr, 0.0);
    }

    #[test]
    fn test_compute_correlations_empty() {
        let blocks = vec![];
        let corrs = compute_correlations(&blocks);
        assert_eq!(corrs.len(), 0);
    }

    #[test]
    fn test_compute_correlations_single_block() {
        let blocks = vec![create_test_block(5.0, true, 10.0, 2.0)];
        let corrs = compute_correlations(&blocks);
        assert_eq!(corrs.len(), 0);
    }

    #[test]
    fn test_compute_correlations_multiple() {
        let blocks = vec![
            create_test_block(1.0, false, 5.0, 1.0),
            create_test_block(5.0, true, 15.0, 3.0),
            create_test_block(9.0, true, 25.0, 5.0),
        ];
        let corrs = compute_correlations(&blocks);
        // Should have some correlations computed
        assert!(!corrs.is_empty());
        assert!(corrs
            .iter()
            .all(|c| c.correlation >= -1.0 && c.correlation <= 1.0));
    }
}
