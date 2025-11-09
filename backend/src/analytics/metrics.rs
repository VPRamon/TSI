/// Metrics computation for scheduling data
use crate::models::schedule::SchedulingBlock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Overall scheduling metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingMetrics {
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
    pub unscheduled_blocks: usize,
    pub scheduling_rate: f64,
    
    pub total_requested_hours: f64,
    pub total_scheduled_hours: f64,
    pub total_visibility_hours: f64,
    pub utilization_rate: f64,
    
    pub priority_stats: StatsSummary,
    pub visibility_hours_stats: StatsSummary,
    pub requested_hours_stats: StatsSummary,
    pub elevation_range_stats: StatsSummary,
    
    pub priority_bin_counts: HashMap<String, usize>,
}

/// Statistical summary (mean, median, std, quartiles)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSummary {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std: f64,
    pub min: f64,
    pub max: f64,
    pub q25: f64,
    pub q75: f64,
}

impl StatsSummary {
    /// Compute statistics from a slice of values
    pub fn from_values(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let count = sorted.len();
        let sum: f64 = sorted.iter().sum();
        let mean = sum / count as f64;
        
        let median = if count % 2 == 0 {
            (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
        } else {
            sorted[count / 2]
        };
        
        let variance: f64 = sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (count as f64 - 1.0).max(1.0);
        let std = variance.sqrt();
        
        let min = sorted[0];
        let max = sorted[count - 1];
        
        let q25_idx = (count as f64 * 0.25) as usize;
        let q75_idx = (count as f64 * 0.75) as usize;
        let q25 = sorted[q25_idx.min(count - 1)];
        let q75 = sorted[q75_idx.min(count - 1)];
        
        Some(Self {
            count,
            mean,
            median,
            std,
            min,
            max,
            q25,
            q75,
        })
    }
}

/// Compute overall scheduling metrics
pub fn compute_metrics(blocks: &[SchedulingBlock]) -> SchedulingMetrics {
    let total_blocks = blocks.len();
    let scheduled_blocks = blocks.iter().filter(|b| b.scheduled_flag).count();
    let unscheduled_blocks = total_blocks - scheduled_blocks;
    let scheduling_rate = if total_blocks > 0 {
        scheduled_blocks as f64 / total_blocks as f64
    } else {
        0.0
    };
    
    let total_requested_hours: f64 = blocks.iter().map(|b| b.requested_hours).sum();
    let total_visibility_hours: f64 = blocks.iter().map(|b| b.total_visibility_hours).sum();
    
    let total_scheduled_hours: f64 = blocks
        .iter()
        .filter(|b| b.scheduled_flag)
        .filter_map(|b| {
            if let (Some(start), Some(stop)) = (b.scheduled_period_start, b.scheduled_period_stop) {
                Some((stop - start) * 24.0)
            } else {
                None
            }
        })
        .sum();
    
    let utilization_rate = if total_visibility_hours > 0.0 {
        total_scheduled_hours / total_visibility_hours
    } else {
        0.0
    };
    
    // Compute stats for various columns
    let priorities: Vec<f64> = blocks.iter().map(|b| b.priority).collect();
    let visibility_hours: Vec<f64> = blocks.iter().map(|b| b.total_visibility_hours).collect();
    let requested_hours: Vec<f64> = blocks.iter().map(|b| b.requested_hours).collect();
    let elevation_ranges: Vec<f64> = blocks.iter().map(|b| b.elevation_range_deg).collect();
    
    let priority_stats = StatsSummary::from_values(&priorities).unwrap_or_else(|| StatsSummary {
        count: 0,
        mean: 0.0,
        median: 0.0,
        std: 0.0,
        min: 0.0,
        max: 0.0,
        q25: 0.0,
        q75: 0.0,
    });
    
    let visibility_hours_stats = StatsSummary::from_values(&visibility_hours).unwrap_or_else(|| StatsSummary {
        count: 0,
        mean: 0.0,
        median: 0.0,
        std: 0.0,
        min: 0.0,
        max: 0.0,
        q25: 0.0,
        q75: 0.0,
    });
    
    let requested_hours_stats = StatsSummary::from_values(&requested_hours).unwrap_or_else(|| StatsSummary {
        count: 0,
        mean: 0.0,
        median: 0.0,
        std: 0.0,
        min: 0.0,
        max: 0.0,
        q25: 0.0,
        q75: 0.0,
    });
    
    let elevation_range_stats = StatsSummary::from_values(&elevation_ranges).unwrap_or_else(|| StatsSummary {
        count: 0,
        mean: 0.0,
        median: 0.0,
        std: 0.0,
        min: 0.0,
        max: 0.0,
        q25: 0.0,
        q75: 0.0,
    });
    
    // Count priority bins
    let mut priority_bin_counts = HashMap::new();
    for block in blocks {
        let bin_name = format!("{}", block.priority_bin);
        *priority_bin_counts.entry(bin_name).or_insert(0) += 1;
    }
    
    SchedulingMetrics {
        total_blocks,
        scheduled_blocks,
        unscheduled_blocks,
        scheduling_rate,
        total_requested_hours,
        total_scheduled_hours,
        total_visibility_hours,
        utilization_rate,
        priority_stats,
        visibility_hours_stats,
        requested_hours_stats,
        elevation_range_stats,
        priority_bin_counts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::schedule::{PriorityBin, VisibilityPeriod};

    #[test]
    fn test_stats_summary() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = StatsSummary::from_values(&values).unwrap();
        
        assert_eq!(stats.count, 5);
        assert!((stats.mean - 3.0).abs() < 1e-9);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
    }

    #[test]
    fn test_compute_metrics() {
        let blocks = vec![
            SchedulingBlock {
                scheduling_block_id: "1".to_string(),
                priority: 10.0,
                min_observation_time_in_sec: 1200.0,
                requested_duration_sec: 3600.0,
                fixed_start_time: None,
                fixed_stop_time: None,
                dec_in_deg: 0.0,
                ra_in_deg: 0.0,
                min_azimuth_angle_in_deg: 0.0,
                max_azimuth_angle_in_deg: 360.0,
                min_elevation_angle_in_deg: 60.0,
                max_elevation_angle_in_deg: 90.0,
                scheduled_period_start: Some(61892.0),
                scheduled_period_stop: Some(61892.1),
                visibility: vec![VisibilityPeriod {
                    start: 61892.0,
                    stop: 61892.5,
                }],
                num_visibility_periods: 1,
                total_visibility_hours: 12.0,
                priority_bin: PriorityBin::High,
                scheduled_flag: true,
                requested_hours: 1.0,
                elevation_range_deg: 30.0,
            },
            SchedulingBlock {
                scheduling_block_id: "2".to_string(),
                priority: 5.0,
                min_observation_time_in_sec: 1800.0,
                requested_duration_sec: 1800.0,
                fixed_start_time: None,
                fixed_stop_time: None,
                dec_in_deg: 0.0,
                ra_in_deg: 0.0,
                min_azimuth_angle_in_deg: 0.0,
                max_azimuth_angle_in_deg: 360.0,
                min_elevation_angle_in_deg: 60.0,
                max_elevation_angle_in_deg: 90.0,
                scheduled_period_start: None,
                scheduled_period_stop: None,
                visibility: vec![VisibilityPeriod {
                    start: 61893.0,
                    stop: 61893.25,
                }],
                num_visibility_periods: 1,
                total_visibility_hours: 6.0,
                priority_bin: PriorityBin::Medium,
                scheduled_flag: false,
                requested_hours: 0.5,
                elevation_range_deg: 30.0,
            },
        ];
        
        let metrics = compute_metrics(&blocks);
        
        assert_eq!(metrics.total_blocks, 2);
        assert_eq!(metrics.scheduled_blocks, 1);
        assert_eq!(metrics.unscheduled_blocks, 1);
        assert!((metrics.scheduling_rate - 0.5).abs() < 1e-9);
        assert!((metrics.total_requested_hours - 1.5).abs() < 1e-9);
        assert!((metrics.total_visibility_hours - 18.0).abs() < 1e-9);
    }
}
