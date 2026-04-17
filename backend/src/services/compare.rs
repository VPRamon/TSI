#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::redundant_closure)]

use crate::api::{
    CompareBlock, CompareData, CompareDiffBlock, CompareStats, RetimedBlockChange, SchedulingChange,
};
use crate::db::get_repository;
use std::collections::{HashMap, HashSet};
use tokio::runtime::Runtime;

/// Retimed-block tolerance: treat scheduled boundaries as unchanged if both
/// differ by less than one second (1 s = 1/86400 days in MJD).
const RETIMED_TOLERANCE_SECONDS: f64 = 1.0;
const SECONDS_PER_DAY: f64 = 86_400.0;

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

/// Gap metrics tuple (count, mean_hours, median_hours).
pub type GapMetrics = (Option<i32>, Option<qtty::Hours>, Option<qtty::Hours>);

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

/// Build a map keyed by non-empty `original_block_id`. Returns an error if a
/// duplicate non-empty key is seen within the same schedule.
fn index_by_original_id<'a>(
    blocks: &'a [CompareBlock],
    schedule_label: &str,
) -> Result<HashMap<String, &'a CompareBlock>, String> {
    let mut map: HashMap<String, &'a CompareBlock> = HashMap::new();
    for b in blocks {
        let key = b.original_block_id.trim();
        if key.is_empty() {
            continue;
        }
        if map.insert(key.to_string(), b).is_some() {
            return Err(format!(
                "Duplicate original_block_id '{}' in {} schedule",
                key, schedule_label
            ));
        }
    }
    Ok(map)
}

fn diff_block_from(
    id: &str,
    current: Option<&CompareBlock>,
    comparison: Option<&CompareBlock>,
) -> CompareDiffBlock {
    // Prefer metadata from whichever side has it (they should agree for matched blocks).
    let any = current
        .or(comparison)
        .expect("at least one side must exist");
    let block_name = current
        .map(|b| b.block_name.clone())
        .filter(|n| !n.is_empty())
        .or_else(|| comparison.map(|b| b.block_name.clone()))
        .unwrap_or_default();

    CompareDiffBlock {
        original_block_id: id.to_string(),
        block_name,
        priority: any.priority,
        requested_hours: any.requested_hours,
        current_scheduling_block_id: current.map(|b| b.scheduling_block_id.clone()),
        comparison_scheduling_block_id: comparison.map(|b| b.scheduling_block_id.clone()),
        current_scheduled_start_mjd: current.and_then(|b| b.scheduled_start_mjd),
        current_scheduled_stop_mjd: current.and_then(|b| b.scheduled_stop_mjd),
        comparison_scheduled_start_mjd: comparison.and_then(|b| b.scheduled_start_mjd),
        comparison_scheduled_stop_mjd: comparison.and_then(|b| b.scheduled_stop_mjd),
    }
}

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

