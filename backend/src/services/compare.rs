#![allow(clippy::manual_is_multiple_of)]
#![allow(clippy::redundant_closure)]

use crate::api::{
    AdvancedCompare, AdvancedCompareParams, AdvancedGlobalMetrics, CoherentBlock, CompareBlock,
    CompareData, CompareDiffBlock, CompareStats, RetimedBlockChange, SchedulingChange,
};
use crate::db::FullRepository;
use std::collections::{HashMap, HashSet};

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
    epsilon_minutes: Option<f64>,
    min_block_size: Option<usize>,
    merge_epsilon_minutes: Option<f64>,
) -> Result<CompareData, String> {
    compute_compare_data_with_gaps(
        current_blocks,
        comparison_blocks,
        current_name,
        comparison_name,
        None,
        None,
        epsilon_minutes,
        min_block_size,
        merge_epsilon_minutes,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn compute_compare_data_with_gaps(
    current_blocks: Vec<CompareBlock>,
    comparison_blocks: Vec<CompareBlock>,
    current_name: String,
    comparison_name: String,
    current_gap_metrics: Option<GapMetrics>,
    comparison_gap_metrics: Option<GapMetrics>,
    epsilon_minutes: Option<f64>,
    min_block_size: Option<usize>,
    merge_epsilon_minutes: Option<f64>,
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

    let advanced_compare = compute_advanced_compare(
        &current_blocks,
        &comparison_blocks,
        &common,
        &only_in_current,
        &only_in_comparison,
        &current_map,
        &comparison_map,
        epsilon_minutes,
        min_block_size,
        merge_epsilon_minutes,
    );

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
        advanced_compare,
    })
}

// =========================================================
// Advanced coherent-block comparison pipeline
// =========================================================

const DEFAULT_EPSILON_MINUTES: f64 = 5.0;
const DEFAULT_MIN_BLOCK_SIZE: usize = 3;
const MJD_DAYS_TO_MINUTES: f64 = 24.0 * 60.0;

/// Per-task record for the timed-common subset.
#[derive(Debug, Clone)]
struct TimedTask {
    original_block_id: String,
    pos_a: usize,
    pos_b: usize,
    shift_minutes: f64,
    start_a_mjd: f64,
    stop_a_mjd: f64,
    start_b_mjd: f64,
    stop_b_mjd: f64,
}

/// Longest Increasing Subsequence length in O(N log N).
fn lis_length(seq: &[usize]) -> usize {
    // tails[i] = smallest tail element of all increasing subsequences of length i+1
    let mut tails: Vec<usize> = Vec::new();
    for &val in seq {
        let pos = tails.partition_point(|&t| t < val);
        if pos == tails.len() {
            tails.push(val);
        } else {
            tails[pos] = val;
        }
    }
    tails.len()
}

