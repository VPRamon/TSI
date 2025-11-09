/// Correlation analysis for scheduling data
use crate::models::schedule::SchedulingBlock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Correlation matrix result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationMatrix {
    pub columns: Vec<String>,
    pub matrix: Vec<Vec<f64>>,
    pub correlations: Vec<CorrelationPair>,
}

/// A single correlation pair with insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationPair {
    pub col1: String,
    pub col2: String,
    pub correlation: f64,
    pub insight: String,
}

/// Compute Spearman rank correlation between two vectors
fn spearman_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }
    
    let n = x.len();
    
    // Compute ranks for x
    let mut x_indexed: Vec<(usize, f64)> = x.iter().copied().enumerate().collect();
    x_indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut x_ranks = vec![0.0; n];
    for (rank, (idx, _)) in x_indexed.iter().enumerate() {
        x_ranks[*idx] = (rank + 1) as f64;
    }
    
    // Compute ranks for y
    let mut y_indexed: Vec<(usize, f64)> = y.iter().copied().enumerate().collect();
    y_indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut y_ranks = vec![0.0; n];
    for (rank, (idx, _)) in y_indexed.iter().enumerate() {
        y_ranks[*idx] = (rank + 1) as f64;
    }
    
    // Compute Pearson correlation on ranks
    let mean_x: f64 = x_ranks.iter().sum::<f64>() / n as f64;
    let mean_y: f64 = y_ranks.iter().sum::<f64>() / n as f64;
    
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

/// Generate insight text for a correlation
fn generate_insight(col1: &str, col2: &str, corr: f64) -> String {
    let strength = if corr.abs() > 0.7 {
        "strong"
    } else if corr.abs() > 0.4 {
        "moderate"
    } else {
        "weak"
    };
    
    let direction = if corr > 0.0 { "positive" } else { "negative" };
    
    format!(
        "{} {} correlation ({:.3}) between {} and {}",
        strength.to_uppercase(),
        direction,
        corr,
        col1,
        col2
    )
}

/// Compute correlation matrix for selected columns
pub fn compute_correlations(
    blocks: &[SchedulingBlock],
    columns: &[String],
) -> CorrelationMatrix {
    let mut column_data: HashMap<String, Vec<f64>> = HashMap::new();
    
    // Extract data for each requested column
    for col_name in columns {
        let data = match col_name.as_str() {
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
            "ra_in_deg" => blocks.iter().map(|b| b.ra_in_deg).collect(),
            "dec_in_deg" => blocks.iter().map(|b| b.dec_in_deg).collect(),
            _ => continue,
        };
        column_data.insert(col_name.clone(), data);
    }
    
    let valid_columns: Vec<String> = column_data.keys().cloned().collect();
    let n_cols = valid_columns.len();
    
    // Compute correlation matrix
    let mut matrix = vec![vec![0.0; n_cols]; n_cols];
    let mut correlations = Vec::new();
    
    for i in 0..n_cols {
        for j in 0..n_cols {
            if i == j {
                matrix[i][j] = 1.0;
            } else if i < j {
                let col1 = &valid_columns[i];
                let col2 = &valid_columns[j];
                
                if let (Some(data1), Some(data2)) = (column_data.get(col1), column_data.get(col2)) {
                    let corr = spearman_correlation(data1, data2);
                    matrix[i][j] = corr;
                    matrix[j][i] = corr;
                    
                    // Add to correlations list if significant
                    if corr.abs() > 0.3 {
                        correlations.push(CorrelationPair {
                            col1: col1.clone(),
                            col2: col2.clone(),
                            correlation: corr,
                            insight: generate_insight(col1, col2, corr),
                        });
                    }
                }
            }
        }
    }
    
    // Sort correlations by absolute value (strongest first)
    correlations.sort_by(|a, b| {
        b.correlation
            .abs()
            .partial_cmp(&a.correlation.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    
    CorrelationMatrix {
        columns: valid_columns,
        matrix,
        correlations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::schedule::{PriorityBin, VisibilityPeriod};

    #[test]
    fn test_spearman_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        
        let corr = spearman_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 1e-9, "Perfect positive correlation");
    }

    #[test]
    fn test_compute_correlations() {
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
        
        let columns = vec![
            "priority".to_string(),
            "total_visibility_hours".to_string(),
            "requested_hours".to_string(),
        ];
        
        let result = compute_correlations(&blocks, &columns);
        
        assert_eq!(result.columns.len(), 3);
        assert_eq!(result.matrix.len(), 3);
        assert_eq!(result.matrix[0].len(), 3);
    }
}
