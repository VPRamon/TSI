/// Distribution statistics and histogram binning
use crate::models::schedule::SchedulingBlock;
use serde::{Deserialize, Serialize};

/// Histogram bin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBin {
    pub bin_start: f64,
    pub bin_end: f64,
    pub count: usize,
    pub frequency: f64,
}

/// Histogram result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    pub column: String,
    pub bins: Vec<HistogramBin>,
    pub total_count: usize,
    pub min: f64,
    pub max: f64,
}

/// Distribution statistics with percentiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    pub column: String,
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std: f64,
    pub min: f64,
    pub max: f64,
    pub q25: f64,
    pub q50: f64,
    pub q75: f64,
    pub p10: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

impl DistributionStats {
    /// Compute distribution statistics from values
    pub fn from_values(column: String, values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let count = sorted.len();
        let sum: f64 = sorted.iter().sum();
        let mean = sum / count as f64;
        
        let median = Self::percentile(&sorted, 50.0);
        
        let variance: f64 = sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (count as f64 - 1.0).max(1.0);
        let std = variance.sqrt();
        
        let min = sorted[0];
        let max = sorted[count - 1];
        
        Some(Self {
            column,
            count,
            mean,
            median,
            std,
            min,
            max,
            q25: Self::percentile(&sorted, 25.0),
            q50: Self::percentile(&sorted, 50.0),
            q75: Self::percentile(&sorted, 75.0),
            p10: Self::percentile(&sorted, 10.0),
            p90: Self::percentile(&sorted, 90.0),
            p95: Self::percentile(&sorted, 95.0),
            p99: Self::percentile(&sorted, 99.0),
        })
    }
    
    /// Compute percentile from sorted values
    fn percentile(sorted: &[f64], p: f64) -> f64 {
        let n = sorted.len();
        if n == 0 {
            return 0.0;
        }
        
        let idx = ((n - 1) as f64 * p / 100.0) as usize;
        sorted[idx.min(n - 1)]
    }
}

/// Compute histogram for a column
pub fn compute_histogram(
    blocks: &[SchedulingBlock],
    column: &str,
    num_bins: usize,
) -> Option<Histogram> {
    let values: Vec<f64> = match column {
        "priority" => blocks.iter().map(|b| b.priority).collect(),
        "total_visibility_hours" | "visibility_hours" => {
            blocks.iter().map(|b| b.total_visibility_hours).collect()
        }
        "requested_hours" => blocks.iter().map(|b| b.requested_hours).collect(),
        "elevation_range_deg" | "elevation_range" => {
            blocks.iter().map(|b| b.elevation_range_deg).collect()
        }
        "min_elevation_angle_in_deg" => {
            blocks.iter().map(|b| b.min_elevation_angle_in_deg).collect()
        }
        "max_elevation_angle_in_deg" => {
            blocks.iter().map(|b| b.max_elevation_angle_in_deg).collect()
        }
        _ => return None,
    };
    
    if values.is_empty() {
        return None;
    }
    
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    
    if min == max {
        // All values are the same
        return Some(Histogram {
            column: column.to_string(),
            bins: vec![HistogramBin {
                bin_start: min,
                bin_end: max,
                count: values.len(),
                frequency: 1.0,
            }],
            total_count: values.len(),
            min,
            max,
        });
    }
    
    let bin_width = (max - min) / num_bins as f64;
    let mut bins: Vec<HistogramBin> = Vec::new();
    
    for i in 0..num_bins {
        let bin_start = min + i as f64 * bin_width;
        let bin_end = if i == num_bins - 1 {
            max
        } else {
            min + (i + 1) as f64 * bin_width
        };
        
        let count = values
            .iter()
            .filter(|&&v| {
                if i == num_bins - 1 {
                    v >= bin_start && v <= bin_end
                } else {
                    v >= bin_start && v < bin_end
                }
            })
            .count();
        
        let frequency = count as f64 / values.len() as f64;
        
        bins.push(HistogramBin {
            bin_start,
            bin_end,
            count,
            frequency,
        });
    }
    
    Some(Histogram {
        column: column.to_string(),
        bins,
        total_count: values.len(),
        min,
        max,
    })
}

