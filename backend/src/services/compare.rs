#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::redundant_closure)]

use crate::api::{CompareBlock, CompareData, CompareStats, SchedulingChange};
use crate::db::get_repository;
use std::collections::{HashMap, HashSet};
use tokio::runtime::Runtime;

/// Compute statistics for a set of blocks.
pub(crate) fn compute_stats(blocks: &[CompareBlock]) -> CompareStats {
    let scheduled_blocks: Vec<&CompareBlock> = blocks.iter().filter(|b| b.scheduled).collect();

    let scheduled_count = scheduled_blocks.len();
    let unscheduled_count = blocks.len() - scheduled_count;

    if scheduled_blocks.is_empty() {
        return CompareStats {
            scheduled_count,
            unscheduled_count,
            total_priority: 0.0,
            mean_priority: 0.0,
            median_priority: 0.0,
            total_hours: qtty::Hours::new(0.0),
            gap_count: None,
            gap_mean_hours: None,
            gap_median_hours: None,
        };
    }

    let priorities: Vec<f64> = scheduled_blocks.iter().map(|b| b.priority).collect();
    let total_priority: f64 = priorities.iter().sum();
    let mean_priority = total_priority / scheduled_count as f64;

    let mut sorted_priorities = priorities.clone();
    sorted_priorities.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_priority = if scheduled_count % 2 == 0 {
        (sorted_priorities[scheduled_count / 2 - 1] + sorted_priorities[scheduled_count / 2]) / 2.0
    } else {
        sorted_priorities[scheduled_count / 2]
    };

    let total_hours_f64: f64 = scheduled_blocks
        .iter()
        .map(|b| b.requested_hours.value())
        .sum();

    CompareStats {
        scheduled_count,
        unscheduled_count,
        total_priority,
        mean_priority,
        median_priority,
        total_hours: qtty::Hours::new(total_hours_f64),
        gap_count: None,
        gap_mean_hours: None,
        gap_median_hours: None,
    }
}

/// Gap metrics tuple (count, mean_hours, median_hours)
pub type GapMetrics = (Option<i32>, Option<qtty::Hours>, Option<qtty::Hours>);

/// Compute statistics for a set of blocks with gap metrics from summary analytics.
pub(crate) fn compute_stats_with_gaps(
    blocks: &[CompareBlock],
    gap_metrics: Option<GapMetrics>,
) -> CompareStats {
    let mut stats = compute_stats(blocks);

    if let Some((gap_count, gap_mean_hours, gap_median_hours)) = gap_metrics {
        stats.gap_count = gap_count;
        stats.gap_mean_hours = gap_mean_hours;
        stats.gap_median_hours = gap_median_hours;
    }

    stats
}

/// Compute comparison data from two sets of blocks.
/// This function takes blocks from both schedules and computes all necessary statistics and changes.
pub fn compute_compare_data(
    current_blocks: Vec<CompareBlock>,
    comparison_blocks: Vec<CompareBlock>,
    current_name: String,
    comparison_name: String,
) -> Result<CompareData, String> {
    compute_compare_data_with_gaps(
        current_blocks,
        comparison_blocks,
        current_name,
        comparison_name,
        None,
        None,
    )
}

