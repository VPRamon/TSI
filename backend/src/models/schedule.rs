/// Data models for telescope scheduling data
use serde::{Deserialize, Serialize};
use std::fmt;

/// A visibility period with start and stop times in MJD
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibilityPeriod {
    pub start: f64,
    pub stop: f64,
}

impl VisibilityPeriod {
    pub fn duration_hours(&self) -> f64 {
        (self.stop - self.start) * 24.0
    }
}

/// Priority bin categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PriorityBin {
    #[serde(rename = "Low (0-5)")]
    Low,
    #[serde(rename = "Medium (5-8)")]
    Medium,
    #[serde(rename = "Medium (8-10)")]
    MediumHigh,
    #[serde(rename = "High (10+)")]
    High,
    #[serde(rename = "No priority")]
    NoPriority,
}

impl fmt::Display for PriorityBin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriorityBin::Low => write!(f, "Low (0-5)"),
            PriorityBin::Medium => write!(f, "Medium (5-8)"),
            PriorityBin::MediumHigh => write!(f, "Medium (8-10)"),
            PriorityBin::High => write!(f, "High (10+)"),
            PriorityBin::NoPriority => write!(f, "No priority"),
        }
    }
}

impl PriorityBin {
    pub fn from_priority(priority: f64) -> Self {
        if priority < 5.0 {
            PriorityBin::Low
        } else if priority < 8.0 {
            PriorityBin::Medium
        } else if priority < 10.0 {
            PriorityBin::MediumHigh
        } else {
            PriorityBin::High
        }
    }
}

/// Scheduled period with start and stop times in MJD
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduledPeriod {
    pub start: f64,
    pub stop: f64,
}

impl ScheduledPeriod {
    pub fn duration_hours(&self) -> f64 {
        (self.stop - self.start) * 24.0
    }
}

/// A complete scheduling block with all required fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulingBlock {
    // Base fields from JSON
    pub scheduling_block_id: String,
    pub priority: f64,
    pub min_observation_time_in_sec: f64,
    pub requested_duration_sec: f64,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_start_time: Option<f64>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_stop_time: Option<f64>,
    
    pub dec_in_deg: f64,
    pub ra_in_deg: f64,
    pub min_azimuth_angle_in_deg: f64,
    pub max_azimuth_angle_in_deg: f64,
    pub min_elevation_angle_in_deg: f64,
    pub max_elevation_angle_in_deg: f64,
    
    // Scheduled period (may be empty for unscheduled)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_period_start: Option<f64>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_period_stop: Option<f64>,
    
    // Visibility data
    pub visibility: Vec<VisibilityPeriod>,
    pub num_visibility_periods: usize,
    pub total_visibility_hours: f64,
    
    // Derived fields
    pub priority_bin: PriorityBin,
    pub scheduled_flag: bool,
    pub requested_hours: f64,
    pub elevation_range_deg: f64,
}

impl SchedulingBlock {
    /// Create a new scheduling block with derived fields computed
    pub fn new(
        scheduling_block_id: String,
        priority: f64,
        min_observation_time_in_sec: f64,
        requested_duration_sec: f64,
        fixed_start_time: Option<f64>,
        fixed_stop_time: Option<f64>,
        dec_in_deg: f64,
        ra_in_deg: f64,
        min_azimuth_angle_in_deg: f64,
        max_azimuth_angle_in_deg: f64,
        min_elevation_angle_in_deg: f64,
        max_elevation_angle_in_deg: f64,
        scheduled_period_start: Option<f64>,
        scheduled_period_stop: Option<f64>,
        visibility: Vec<VisibilityPeriod>,
    ) -> Self {
        // Compute derived fields
        let num_visibility_periods = visibility.len();
        let total_visibility_hours: f64 = visibility.iter().map(|v| v.duration_hours()).sum();
        let priority_bin = PriorityBin::from_priority(priority);
        let scheduled_flag = scheduled_period_start.is_some() && scheduled_period_stop.is_some();
        let requested_hours = requested_duration_sec / 3600.0;
        let elevation_range_deg = max_elevation_angle_in_deg - min_elevation_angle_in_deg;

        Self {
            scheduling_block_id,
            priority,
            min_observation_time_in_sec,
            requested_duration_sec,
            fixed_start_time,
            fixed_stop_time,
            dec_in_deg,
            ra_in_deg,
            min_azimuth_angle_in_deg,
            max_azimuth_angle_in_deg,
            min_elevation_angle_in_deg,
            max_elevation_angle_in_deg,
            scheduled_period_start,
            scheduled_period_stop,
            visibility,
            num_visibility_periods,
            total_visibility_hours,
            priority_bin,
            scheduled_flag,
            requested_hours,
            elevation_range_deg,
        }
    }

    /// Check if this observation is impossible (visibility < min observation time)
    pub fn is_impossible(&self, tolerance_sec: f64) -> bool {
        let visibility_sec = self.total_visibility_hours * 3600.0;
        self.min_observation_time_in_sec - tolerance_sec > visibility_sec
            || self.requested_duration_sec - tolerance_sec > visibility_sec
    }
}

/// Dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub filename: String,
    pub num_blocks: usize,
    pub num_scheduled: usize,
    pub num_unscheduled: usize,
    pub loaded_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_period_duration() {
        let period = VisibilityPeriod {
            start: 61892.0,
            stop: 61892.5,
        };
        assert!((period.duration_hours() - 12.0).abs() < 1e-9);
    }

    #[test]
    fn test_priority_bin_classification() {
        assert_eq!(PriorityBin::from_priority(3.0), PriorityBin::Low);
        assert_eq!(PriorityBin::from_priority(6.5), PriorityBin::Medium);
        assert_eq!(PriorityBin::from_priority(8.5), PriorityBin::MediumHigh);
        assert_eq!(PriorityBin::from_priority(12.0), PriorityBin::High);
    }

    #[test]
    fn test_is_impossible() {
        let block = SchedulingBlock::new(
            "test".to_string(),
            8.5,
            1200.0,
            1200.0,
            None,
            None,
            0.0,
            0.0,
            0.0,
            360.0,
            60.0,
            90.0,
            None,
            None,
            vec![VisibilityPeriod {
                start: 61892.0,
                stop: 61892.0001, // Very short visibility
            }],
        );
        
        // Should be impossible: 1200s requested but only ~8.64s available
        assert!(block.is_impossible(1.0));
    }
}