/// Compute distribution statistics for a column
pub fn compute_distribution_stats(
    blocks: &[SchedulingBlock],
    column: &str,
) -> Option<DistributionStats> {
    let values: Vec<f64> = match column {
        "priority" => blocks.iter().map(|b| b.priority).collect(),
        "total_visibility_hours" | "visibility_hours" => {
            blocks.iter().map(|b| b.total_visibility_hours).collect()
        }
        "requested_hours" => blocks.iter().map(|b| b.requested_hours).collect(),
        "elevation_range_deg" | "elevation_range" => {
            blocks.iter().map(|b| b.elevation_range_deg).collect()
        }
        "min_elevation_angle_in_deg" => {
            blocks.iter().map(|b| b.min_elevation_angle_in_deg).collect()
        }
        "max_elevation_angle_in_deg" => {
            blocks.iter().map(|b| b.max_elevation_angle_in_deg).collect()
        }
        _ => return None,
    };
    
    DistributionStats::from_values(column.to_string(), &values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::schedule::{PriorityBin, VisibilityPeriod};

    #[test]
    fn test_distribution_stats() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let stats = DistributionStats::from_values("test".to_string(), &values).unwrap();
        
        assert_eq!(stats.count, 10);
        assert!((stats.mean - 5.5).abs() < 1e-9);
        // Median for [1..10] at p50: idx = 9*0.5 = 4.5 -> 4, sorted[4] = 5.0
        assert_eq!(stats.median, 5.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 10.0);
        // Q25 at p25: idx = 9*0.25 = 2.25 -> 2, sorted[2] = 3.0
        assert_eq!(stats.q25, 3.0);
        // Q75 at p75: idx = 9*0.75 = 6.75 -> 6, sorted[6] = 7.0
        assert_eq!(stats.q75, 7.0);
    }

    #[test]
    fn test_compute_histogram() {
        let blocks = vec![
            SchedulingBlock {
                scheduling_block_id: "1".to_string(),
                priority: 2.0,
                min_observation_time_in_sec: 1200.0,
                requested_duration_sec: 3600.0,
                fixed_start_time: None,
                fixed_stop_time: None,
                target_name: None,
                target_id: None,
                dec_in_deg: 0.0,
                ra_in_deg: 0.0,
                min_azimuth_angle_in_deg: 0.0,
                max_azimuth_angle_in_deg: 360.0,
                min_elevation_angle_in_deg: 60.0,
                max_elevation_angle_in_deg: 90.0,
                scheduled_period_start: None,
                scheduled_period_stop: None,
                visibility: vec![VisibilityPeriod {
                    start: 61892.0,
                    stop: 61892.5,
                }],
                num_visibility_periods: 1,
                total_visibility_hours: 12.0,
                priority_bin: PriorityBin::Low,
                scheduled_flag: false,
                requested_hours: 1.0,
                elevation_range_deg: 30.0,
            },
            SchedulingBlock {
                scheduling_block_id: "2".to_string(),
                priority: 8.0,
                min_observation_time_in_sec: 1800.0,
                requested_duration_sec: 1800.0,
                fixed_start_time: None,
                fixed_stop_time: None,
                target_name: None,
                target_id: None,
                dec_in_deg: 0.0,
                ra_in_deg: 0.0,
                min_azimuth_angle_in_deg: 0.0,
                max_azimuth_angle_in_deg: 360.0,
                min_elevation_angle_in_deg: 60.0,
                max_elevation_angle_in_deg: 90.0,
                scheduled_period_start: Some(61892.0),
                scheduled_period_stop: Some(61892.1),
                visibility: vec![VisibilityPeriod {
                    start: 61893.0,
                    stop: 61893.25,
                }],
                num_visibility_periods: 1,
                total_visibility_hours: 6.0,
                priority_bin: PriorityBin::MediumHigh,
                scheduled_flag: true,
                requested_hours: 0.5,
                elevation_range_deg: 30.0,
            },
        ];
        
        let histogram = compute_histogram(&blocks, "priority", 5).unwrap();
        
        assert_eq!(histogram.column, "priority");
        assert_eq!(histogram.bins.len(), 5);
        assert_eq!(histogram.total_count, 2);
        assert_eq!(histogram.min, 2.0);
        assert_eq!(histogram.max, 8.0);
    }
}
