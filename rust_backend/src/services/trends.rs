use crate::db::models::{
    EmpiricalRatePoint, HeatmapBin, SmoothedPoint, TrendsBlock, TrendsData, TrendsMetrics,
};
use crate::db::{analytics, operations};
use pyo3::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;

/// Compute overview metrics from trends blocks.
fn compute_metrics(blocks: &[TrendsBlock]) -> TrendsMetrics {
    let total_count = blocks.len();
    let scheduled_count = blocks.iter().filter(|b| b.scheduled).count();
    let zero_visibility_count = blocks.iter().filter(|b| b.total_visibility_hours == 0.0).count();
    let scheduling_rate = if total_count > 0 {
        scheduled_count as f64 / total_count as f64
    } else {
        0.0
    };

    // Collect all values for stats
    let priorities: Vec<f64> = blocks.iter().map(|b| b.priority).collect();
    let visibilities: Vec<f64> = blocks.iter().map(|b| b.total_visibility_hours).collect();
    let times: Vec<f64> = blocks.iter().map(|b| b.requested_hours).collect();

    let priority_min = priorities
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let priority_max = priorities
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let priority_mean = if !priorities.is_empty() {
        priorities.iter().sum::<f64>() / priorities.len() as f64
    } else {
        0.0
    };

    let visibility_min = visibilities
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let visibility_max = visibilities
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let visibility_mean = if !visibilities.is_empty() {
        visibilities.iter().sum::<f64>() / visibilities.len() as f64
    } else {
        0.0
    };

    let time_min = times.iter().copied().fold(f64::INFINITY, f64::min);
    let time_max = times.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let time_mean = if !times.is_empty() {
        times.iter().sum::<f64>() / times.len() as f64
    } else {
        0.0
    };

    TrendsMetrics {
        total_count,
        scheduled_count,
        scheduling_rate,
        zero_visibility_count,
        priority_min,
        priority_max,
        priority_mean,
        visibility_min,
        visibility_max,
        visibility_mean,
        time_min,
        time_max,
        time_mean,
    }
}