fn median_f64(vals: &mut [f64]) -> Option<f64> {
    if vals.is_empty() {
        return None;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = vals.len();
    if n % 2 == 0 {
        Some((vals[n / 2 - 1] + vals[n / 2]) / 2.0)
    } else {
        Some(vals[n / 2])
    }
}

fn std_dev(vals: &[f64], mean: f64) -> f64 {
    if vals.len() < 2 {
        return 0.0;
    }
    let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vals.len() as f64;
    var.sqrt()
}

#[allow(clippy::too_many_arguments)]
fn compute_advanced_compare(
    current_blocks: &[CompareBlock],
    comparison_blocks: &[CompareBlock],
    common: &[String],
    only_in_current: &[String],
    only_in_comparison: &[String],
    current_map: &HashMap<String, &CompareBlock>,
    comparison_map: &HashMap<String, &CompareBlock>,
    epsilon_minutes: Option<f64>,
    min_block_size: Option<usize>,
    merge_epsilon_minutes: Option<f64>,
) -> AdvancedCompare {
    let eps = epsilon_minutes.unwrap_or(DEFAULT_EPSILON_MINUTES);
    let min_bs = min_block_size.unwrap_or(DEFAULT_MIN_BLOCK_SIZE);
    let merge_eps = merge_epsilon_minutes.unwrap_or(eps);

    let params = AdvancedCompareParams {
        epsilon_minutes: eps,
        min_block_size: min_bs,
        merge_epsilon_minutes: merge_eps,
    };

    let ignored_missing_key_current = current_blocks
        .iter()
        .filter(|b| b.original_block_id.trim().is_empty())
        .count();
    let ignored_missing_key_comparison = comparison_blocks
        .iter()
        .filter(|b| b.original_block_id.trim().is_empty())
        .count();

    // Build position maps: original_block_id → position in sorted schedule A/B.
    // We sort scheduled blocks by start_mjd for deterministic ordering.
    let pos_map_a = build_position_map(current_blocks);
    let pos_map_b = build_position_map(comparison_blocks);

    // Build timed_common: common tasks with valid windows in both schedules.
    let mut timed: Vec<TimedTask> = Vec::new();
    for id in common {
        let (Some(ca), Some(cb)) = (current_map.get(id), comparison_map.get(id)) else {
            continue;
        };
        let (Some(sa), Some(ea)) = (ca.scheduled_start_mjd, ca.scheduled_stop_mjd) else {
            continue;
        };
        let (Some(sb), Some(eb)) = (cb.scheduled_start_mjd, cb.scheduled_stop_mjd) else {
            continue;
        };
        let pa = match pos_map_a.get(id.as_str()) {
            Some(&p) => p,
            None => continue,
        };
        let pb = match pos_map_b.get(id.as_str()) {
            Some(&p) => p,
            None => continue,
        };
        let shift = (sb - sa) * MJD_DAYS_TO_MINUTES;
        timed.push(TimedTask {
            original_block_id: id.clone(),
            pos_a: pa,
            pos_b: pb,
            shift_minutes: shift,
            start_a_mjd: sa,
            stop_a_mjd: ea,
            start_b_mjd: sb,
            stop_b_mjd: eb,
        });
    }

    // Sort by pos_a for segmentation.
    timed.sort_by_key(|t| t.pos_a);

    let matched_count = common.len();
    let timed_common_count = timed.len();
    let universe = matched_count + only_in_current.len() + only_in_comparison.len();
    let match_ratio = if universe > 0 {
        matched_count as f64 / universe as f64
    } else {
        0.0
    };

    if timed_common_count == 0 {
        return AdvancedCompare {
            params_used: params,
            global_metrics: AdvancedGlobalMetrics {
                match_ratio,
                matched_count,
                timed_common_count: 0,
                only_in_current_count: only_in_current.len(),
                only_in_comparison_count: only_in_comparison.len(),
                coherent_block_count: 0,
                ungrouped_common_count: 0,
                order_preservation_ratio: None,
                global_shift_median_minutes: None,
                local_shift_mad_minutes: None,
                ignored_missing_key_current,
                ignored_missing_key_comparison,
            },
            blocks: Vec::new(),
        };
    }

    // Order preservation ratio via LIS on pos_b ordered by pos_a.
    let pos_b_seq: Vec<usize> = timed.iter().map(|t| t.pos_b).collect();
    let lis_len = lis_length(&pos_b_seq);
    let order_preservation_ratio = lis_len as f64 / timed_common_count as f64;

    // Global shift median.
    let mut shifts: Vec<f64> = timed.iter().map(|t| t.shift_minutes).collect();
    let global_shift_median = median_f64(&mut shifts).unwrap_or(0.0);

    // Local shift MAD (median absolute deviation from global median).
    let mut deviations: Vec<f64> = timed
        .iter()
        .map(|t| (t.shift_minutes - global_shift_median).abs())
        .collect();
    let local_shift_mad = median_f64(&mut deviations).unwrap_or(0.0);

    // Segmentation: scan by pos_a, keep block while pos_b stays strictly
    // increasing and |shift[i+1] - shift[i]| <= epsilon.
    let mut raw_blocks: Vec<Vec<usize>> = Vec::new(); // indices into `timed`
    let mut current_block: Vec<usize> = vec![0];

    for i in 1..timed.len() {
        let prev = &timed[current_block[current_block.len() - 1]];
        let curr = &timed[i];
        let order_ok = curr.pos_b > prev.pos_b;
        let shift_ok = (curr.shift_minutes - prev.shift_minutes).abs() <= eps;
        if order_ok && shift_ok {
            current_block.push(i);
        } else {
            raw_blocks.push(std::mem::take(&mut current_block));
            current_block = vec![i];
        }
    }
    if !current_block.is_empty() {
        raw_blocks.push(current_block);
    }

    // Filter by min_block_size.
    let mut survivors: Vec<Vec<usize>> = Vec::new();
    let mut ungrouped_count: usize = 0;
    for blk in raw_blocks {
        if blk.len() >= min_bs {
            survivors.push(blk);
        } else {
            ungrouped_count += blk.len();
        }
    }

    // Merge adjacent blocks if avg_shift diff <= merge_eps and no order inversion.
    let mut merged = merge_adjacent_blocks(&timed, survivors, merge_eps);

    // Build output blocks.
    let coherent_blocks: Vec<CoherentBlock> = merged
        .drain(..)
        .enumerate()
        .map(|(idx, indices)| build_coherent_block(idx, &indices, &timed))
        .collect();

    AdvancedCompare {
        params_used: params,
        global_metrics: AdvancedGlobalMetrics {
            match_ratio,
            matched_count,
            timed_common_count,
            only_in_current_count: only_in_current.len(),
            only_in_comparison_count: only_in_comparison.len(),
            coherent_block_count: coherent_blocks.len(),
            ungrouped_common_count: ungrouped_count,
            order_preservation_ratio: Some(order_preservation_ratio),
            global_shift_median_minutes: Some(global_shift_median),
            local_shift_mad_minutes: Some(local_shift_mad),
            ignored_missing_key_current,
            ignored_missing_key_comparison,
        },
        blocks: coherent_blocks,
    }
}

/// Build a position map: for each scheduled block with a valid start time,
/// assign a position index based on `scheduled_start_mjd` ascending order.
fn build_position_map(blocks: &[CompareBlock]) -> HashMap<&str, usize> {
    let mut scheduled: Vec<(&str, f64)> = blocks
        .iter()
        .filter_map(|b| {
            let id = b.original_block_id.trim();
            if id.is_empty() {
                return None;
            }
            b.scheduled_start_mjd.map(|s| (id, s))
        })
        .collect();
    // Stable sort by start time, tie-break by ID.
    scheduled.sort_by(|a, b| {
        a.1.partial_cmp(&b.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(b.0))
    });
    scheduled
        .into_iter()
        .enumerate()
        .map(|(i, (id, _))| (id, i))
        .collect()
}

fn avg_shift_of(timed: &[TimedTask], indices: &[usize]) -> f64 {
    if indices.is_empty() {
        return 0.0;
    }
    let sum: f64 = indices.iter().map(|&i| timed[i].shift_minutes).sum();
    sum / indices.len() as f64
}

fn merge_adjacent_blocks(
    timed: &[TimedTask],
    mut blocks: Vec<Vec<usize>>,
    merge_eps: f64,
) -> Vec<Vec<usize>> {
    if blocks.len() < 2 {
        return blocks;
    }
    let mut merged: Vec<Vec<usize>> = Vec::new();
    merged.push(blocks.remove(0));
    for blk in blocks {
        let prev = merged.last().unwrap();
        let prev_avg = avg_shift_of(timed, prev);
        let curr_avg = avg_shift_of(timed, &blk);

        let prev_last_pos_b = timed[*prev.last().unwrap()].pos_b;
        let curr_first_pos_b = timed[blk[0]].pos_b;
        let no_inversion = curr_first_pos_b > prev_last_pos_b;
        let close_shift = (curr_avg - prev_avg).abs() <= merge_eps;

        if no_inversion && close_shift {
            merged.last_mut().unwrap().extend(blk);
        } else {
            merged.push(blk);
        }
    }
    merged
}

fn build_coherent_block(idx: usize, indices: &[usize], timed: &[TimedTask]) -> CoherentBlock {
    let tasks: Vec<&TimedTask> = indices.iter().map(|&i| &timed[i]).collect();
    let ids: Vec<String> = tasks.iter().map(|t| t.original_block_id.clone()).collect();
    let size = tasks.len();
    let shifts: Vec<f64> = tasks.iter().map(|t| t.shift_minutes).collect();
    let avg_shift = shifts.iter().sum::<f64>() / size as f64;
    let shift_std = std_dev(&shifts, avg_shift);

    CoherentBlock {
        block_index: idx,
        original_block_ids: ids,
        size,
        pos_a_start: tasks.first().unwrap().pos_a,
        pos_a_end: tasks.last().unwrap().pos_a,
        pos_b_start: tasks.iter().map(|t| t.pos_b).min().unwrap(),
        pos_b_end: tasks.iter().map(|t| t.pos_b).max().unwrap(),
        start_a_mjd: tasks
            .iter()
            .map(|t| t.start_a_mjd)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
        end_a_mjd: tasks
            .iter()
            .map(|t| t.stop_a_mjd)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
        start_b_mjd: tasks
            .iter()
            .map(|t| t.start_b_mjd)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
        end_b_mjd: tasks
            .iter()
            .map(|t| t.stop_b_mjd)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap(),
        avg_shift_minutes: avg_shift,
        shift_std_minutes: shift_std,
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn get_compare_data(
    repo: &(dyn FullRepository + 'static),
    current_schedule_id: crate::api::ScheduleId,
    comparison_schedule_id: crate::api::ScheduleId,
    current_name: String,
    comparison_name: String,
    epsilon_minutes: Option<f64>,
    min_block_size: Option<usize>,
    merge_epsilon_minutes: Option<f64>,
) -> Result<CompareData, String> {
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
        epsilon_minutes,
        min_block_size,
        merge_epsilon_minutes,
    )
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

    fn ccd(
        current: Vec<CompareBlock>,
        comparison: Vec<CompareBlock>,
    ) -> Result<CompareData, String> {
        compute_compare_data(
            current,
            comparison,
            "A".into(),
            "B".into(),
            None,
            None,
            None,
        )
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
        let data = ccd(current, comparison).unwrap();
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
        let data = ccd(current, comparison).unwrap();
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
        let err = ccd(current, comparison).unwrap_err();
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
        let data = ccd(current, comparison).unwrap();

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
        let data = ccd(current, comparison).unwrap();
        assert_eq!(data.retimed_blocks.len(), 1);
        assert_eq!(data.retimed_blocks[0].original_block_id, "over_tol");
        assert!((data.retimed_blocks[0].start_shift_hours - 120.0 / 3600.0).abs() < 1e-9);
    }

    // =========================================================
    // Advanced compare pipeline tests
    // =========================================================

    /// Helper: build CompareData with specific advanced params.
    fn ccd_adv(
        current: Vec<CompareBlock>,
        comparison: Vec<CompareBlock>,
        eps: Option<f64>,
        min_bs: Option<usize>,
        merge_eps: Option<f64>,
    ) -> Result<CompareData, String> {
        compute_compare_data(
            current,
            comparison,
            "A".into(),
            "B".into(),
            eps,
            min_bs,
            merge_eps,
        )
    }

    #[test]
    fn lis_length_basic() {
        assert_eq!(lis_length(&[]), 0);
        assert_eq!(lis_length(&[3, 1, 2, 4]), 3); // 1,2,4
        assert_eq!(lis_length(&[0, 1, 2, 3]), 4);
        assert_eq!(lis_length(&[3, 2, 1, 0]), 1);
    }

    #[test]
    fn constant_shift_forms_single_block() {
        // 5 tasks shifted by exactly 10 minutes → one block.
        let shift_days = 10.0 / MJD_DAYS_TO_MINUTES;
        let current: Vec<CompareBlock> = (0..5)
            .map(|i| {
                let s = 100.0 + i as f64 * 0.1;
                block(
                    &format!("c{i}"),
                    &format!("T{i}"),
                    1.0,
                    true,
                    1.0,
                    Some((s, s + 0.05)),
                )
            })
            .collect();
        let comparison: Vec<CompareBlock> = (0..5)
            .map(|i| {
                let s = 100.0 + i as f64 * 0.1 + shift_days;
                block(
                    &format!("m{i}"),
                    &format!("T{i}"),
                    1.0,
                    true,
                    1.0,
                    Some((s, s + 0.05)),
                )
            })
            .collect();
        let data = ccd(current, comparison).unwrap();
        let adv = &data.advanced_compare;

        assert_eq!(adv.global_metrics.timed_common_count, 5);
        assert_eq!(adv.global_metrics.coherent_block_count, 1);
        assert_eq!(adv.global_metrics.ungrouped_common_count, 0);
        assert!((adv.global_metrics.order_preservation_ratio.unwrap() - 1.0).abs() < 1e-9);
        assert_eq!(adv.blocks[0].size, 5);
        assert!((adv.blocks[0].avg_shift_minutes - 10.0).abs() < 1e-6);
        assert!(adv.blocks[0].shift_std_minutes < 1e-6);
    }

    #[test]
    fn insertion_at_start_preserves_block() {
        // B inserts a new task at position 0 but keeps the rest in order
        // with same shift. The inserted task won't be in common.
        let shift_days = 5.0 / MJD_DAYS_TO_MINUTES;
        let current: Vec<CompareBlock> = (0..4)
            .map(|i| {
                let s = 100.0 + i as f64 * 0.1;
                block(
                    &format!("c{i}"),
                    &format!("T{i}"),
                    1.0,
                    true,
                    1.0,
                    Some((s, s + 0.05)),
                )
            })
            .collect();
        let mut comparison: Vec<CompareBlock> = vec![block(
            "extra",
            "new_task",
            1.0,
            true,
            1.0,
            Some((99.0, 99.05)),
        )];
        for i in 0..4 {
            let s = 100.0 + i as f64 * 0.1 + shift_days;
            comparison.push(block(
                &format!("m{i}"),
                &format!("T{i}"),
                1.0,
                true,
                1.0,
                Some((s, s + 0.05)),
            ));
        }

        let data = ccd(current, comparison).unwrap();
        let adv = &data.advanced_compare;
        assert_eq!(adv.global_metrics.timed_common_count, 4);
        assert_eq!(adv.global_metrics.only_in_comparison_count, 1);
        // All 4 common tasks should form one block.
        assert_eq!(adv.global_metrics.coherent_block_count, 1);
        assert_eq!(adv.blocks[0].size, 4);
    }

    #[test]
    fn reordering_splits_blocks() {
        // Tasks T0..T5 in A. In B, T2 is moved before T0 → breaks the block.
        let current: Vec<CompareBlock> = (0..6)
            .map(|i| {
                let s = 100.0 + i as f64 * 0.1;
                block(
                    &format!("c{i}"),
                    &format!("T{i}"),
                    1.0,
                    true,
                    1.0,
                    Some((s, s + 0.05)),
                )
            })
            .collect();
        // B order: T2, T0, T1, T3, T4, T5 (T2 moved to front)
        let b_order = [2, 0, 1, 3, 4, 5];
        let comparison: Vec<CompareBlock> = b_order
            .iter()
            .enumerate()
            .map(|(pos, &orig)| {
                let s = 200.0 + pos as f64 * 0.1;
                block(
                    &format!("m{orig}"),
                    &format!("T{orig}"),
                    1.0,
                    true,
                    1.0,
                    Some((s, s + 0.05)),
                )
            })
            .collect();
        let data = ccd(current, comparison).unwrap();
        let adv = &data.advanced_compare;
        // LIS should be 5 (all except T2's out-of-order position)
        assert!(adv.global_metrics.order_preservation_ratio.unwrap() < 1.0);
        // Should have more than one block (T2 can't join T0,T1 because of order inversion)
        assert!(adv.global_metrics.coherent_block_count >= 1);
    }

    #[test]
    fn min_block_size_filters_small_groups() {
        // 2 tasks with one shift, 4 tasks with another. min_block_size=3 drops the pair.
        let current: Vec<CompareBlock> = (0..6)
            .map(|i| {
                let s = 100.0 + i as f64 * 0.1;
                block(
                    &format!("c{i}"),
                    &format!("T{i}"),
                    1.0,
                    true,
                    1.0,
                    Some((s, s + 0.05)),
                )
            })
            .collect();
        // B: first 2 tasks shifted by 100 min, last 4 shifted by 0 min (big jump)
        let comparison: Vec<CompareBlock> = (0..6)
            .map(|i| {
                let shift_days = if i < 2 {
                    100.0 / MJD_DAYS_TO_MINUTES
                } else {
                    0.0
                };
                let s = 100.0 + i as f64 * 0.1 + shift_days;
                block(
                    &format!("m{i}"),
                    &format!("T{i}"),
                    1.0,
                    true,
                    1.0,
                    Some((s, s + 0.05)),
                )
            })
            .collect();
        let data = ccd_adv(current, comparison, None, Some(3), None).unwrap();
        let adv = &data.advanced_compare;
        // The group of 2 should be filtered out → ungrouped
        assert_eq!(adv.global_metrics.ungrouped_common_count, 2);
        assert_eq!(adv.global_metrics.coherent_block_count, 1);
        assert_eq!(adv.blocks[0].size, 4);
    }

    #[test]
    fn merge_combines_adjacent_close_blocks() {
        // 3 tasks with shift=0, then 3 tasks with shift=2 min.
        // With epsilon=1 they split; with merge_epsilon=3 they merge.
        let current: Vec<CompareBlock> = (0..6)
            .map(|i| {
                let s = 100.0 + i as f64 * 0.1;
                block(
                    &format!("c{i}"),
                    &format!("T{i}"),
                    1.0,
                    true,
                    1.0,
                    Some((s, s + 0.05)),
                )
            })
            .collect();
        let comparison: Vec<CompareBlock> = (0..6)
            .map(|i| {
                let shift_days = if i < 3 {
                    0.0
                } else {
                    2.0 / MJD_DAYS_TO_MINUTES
                };
                let s = 100.0 + i as f64 * 0.1 + shift_days;
                block(
                    &format!("m{i}"),
                    &format!("T{i}"),
                    1.0,
                    true,
                    1.0,
                    Some((s, s + 0.05)),
                )
            })
            .collect();

        // Without merge (eps=1, merge_eps=1): two blocks of 3
        let data1 = ccd_adv(
            current.clone(),
            comparison.clone(),
            Some(1.0),
            Some(1),
            Some(1.0),
        )
        .unwrap();
        assert_eq!(
            data1.advanced_compare.global_metrics.coherent_block_count,
            2
        );

        // With merge (eps=1, merge_eps=3): blocks should merge into one
        let data2 = ccd_adv(current, comparison, Some(1.0), Some(1), Some(3.0)).unwrap();
        assert_eq!(
            data2.advanced_compare.global_metrics.coherent_block_count,
            1
        );
        assert_eq!(data2.advanced_compare.blocks[0].size, 6);
    }

    #[test]
    fn order_preservation_ratio_via_lis() {
        // A: T0, T1, T2, T3 in order. B: T2, T0, T1, T3.
        // LIS of pos_b (ordered by pos_a) = LIS([1, 2, 0, 3]) = 3 → ratio = 3/4 = 0.75
        let current = vec![
            block("c0", "T0", 1.0, true, 1.0, Some((100.0, 100.05))),
            block("c1", "T1", 1.0, true, 1.0, Some((100.1, 100.15))),
            block("c2", "T2", 1.0, true, 1.0, Some((100.2, 100.25))),
            block("c3", "T3", 1.0, true, 1.0, Some((100.3, 100.35))),
        ];
        // B order: T2, T0, T1, T3
        let comparison = vec![
            block("m2", "T2", 1.0, true, 1.0, Some((200.0, 200.05))),
            block("m0", "T0", 1.0, true, 1.0, Some((200.1, 200.15))),
            block("m1", "T1", 1.0, true, 1.0, Some((200.2, 200.25))),
            block("m3", "T3", 1.0, true, 1.0, Some((200.3, 200.35))),
        ];
        let data = ccd(current, comparison).unwrap();
        let ratio = data
            .advanced_compare
            .global_metrics
            .order_preservation_ratio
            .unwrap();
        assert!((ratio - 0.75).abs() < 1e-9, "Expected 0.75, got {ratio}");
    }

    #[test]
    fn no_timed_common_produces_empty_blocks_and_null_metrics() {
        // All common tasks are unscheduled → timed_common_count = 0.
        let current = vec![
            block("c0", "T0", 1.0, false, 1.0, None),
            block("c1", "T1", 2.0, false, 1.0, None),
        ];
        let comparison = vec![
            block("m0", "T0", 1.0, false, 1.0, None),
            block("m1", "T1", 2.0, false, 1.0, None),
        ];
        let data = ccd(current, comparison).unwrap();
        let adv = &data.advanced_compare;
        assert_eq!(adv.global_metrics.timed_common_count, 0);
        assert!(adv.global_metrics.order_preservation_ratio.is_none());
        assert!(adv.global_metrics.global_shift_median_minutes.is_none());
        assert!(adv.global_metrics.local_shift_mad_minutes.is_none());
        assert!(adv.blocks.is_empty());
        assert_eq!(adv.global_metrics.matched_count, 2);
    }

    #[test]
    fn ignored_missing_key_counts_empty_ids() {
        let current = vec![
            block("c0", "T0", 1.0, true, 1.0, Some((100.0, 100.1))),
            block("c1", "", 1.0, true, 1.0, Some((101.0, 101.1))),
            block("c2", "", 1.0, true, 1.0, Some((102.0, 102.1))),
        ];
        let comparison = vec![
            block("m0", "T0", 1.0, true, 1.0, Some((100.0, 100.1))),
            block("m1", "", 1.0, true, 1.0, Some((103.0, 103.1))),
        ];
        let data = ccd(current, comparison).unwrap();
        let adv = &data.advanced_compare;
        assert_eq!(adv.global_metrics.ignored_missing_key_current, 2);
        assert_eq!(adv.global_metrics.ignored_missing_key_comparison, 1);
    }

    #[test]
    fn defaults_are_echoed_in_params_used() {
        let current = vec![block("c0", "T0", 1.0, true, 1.0, Some((100.0, 100.1)))];
        let comparison = vec![block("m0", "T0", 1.0, true, 1.0, Some((100.0, 100.1)))];
        let data = ccd(current, comparison).unwrap();
        let p = &data.advanced_compare.params_used;
        assert!((p.epsilon_minutes - 5.0).abs() < 1e-9);
        assert_eq!(p.min_block_size, 3);
        assert!((p.merge_epsilon_minutes - 5.0).abs() < 1e-9);
    }
}