pub fn compute_compare_data_with_gaps(
    current_blocks: Vec<CompareBlock>,
    comparison_blocks: Vec<CompareBlock>,
    current_name: String,
    comparison_name: String,
    current_gap_metrics: Option<GapMetrics>,
    comparison_gap_metrics: Option<GapMetrics>,
) -> Result<CompareData, String> {
    // Match by non-empty original_block_id.
    let current_map = index_by_original_id(&current_blocks, "current")?;
    let comparison_map = index_by_original_id(&comparison_blocks, "comparison")?;

    let current_keys: HashSet<&String> = current_map.keys().collect();
    let comparison_keys: HashSet<&String> = comparison_map.keys().collect();

    let mut common: Vec<String> = current_keys
        .intersection(&comparison_keys)
        .map(|s| (*s).clone())
        .collect();
    common.sort();

    let mut only_in_current: Vec<String> = current_keys
        .difference(&comparison_keys)
        .map(|s| (*s).clone())
        .collect();
    only_in_current.sort();

    let mut only_in_comparison: Vec<String> = comparison_keys
        .difference(&current_keys)
        .map(|s| (*s).clone())
        .collect();
    only_in_comparison.sort();

    // Legacy scheduling_changes (status flips on common ids).
    let mut scheduling_changes: Vec<SchedulingChange> = Vec::new();

    // Grouped diff outputs.
    let mut scheduled_only_current: Vec<CompareDiffBlock> = Vec::new();
    let mut scheduled_only_comparison: Vec<CompareDiffBlock> = Vec::new();
    let mut retimed_blocks: Vec<RetimedBlockChange> = Vec::new();

    for id in &common {
        let cur = current_map.get(id).copied();
        let cmp = comparison_map.get(id).copied();
        let (Some(cur_b), Some(cmp_b)) = (cur, cmp) else {
            continue;
        };

        // Status flips.
        if !cur_b.scheduled && cmp_b.scheduled {
            scheduling_changes.push(SchedulingChange {
                scheduling_block_id: id.clone(),
                priority: cmp_b.priority,
                change_type: "newly_scheduled".to_string(),
            });
            scheduled_only_comparison.push(diff_block_from(id, cur, cmp));
        } else if cur_b.scheduled && !cmp_b.scheduled {
            scheduling_changes.push(SchedulingChange {
                scheduling_block_id: id.clone(),
                priority: cur_b.priority,
                change_type: "newly_unscheduled".to_string(),
            });
            scheduled_only_current.push(diff_block_from(id, cur, cmp));
        } else if cur_b.scheduled && cmp_b.scheduled {
            // Retimed detection.
            let (Some(cs), Some(ce), Some(ms), Some(me)) = (
                cur_b.scheduled_start_mjd,
                cur_b.scheduled_stop_mjd,
                cmp_b.scheduled_start_mjd,
                cmp_b.scheduled_stop_mjd,
            ) else {
                continue;
            };
            let start_shift_seconds = (ms - cs) * SECONDS_PER_DAY;
            let stop_shift_seconds = (me - ce) * SECONDS_PER_DAY;
            if start_shift_seconds.abs() > RETIMED_TOLERANCE_SECONDS
                || stop_shift_seconds.abs() > RETIMED_TOLERANCE_SECONDS
            {
                let block_name = if !cur_b.block_name.is_empty() {
                    cur_b.block_name.clone()
                } else {
                    cmp_b.block_name.clone()
                };
                retimed_blocks.push(RetimedBlockChange {
                    original_block_id: id.clone(),
                    block_name,
                    priority: cur_b.priority,
                    requested_hours: cur_b.requested_hours,
                    current_scheduling_block_id: Some(cur_b.scheduling_block_id.clone()),
                    comparison_scheduling_block_id: Some(cmp_b.scheduling_block_id.clone()),
                    current_scheduled_start_mjd: Some(cs),
                    current_scheduled_stop_mjd: Some(ce),
                    comparison_scheduled_start_mjd: Some(ms),
                    comparison_scheduled_stop_mjd: Some(me),
                    start_shift_hours: start_shift_seconds / 3600.0,
                    stop_shift_hours: stop_shift_seconds / 3600.0,
                });
            }
        }
    }

    // Schedule-exclusive blocks.
    let mut only_in_current_blocks: Vec<CompareDiffBlock> = only_in_current
        .iter()
        .map(|id| diff_block_from(id, current_map.get(id).copied(), None))
        .collect();
    let mut only_in_comparison_blocks: Vec<CompareDiffBlock> = only_in_comparison
        .iter()
        .map(|id| diff_block_from(id, None, comparison_map.get(id).copied()))
        .collect();

    // Append schedule-exclusive blocks that are scheduled into their "scheduled only" list.
    for id in &only_in_current {
        if let Some(b) = current_map.get(id).copied() {
            if b.scheduled {
                scheduled_only_current.push(diff_block_from(id, Some(b), None));
            }
        }
    }
    for id in &only_in_comparison {
        if let Some(b) = comparison_map.get(id).copied() {
            if b.scheduled {
                scheduled_only_comparison.push(diff_block_from(id, None, Some(b)));
            }
        }
    }

    // Sort priority-based tables by descending priority.
    let by_priority_desc = |a: &CompareDiffBlock, b: &CompareDiffBlock| {
        b.priority
            .partial_cmp(&a.priority)
            .unwrap_or(std::cmp::Ordering::Equal)
    };
    scheduled_only_current.sort_by(by_priority_desc);
    scheduled_only_comparison.sort_by(by_priority_desc);
    only_in_current_blocks.sort_by(by_priority_desc);
    only_in_comparison_blocks.sort_by(by_priority_desc);

    // Sort retimed by absolute start-shift descending.
    retimed_blocks.sort_by(|a, b| {
        b.start_shift_hours
            .abs()
            .partial_cmp(&a.start_shift_hours.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let current_stats = compute_stats_with_gaps(&current_blocks, current_gap_metrics);
    let comparison_stats = compute_stats_with_gaps(&comparison_blocks, comparison_gap_metrics);

    Ok(CompareData {
        current_blocks,
        comparison_blocks,
        current_stats,
        comparison_stats,
        common_ids: common,
        only_in_current,
        only_in_comparison,
        scheduling_changes,
        scheduled_only_current,
        scheduled_only_comparison,
        only_in_current_blocks,
        only_in_comparison_blocks,
        retimed_blocks,
        current_name,
        comparison_name,
    })
}

pub async fn get_compare_data(
    current_schedule_id: crate::api::ScheduleId,
    comparison_schedule_id: crate::api::ScheduleId,
    current_name: String,
    comparison_name: String,
) -> Result<CompareData, String> {
    let repo = get_repository().map_err(|e| format!("Failed to get repository: {}", e))?;

    let current_blocks = repo
        .fetch_compare_blocks(current_schedule_id)
        .await
        .map_err(|e| format!("Failed to fetch current schedule blocks: {}", e))?;
    let comparison_blocks = repo
        .fetch_compare_blocks(comparison_schedule_id)
        .await
        .map_err(|e| format!("Failed to fetch comparison schedule blocks: {}", e))?;

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

pub fn py_get_compare_data(
    current_schedule_id: crate::api::ScheduleId,
    comparison_schedule_id: crate::api::ScheduleId,
    current_name: String,
    comparison_name: String,
) -> Result<CompareData, String> {
    let runtime = Runtime::new().map_err(|e| format!("Failed to create async runtime: {}", e))?;
    runtime.block_on(get_compare_data(
        current_schedule_id,
        comparison_schedule_id,
        current_name,
        comparison_name,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(
        sbid: &str,
        original: &str,
        priority: f64,
        scheduled: bool,
        hours: f64,
        window: Option<(f64, f64)>,
    ) -> CompareBlock {
        CompareBlock {
            scheduling_block_id: sbid.to_string(),
            original_block_id: original.to_string(),
            block_name: format!("name-{original}"),
            priority,
            scheduled,
            requested_hours: qtty::Hours::new(hours),
            scheduled_start_mjd: window.map(|(s, _)| s),
            scheduled_stop_mjd: window.map(|(_, e)| e),
        }
    }

    #[test]
    fn test_compute_stats_mixed() {
        let blocks = vec![
            block("1", "a", 3.0, true, 1.0, None),
            block("2", "b", 5.0, false, 2.0, None),
            block("3", "c", 7.0, true, 3.0, None),
        ];
        let stats = compute_stats(&blocks);
        assert_eq!(stats.scheduled_count, 2);
        assert_eq!(stats.unscheduled_count, 1);
        assert_eq!(stats.total_priority, 10.0);
    }

    #[test]
    fn matches_by_original_block_id_not_position() {
        // Note: different scheduling_block_ids and different positions.
        let current = vec![
            block("11", "A", 5.0, true, 1.0, Some((100.0, 100.1))),
            block("12", "B", 6.0, false, 2.0, None),
        ];
        let comparison = vec![
            block("22", "B", 6.0, true, 2.0, Some((200.0, 200.1))),
            block("21", "A", 5.0, true, 1.0, Some((100.0, 100.1))),
        ];
        let data = compute_compare_data(current, comparison, "A".into(), "B".into()).unwrap();
        assert_eq!(data.common_ids, vec!["A", "B"]);
        assert!(data.only_in_current.is_empty());
        assert!(data.only_in_comparison.is_empty());
        // B was newly scheduled.
        assert_eq!(data.scheduling_changes.len(), 1);
        assert_eq!(data.scheduling_changes[0].change_type, "newly_scheduled");
    }

    #[test]
    fn empty_original_ids_are_never_matched() {
        let current = vec![block("1", "", 5.0, true, 1.0, Some((10.0, 10.1)))];
        let comparison = vec![block("2", "", 5.0, true, 1.0, Some((10.0, 10.1)))];
        let data = compute_compare_data(current, comparison, "A".into(), "B".into()).unwrap();
        assert!(data.common_ids.is_empty());
        assert!(data.only_in_current.is_empty());
        assert!(data.only_in_comparison.is_empty());
    }

    #[test]
    fn duplicate_original_id_is_rejected() {
        let current = vec![
            block("1", "dup", 1.0, true, 1.0, Some((10.0, 10.1))),
            block("2", "dup", 2.0, true, 1.0, Some((11.0, 11.1))),
        ];
        let comparison: Vec<CompareBlock> = vec![];
        let err = compute_compare_data(current, comparison, "A".into(), "B".into()).unwrap_err();
        assert!(err.contains("Duplicate"));
        assert!(err.contains("current"));
    }

    #[test]
    fn scheduled_only_groups_cover_flips_and_exclusives() {
        let current = vec![
            block("1", "common_flip", 3.0, true, 1.0, Some((10.0, 10.1))),
            block(
                "2",
                "current_only_sched",
                4.0,
                true,
                1.5,
                Some((12.0, 12.1)),
            ),
            block("3", "current_only_unsched", 2.0, false, 1.0, None),
        ];
        let comparison = vec![
            block("11", "common_flip", 3.0, false, 1.0, None),
            block("12", "cmp_only_sched", 5.0, true, 2.0, Some((20.0, 20.2))),
        ];
        let data = compute_compare_data(current, comparison, "A".into(), "B".into()).unwrap();

        let cur_only_sched_ids: Vec<_> = data
            .scheduled_only_current
            .iter()
            .map(|b| b.original_block_id.clone())
            .collect();
        assert!(cur_only_sched_ids.contains(&"common_flip".to_string()));
        assert!(cur_only_sched_ids.contains(&"current_only_sched".to_string()));

        let cmp_only_sched_ids: Vec<_> = data
            .scheduled_only_comparison
            .iter()
            .map(|b| b.original_block_id.clone())
            .collect();
        assert!(cmp_only_sched_ids.contains(&"cmp_only_sched".to_string()));
    }

    #[test]
    fn retimed_detection_respects_one_second_tolerance() {
        let tiny = 0.5 / SECONDS_PER_DAY; // 0.5 s in MJD days
        let big = 120.0 / SECONDS_PER_DAY; // 120 s

        let current = vec![
            block("1", "sub_tol", 1.0, true, 1.0, Some((100.0, 100.1))),
            block("2", "over_tol", 2.0, true, 1.0, Some((200.0, 200.1))),
        ];
        let comparison = vec![
            block(
                "11",
                "sub_tol",
                1.0,
                true,
                1.0,
                Some((100.0 + tiny, 100.1 + tiny)),
            ),
            block(
                "12",
                "over_tol",
                2.0,
                true,
                1.0,
                Some((200.0 + big, 200.1 + big)),
            ),
        ];
        let data = compute_compare_data(current, comparison, "A".into(), "B".into()).unwrap();
        assert_eq!(data.retimed_blocks.len(), 1);
        assert_eq!(data.retimed_blocks[0].original_block_id, "over_tol");
        assert!((data.retimed_blocks[0].start_shift_hours - 120.0 / 3600.0).abs() < 1e-9);
    }
}