/// Compute empirical scheduling rates by priority.
fn compute_by_priority(blocks: &[TrendsBlock]) -> Vec<EmpiricalRatePoint> {
    // Group by priority value
    let mut priority_groups: HashMap<i32, (usize, usize)> = HashMap::new();

    for block in blocks {
        let priority_int = block.priority.round() as i32;
        let entry = priority_groups.entry(priority_int).or_insert((0, 0));
        entry.0 += 1; // total count
        if block.scheduled {
            entry.1 += 1; // scheduled count
        }
    }

    let mut rates: Vec<EmpiricalRatePoint> = priority_groups
        .into_iter()
        .map(|(priority, (total, scheduled))| {
            let rate = if total > 0 {
                scheduled as f64 / total as f64
            } else {
                0.0
            };
            EmpiricalRatePoint {
                bin_label: format!("Priority {}", priority),
                mid_value: priority as f64,
                scheduled_rate: rate,
                count: total,
            }
        })
        .collect();

    rates.sort_by(|a, b| {
        a.mid_value
            .partial_cmp(&b.mid_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    rates
}

/// Compute empirical scheduling rates by binning a continuous variable.
fn compute_by_bins(
    blocks: &[TrendsBlock],
    get_value: impl Fn(&TrendsBlock) -> f64,
    n_bins: usize,
    label_prefix: &str,
) -> Vec<EmpiricalRatePoint> {
    if blocks.is_empty() {
        return vec![];
    }

    // Find min and max
    let values: Vec<f64> = blocks.iter().map(|b| get_value(b)).collect();
    let min_val = values
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let max_val = values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    if min_val == max_val {
        // All values are the same
        let scheduled = blocks.iter().filter(|b| b.scheduled).count();
        return vec![EmpiricalRatePoint {
            bin_label: format!("{} [{:.1}]", label_prefix, min_val),
            mid_value: min_val,
            scheduled_rate: scheduled as f64 / blocks.len() as f64,
            count: blocks.len(),
        }];
    }

    let bin_width = (max_val - min_val) / n_bins as f64;

    // Create bins
    let mut bins: Vec<(usize, usize, f64)> = vec![(0, 0, 0.0); n_bins];

    for block in blocks {
        let value = get_value(block);
        let mut bin_idx = ((value - min_val) / bin_width).floor() as usize;
        if bin_idx >= n_bins {
            bin_idx = n_bins - 1;
        }

        bins[bin_idx].0 += 1; // total count
        if block.scheduled {
            bins[bin_idx].1 += 1; // scheduled count
        }
        bins[bin_idx].2 += value; // sum for mean
    }

    bins.into_iter()
        .enumerate()
        .filter(|(_, (total, _, _))| *total > 0)
        .map(|(idx, (total, scheduled, sum))| {
            let rate = if total > 0 {
                scheduled as f64 / total as f64
            } else {
                0.0
            };
            let mid_value = sum / total as f64;
            let bin_start = min_val + (idx as f64 * bin_width);
            let bin_end = min_val + ((idx + 1) as f64 * bin_width);
            EmpiricalRatePoint {
                bin_label: format!("{} [{:.1}-{:.1}]", label_prefix, bin_start, bin_end),
                mid_value,
                scheduled_rate: rate,
                count: total,
            }
        })
        .collect()
}

/// Compute smoothed trend using Gaussian kernel weighted average.
fn compute_smoothed_trend(
    blocks: &[TrendsBlock],
    get_x: impl Fn(&TrendsBlock) -> f64,
    bandwidth: f64,
    n_points: usize,
) -> Vec<SmoothedPoint> {
    if blocks.is_empty() {
        return vec![];
    }

    let x_values: Vec<f64> = blocks.iter().map(|b| get_x(b)).collect();
    let y_values: Vec<f64> = blocks.iter().map(|b| if b.scheduled { 1.0 } else { 0.0 }).collect();

    let x_min = x_values
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let x_max = x_values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    if x_min == x_max {
        // All values are the same
        let scheduled = blocks.iter().filter(|b| b.scheduled).count();
        return vec![SmoothedPoint {
            x: x_min,
            y_smoothed: scheduled as f64 / blocks.len() as f64,
            n_samples: blocks.len(),
        }];
    }

    let x_range = x_max - x_min;
    let bw = bandwidth * x_range;

    let mut smoothed = Vec::with_capacity(n_points);

    for i in 0..n_points {
        let x_point = x_min + (i as f64 / (n_points - 1) as f64) * x_range;

        // Compute Gaussian weights
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        let mut n_significant = 0;

        for (j, &x_val) in x_values.iter().enumerate() {
            let distance = (x_val - x_point).abs();
            let weight = (-0.5 * (distance / bw).powi(2)).exp();

            weighted_sum += weight * y_values[j];
            weight_sum += weight;

            if weight > 0.01 {
                n_significant += 1;
            }
        }

        let y_smoothed = if weight_sum > 0.0 {
            weighted_sum / weight_sum
        } else {
            0.0
        };

        smoothed.push(SmoothedPoint {
            x: x_point,
            y_smoothed,
            n_samples: n_significant,
        });
    }

    smoothed
}

/// Compute 2D heatmap bins for visibility vs requested time.
fn compute_heatmap_bins(blocks: &[TrendsBlock], n_bins: usize) -> Vec<HeatmapBin> {
    if blocks.is_empty() {
        return vec![];
    }

    // Find ranges
    let vis_values: Vec<f64> = blocks.iter().map(|b| b.total_visibility_hours).collect();
    let time_values: Vec<f64> = blocks.iter().map(|b| b.requested_hours).collect();

    let vis_min = vis_values
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let vis_max = vis_values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let time_min = time_values
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let time_max = time_values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);

    if vis_min == vis_max || time_min == time_max {
        return vec![];
    }

    let vis_width = (vis_max - vis_min) / n_bins as f64;
    let time_width = (time_max - time_min) / n_bins as f64;

    // Create 2D bins
    let mut bins: HashMap<(usize, usize), (usize, usize, f64, f64)> = HashMap::new();

    for block in blocks {
        let vis_idx = ((block.total_visibility_hours - vis_min) / vis_width).floor() as usize;
        let time_idx = ((block.requested_hours - time_min) / time_width).floor() as usize;

        let vis_idx = vis_idx.min(n_bins - 1);
        let time_idx = time_idx.min(n_bins - 1);

        let entry = bins.entry((vis_idx, time_idx)).or_insert((0, 0, 0.0, 0.0));
        entry.0 += 1; // total count
        if block.scheduled {
            entry.1 += 1; // scheduled count
        }
        entry.2 += block.total_visibility_hours; // sum for mean
        entry.3 += block.requested_hours; // sum for mean
    }

    bins.into_iter()
        .filter(|(_, (total, _, _, _))| *total > 0)
        .map(|(_, (total, scheduled, vis_sum, time_sum))| {
            let rate = if total > 0 {
                scheduled as f64 / total as f64
            } else {
                0.0
            };
            HeatmapBin {
                visibility_mid: vis_sum / total as f64,
                time_mid: time_sum / total as f64,
                scheduled_rate: rate,
                count: total,
            }
        })
        .collect()
}

/// Compute trends data with empirical rates, smoothed curves, and heatmap bins.
pub fn compute_trends_data(
    blocks: Vec<TrendsBlock>,
    n_bins: usize,
    bandwidth: f64,
    n_smooth_points: usize,
) -> Result<TrendsData, String> {
    if blocks.is_empty() {
        return Err("No blocks provided for trends analysis".to_string());
    }

    // Compute metrics
    let metrics = compute_metrics(&blocks);

    // Compute empirical rates
    let by_priority = compute_by_priority(&blocks);
    let by_visibility = compute_by_bins(
        &blocks,
        |b| b.total_visibility_hours,
        n_bins,
        "Visibility",
    );
    let by_time = compute_by_bins(&blocks, |b| b.requested_hours, n_bins, "Time");

    // Compute smoothed trends
    let smoothed_visibility = compute_smoothed_trend(
        &blocks,
        |b| b.total_visibility_hours,
        bandwidth,
        n_smooth_points,
    );
    let smoothed_time = compute_smoothed_trend(
        &blocks,
        |b| b.requested_hours,
        bandwidth,
        n_smooth_points,
    );

    // Compute heatmap bins
    let heatmap_bins = compute_heatmap_bins(&blocks, n_bins);

    // Get unique priority values for filtering
    let mut priority_values: Vec<f64> = blocks
        .iter()
        .map(|b| b.priority)
        .collect();
    priority_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    priority_values.dedup();

    Ok(TrendsData {
        blocks,
        metrics,
        by_priority,
        by_visibility,
        by_time,
        smoothed_visibility,
        smoothed_time,
        heatmap_bins,
        priority_values,
    })
}

/// Get complete trends data with computed analytics.
/// Uses pre-computed analytics table when available for ~10-100x faster performance.
pub async fn get_trends_data(
    schedule_id: i64,
    filter_impossible: bool,
    n_bins: usize,
    bandwidth: f64,
    n_smooth_points: usize,
) -> Result<TrendsData, String> {
    // Try analytics table first (much faster - no JOINs, pre-computed metrics)
    let mut blocks = match analytics::fetch_analytics_blocks_for_trends(schedule_id).await {
        Ok(b) if !b.is_empty() => b,
        Ok(_) | Err(_) => {
            // Fall back to operations table if analytics not populated
            operations::fetch_trends_blocks(schedule_id).await?
        }
    };

    // Apply impossible filter if requested
    if filter_impossible {
        blocks.retain(|b| b.total_visibility_hours > 0.0);
    }

    compute_trends_data(blocks, n_bins, bandwidth, n_smooth_points)
}

/// Get complete trends data with computed analytics and metadata.
#[pyfunction]
pub fn py_get_trends_data(
    schedule_id: i64,
    filter_impossible: bool,
    n_bins: usize,
    bandwidth: f64,
    n_smooth_points: usize,
) -> PyResult<TrendsData> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(get_trends_data(
            schedule_id,
            filter_impossible,
            n_bins,
            bandwidth,
            n_smooth_points,
        ))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_metrics() {
        let blocks = vec![
            TrendsBlock {
                scheduling_block_id: 1,
                priority: 5.0,
                total_visibility_hours: 10.0,
                requested_hours: 2.0,
                scheduled: true,
            },
            TrendsBlock {
                scheduling_block_id: 2,
                priority: 3.0,
                total_visibility_hours: 5.0,
                requested_hours: 1.0,
                scheduled: false,
            },
        ];

        let metrics = compute_metrics(&blocks);
        assert_eq!(metrics.total_count, 2);
        assert_eq!(metrics.scheduled_count, 1);
        assert_eq!(metrics.scheduling_rate, 0.5);
    }

    #[test]
    fn test_compute_by_priority() {
        let blocks = vec![
            TrendsBlock {
                scheduling_block_id: 1,
                priority: 5.0,
                total_visibility_hours: 10.0,
                requested_hours: 2.0,
                scheduled: true,
            },
            TrendsBlock {
                scheduling_block_id: 2,
                priority: 5.0,
                total_visibility_hours: 8.0,
                requested_hours: 1.5,
                scheduled: true,
            },
            TrendsBlock {
                scheduling_block_id: 3,
                priority: 3.0,
                total_visibility_hours: 5.0,
                requested_hours: 1.0,
                scheduled: false,
            },
        ];

        let rates = compute_by_priority(&blocks);
        assert_eq!(rates.len(), 2);
        assert_eq!(rates[0].mid_value, 3.0);
        assert_eq!(rates[0].scheduled_rate, 0.0);
        assert_eq!(rates[1].mid_value, 5.0);
        assert_eq!(rates[1].scheduled_rate, 1.0);
    }
}
