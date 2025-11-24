use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a single visibility period (start, stop) in UTC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisibilityPeriod {
    pub start: DateTime<Utc>,
    pub stop: DateTime<Utc>,
}

impl VisibilityPeriod {
    pub fn new(start: DateTime<Utc>, stop: DateTime<Utc>) -> Self {
        Self { start, stop }
    }
    
    /// Duration of the visibility period in seconds
    pub fn duration_seconds(&self) -> f64 {
        (self.stop - self.start).num_seconds() as f64
    }
    
    /// Duration of the visibility period in hours
    pub fn duration_hours(&self) -> f64 {
        self.duration_seconds() / 3600.0
    }
}

/// Core scheduling block data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingBlock {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub requested_duration_sec: f64,
    pub min_observation_time_sec: Option<f64>,
    
    // Time constraints
    pub fixed_start_time: Option<f64>,  // MJD
    pub fixed_stop_time: Option<f64>,   // MJD
    
    // Coordinates
    pub ra_in_deg: Option<f64>,
    pub dec_in_deg: Option<f64>,
    
    // Angle constraints
    pub min_azimuth_angle_in_deg: Option<f64>,
    pub max_azimuth_angle_in_deg: Option<f64>,
    pub min_elevation_angle_in_deg: Option<f64>,
    pub max_elevation_angle_in_deg: Option<f64>,
    
    // Scheduled period (if scheduled)
    pub scheduled_start: Option<f64>,  // MJD
    pub scheduled_stop: Option<f64>,   // MJD
    
    // Visibility periods
    pub visibility_periods: Vec<VisibilityPeriod>,
}

impl SchedulingBlock {
    /// Check if the block is scheduled
    pub fn is_scheduled(&self) -> bool {
        self.scheduled_start.is_some() && self.scheduled_stop.is_some()
    }
    
    /// Get requested duration in hours
    pub fn requested_hours(&self) -> f64 {
        self.requested_duration_sec / 3600.0
    }
    
    /// Get elevation range in degrees
    pub fn elevation_range_deg(&self) -> Option<f64> {
        match (self.max_elevation_angle_in_deg, self.min_elevation_angle_in_deg) {
            (Some(max), Some(min)) => Some(max - min),
            _ => None,
        }
    }
    
    /// Calculate total visibility hours
    pub fn total_visibility_hours(&self) -> f64 {
        self.visibility_periods
            .iter()
            .map(|p| p.duration_hours())
            .sum()
    }
    
    /// Number of visibility periods
    pub fn num_visibility_periods(&self) -> usize {
        self.visibility_periods.len()
    }
    
    /// Assign priority bin based on priority value
    pub fn priority_bin(&self) -> &'static str {
        if self.priority < 0.0 {
            "Invalid (<0)"
        } else if self.priority < 8.0 {
            "Low (0-8)"
        } else if self.priority < 10.0 {
            "Medium (8-10)"
        } else if self.priority < 12.0 {
            "High (10-12)"
        } else if self.priority < 15.0 {
            "Very High (12-15)"
        } else {
            "Critical (>15)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    
    #[test]
    fn test_visibility_period_duration() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let stop = Utc.with_ymd_and_hms(2024, 1, 1, 1, 0, 0).unwrap();
        let period = VisibilityPeriod::new(start, stop);
        
        assert_eq!(period.duration_seconds(), 3600.0);
        assert_eq!(period.duration_hours(), 1.0);
    }
    
    #[test]
    fn test_scheduling_block_is_scheduled() {
        let block = SchedulingBlock {
            scheduling_block_id: "test-1".to_string(),
            priority: 10.0,
            requested_duration_sec: 3600.0,
            min_observation_time_sec: Some(1800.0),
            fixed_start_time: None,
            fixed_stop_time: None,
            ra_in_deg: Some(180.0),
            dec_in_deg: Some(45.0),
            min_azimuth_angle_in_deg: Some(0.0),
            max_azimuth_angle_in_deg: Some(360.0),
            min_elevation_angle_in_deg: Some(30.0),
            max_elevation_angle_in_deg: Some(80.0),
            scheduled_start: Some(59580.0),
            scheduled_stop: Some(59580.5),
            visibility_periods: vec![],
        };
        
        assert!(block.is_scheduled());
        assert_eq!(block.requested_hours(), 1.0);
        assert_eq!(block.elevation_range_deg(), Some(50.0));
        assert_eq!(block.priority_bin(), "Medium (8-10)");
    }
    
    #[test]
    fn test_priority_bins() {
        let mut block = SchedulingBlock {
            scheduling_block_id: "test".to_string(),
            priority: 5.0,
            requested_duration_sec: 3600.0,
            min_observation_time_sec: None,
            fixed_start_time: None,
            fixed_stop_time: None,
            ra_in_deg: None,
            dec_in_deg: None,
            min_azimuth_angle_in_deg: None,
            max_azimuth_angle_in_deg: None,
            min_elevation_angle_in_deg: None,
            max_elevation_angle_in_deg: None,
            scheduled_start: None,
            scheduled_stop: None,
            visibility_periods: vec![],
        };
        
        assert_eq!(block.priority_bin(), "Low (0-8)");
        
        block.priority = 9.0;
        assert_eq!(block.priority_bin(), "Medium (8-10)");
        
        block.priority = 11.0;
        assert_eq!(block.priority_bin(), "High (10-12)");
        
        block.priority = 13.0;
        assert_eq!(block.priority_bin(), "Very High (12-15)");
        
        block.priority = 16.0;
        assert_eq!(block.priority_bin(), "Critical (>15)");
    }
}