/// Compute comparison data from two sets of blocks with gap metrics.
pub fn compute_compare_data_with_gaps(
    current_blocks: Vec<CompareBlock>,
    comparison_blocks: Vec<CompareBlock>,
    current_name: String,
    comparison_name: String,
    current_gap_metrics: Option<GapMetrics>,
    comparison_gap_metrics: Option<GapMetrics>,
) -> Result<CompareData, String> {
    // Create ID sets for comparison
    let current_ids: HashSet<String> = current_blocks
        .iter()
        .map(|b| b.scheduling_block_id.clone())
        .collect();

    let comparison_ids: HashSet<String> = comparison_blocks
        .iter()
        .map(|b| b.scheduling_block_id.clone())
        .collect();

    // Find differences
    let only_in_current: Vec<String> = current_ids.difference(&comparison_ids).cloned().collect();

    let only_in_comparison: Vec<String> =
        comparison_ids.difference(&current_ids).cloned().collect();

    let common_ids: Vec<String> = current_ids.intersection(&comparison_ids).cloned().collect();

    // Create maps for efficient lookup
    let current_map: HashMap<String, &CompareBlock> = current_blocks
        .iter()
        .map(|b| (b.scheduling_block_id.clone(), b))
        .collect();

    let comparison_map: HashMap<String, &CompareBlock> = comparison_blocks
        .iter()
        .map(|b| (b.scheduling_block_id.clone(), b))
        .collect();

    // Find scheduling changes
    let mut scheduling_changes = Vec::new();
    for id in &common_ids {
        if let (Some(current_block), Some(comparison_block)) =
            (current_map.get(id), comparison_map.get(id))
        {
            // Newly scheduled: was unscheduled in current, scheduled in comparison
            if !current_block.scheduled && comparison_block.scheduled {
                scheduling_changes.push(SchedulingChange {
                    scheduling_block_id: id.clone(),
                    priority: comparison_block.priority,
                    change_type: "newly_scheduled".to_string(),
                });
            }

            // Newly unscheduled: was scheduled in current, unscheduled in comparison
            if current_block.scheduled && !comparison_block.scheduled {
                scheduling_changes.push(SchedulingChange {
                    scheduling_block_id: id.clone(),
                    priority: current_block.priority,
                    change_type: "newly_unscheduled".to_string(),
                });
            }
        }
    }

    // Compute statistics with gap metrics
    let current_stats = compute_stats_with_gaps(&current_blocks, current_gap_metrics);
    let comparison_stats = compute_stats_with_gaps(&comparison_blocks, comparison_gap_metrics);

    Ok(CompareData {
        current_blocks,
        comparison_blocks,
        current_stats,
        comparison_stats,
        common_ids,
        only_in_current,
        only_in_comparison,
        scheduling_changes,
        current_name,
        comparison_name,
    })
}

/// Get comparison data from the database by fetching both schedules.
pub async fn get_compare_data(
    current_schedule_id: crate::api::ScheduleId,
    comparison_schedule_id: crate::api::ScheduleId,
    current_name: String,
    comparison_name: String,
) -> Result<CompareData, String> {
    // Use the initialized repository (local by default)
    let repo = get_repository().map_err(|e| format!("Failed to get repository: {}", e))?;

    // Fetch blocks from both schedules
    let current_blocks = repo
        .fetch_compare_blocks(current_schedule_id)
        .await
        .map_err(|e| format!("Failed to fetch current schedule blocks: {}", e))?;
    let comparison_blocks = repo
        .fetch_compare_blocks(comparison_schedule_id)
        .await
        .map_err(|e| format!("Failed to fetch comparison schedule blocks: {}", e))?;

    // Fetch gap metrics from summary analytics
    let current_gap_metrics = repo.fetch_gap_metrics(current_schedule_id).await.ok();
    let comparison_gap_metrics = repo.fetch_gap_metrics(comparison_schedule_id).await.ok();

    compute_compare_data_with_gaps(
        current_blocks,
        comparison_blocks,
        current_name,
        comparison_name,
        current_gap_metrics,
        comparison_gap_metrics,
    )
}

/// Fetches and compares two schedules from the database.
pub fn py_get_compare_data(
    current_schedule_id: crate::api::ScheduleId,
    comparison_schedule_id: crate::api::ScheduleId,
    current_name: String,
    comparison_name: String,
) -> Result<CompareData, String> {
    let runtime = Runtime::new().map_err(|e| {
        format!("Failed to create async runtime: {}", e)
    })?;

    runtime
        .block_on(get_compare_data(
            current_schedule_id,
            comparison_schedule_id,
            current_name,
            comparison_name,
        ))
}

#[cfg(test)]
mod tests {
    use super::{compute_compare_data, compute_stats};
    use crate::api::CompareBlock;

    fn create_test_block(id: &str, priority: f64, scheduled: bool, hours: f64) -> CompareBlock {
        CompareBlock {
            scheduling_block_id: id.to_string(),
            priority,
            scheduled,
            requested_hours: qtty::Hours::new(hours),
        }
    }

    #[test]
    fn test_compute_stats_empty() {
        let blocks = vec![];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 0);
        assert_eq!(stats.unscheduled_count, 0);
        assert_eq!(stats.total_priority, 0.0);
        assert_eq!(stats.mean_priority, 0.0);
        assert_eq!(stats.median_priority, 0.0);
        assert_eq!(stats.total_hours.value(), 0.0);
    }

    #[test]
    fn test_compute_stats_all_unscheduled() {
        let blocks = vec![
            create_test_block("b1", 5.0, false, 1.0),
            create_test_block("b2", 7.0, false, 2.0),
        ];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 0);
        assert_eq!(stats.unscheduled_count, 2);
        assert_eq!(stats.total_priority, 0.0);
        assert_eq!(stats.mean_priority, 0.0);
        assert_eq!(stats.median_priority, 0.0);
        assert_eq!(stats.total_hours.value(), 0.0);
    }

    #[test]
    fn test_compute_stats_single_scheduled() {
        let blocks = vec![create_test_block("b1", 8.5, true, 3.5)];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 1);
        assert_eq!(stats.unscheduled_count, 0);
        assert_eq!(stats.total_priority, 8.5);
        assert_eq!(stats.mean_priority, 8.5);
        assert_eq!(stats.median_priority, 8.5);
        assert_eq!(stats.total_hours.value(), 3.5);
    }

    #[test]
    fn test_compute_stats_odd_count() {
        let blocks = vec![
            create_test_block("b1", 3.0, true, 1.0),
            create_test_block("b2", 5.0, true, 2.0),
            create_test_block("b3", 7.0, true, 3.0),
        ];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 3);
        assert_eq!(stats.unscheduled_count, 0);
        assert_eq!(stats.total_priority, 15.0);
        assert_eq!(stats.mean_priority, 5.0);
        assert_eq!(stats.median_priority, 5.0); // Middle value
        assert_eq!(stats.total_hours.value(), 6.0);
    }

    #[test]
    fn test_compute_stats_even_count() {
        let blocks = vec![
            create_test_block("b1", 2.0, true, 1.0),
            create_test_block("b2", 4.0, true, 1.5),
            create_test_block("b3", 6.0, true, 2.0),
            create_test_block("b4", 8.0, true, 2.5),
        ];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 4);
        assert_eq!(stats.unscheduled_count, 0);
        assert_eq!(stats.total_priority, 20.0);
        assert_eq!(stats.mean_priority, 5.0);
        assert_eq!(stats.median_priority, 5.0); // (4.0 + 6.0) / 2
        assert_eq!(stats.total_hours.value(), 7.0);
    }

    #[test]
    fn test_compute_stats_mixed() {
        let blocks = vec![
            create_test_block("b1", 3.0, true, 1.0),
            create_test_block("b2", 5.0, false, 2.0),
            create_test_block("b3", 7.0, true, 3.0),
            create_test_block("b4", 9.0, false, 4.0),
        ];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 2);
        assert_eq!(stats.unscheduled_count, 2);
        assert_eq!(stats.total_priority, 10.0);
        assert_eq!(stats.mean_priority, 5.0);
        assert_eq!(stats.median_priority, 5.0); // (3.0 + 7.0) / 2
        assert_eq!(stats.total_hours.value(), 4.0);
    }

    #[test]
    fn test_compute_compare_data_empty() {
        let result = compute_compare_data(vec![], vec![], "Current".into(), "Comparison".into());

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.current_blocks.len(), 0);
        assert_eq!(data.comparison_blocks.len(), 0);
        assert_eq!(data.common_ids.len(), 0);
        assert_eq!(data.only_in_current.len(), 0);
        assert_eq!(data.only_in_comparison.len(), 0);
        assert_eq!(data.scheduling_changes.len(), 0);
    }

    #[test]
    fn test_compute_compare_data_identical() {
        let current = vec![
            create_test_block("b1", 5.0, true, 1.0),
            create_test_block("b2", 7.0, false, 2.0),
        ];
        let comparison = vec![
            create_test_block("b1", 5.0, true, 1.0),
            create_test_block("b2", 7.0, false, 2.0),
        ];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.common_ids.len(), 2);
        assert_eq!(data.only_in_current.len(), 0);
        assert_eq!(data.only_in_comparison.len(), 0);
        assert_eq!(data.scheduling_changes.len(), 0);
    }

    #[test]
    fn test_compute_compare_data_newly_scheduled() {
        let current = vec![create_test_block("b1", 5.0, false, 1.0)];
        let comparison = vec![create_test_block("b1", 5.0, true, 1.0)];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.scheduling_changes.len(), 1);
        assert_eq!(data.scheduling_changes[0].change_type, "newly_scheduled");
        assert_eq!(data.scheduling_changes[0].priority, 5.0);
    }

    #[test]
    fn test_compute_compare_data_newly_unscheduled() {
        let current = vec![create_test_block("b1", 8.0, true, 2.0)];
        let comparison = vec![create_test_block("b1", 8.0, false, 2.0)];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.scheduling_changes.len(), 1);
        assert_eq!(data.scheduling_changes[0].change_type, "newly_unscheduled");
        assert_eq!(data.scheduling_changes[0].priority, 8.0);
    }

    #[test]
    fn test_compute_compare_data_only_in_current() {
        let current = vec![
            create_test_block("b1", 5.0, true, 1.0),
            create_test_block("b2", 7.0, false, 2.0),
        ];
        let comparison = vec![create_test_block("b1", 5.0, true, 1.0)];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.only_in_current.len(), 1);
        assert!(data.only_in_current.contains(&"b2".to_string()));
        assert_eq!(data.only_in_comparison.len(), 0);
    }

    #[test]
    fn test_compute_compare_data_only_in_comparison() {
        let current = vec![create_test_block("b1", 5.0, true, 1.0)];
        let comparison = vec![
            create_test_block("b1", 5.0, true, 1.0),
            create_test_block("b3", 9.0, false, 3.0),
        ];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.only_in_current.len(), 0);
        assert_eq!(data.only_in_comparison.len(), 1);
        assert!(data.only_in_comparison.contains(&"b3".to_string()));
    }

    #[test]
    fn test_compute_compare_data_complex() {
        let current = vec![
            create_test_block("common1", 3.0, false, 1.0),
            create_test_block("common2", 5.0, true, 2.0),
            create_test_block("only_current", 7.0, true, 3.0),
        ];
        let comparison = vec![
            create_test_block("common1", 3.0, true, 1.0), // Newly scheduled
            create_test_block("common2", 5.0, false, 2.0), // Newly unscheduled
            create_test_block("only_comparison", 9.0, false, 4.0),
        ];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.common_ids.len(), 2);
        assert_eq!(data.only_in_current.len(), 1);
        assert_eq!(data.only_in_comparison.len(), 1);
        assert_eq!(data.scheduling_changes.len(), 2);

        // Check scheduling changes
        let newly_scheduled = data
            .scheduling_changes
            .iter()
            .find(|c| c.change_type == "newly_scheduled");
        assert!(newly_scheduled.is_some());
        assert_eq!(
            newly_scheduled.unwrap().scheduling_block_id,
            "common1".to_string()
        );

        let newly_unscheduled = data
            .scheduling_changes
            .iter()
            .find(|c| c.change_type == "newly_unscheduled");
        assert!(newly_unscheduled.is_some());
        assert_eq!(
            newly_unscheduled.unwrap().scheduling_block_id,
            "common2".to_string()
        );
    }
}
