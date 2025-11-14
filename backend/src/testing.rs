//! Test fixtures and utilities for backend testing
//! 
//! This module provides common test utilities, fixtures, and helper functions
//! to reduce code duplication in tests.

#[cfg(test)]
pub mod fixtures {
    use crate::models::schedule::{PriorityBin, SchedulingBlock, VisibilityPeriod};

    /// Create a test scheduling block with customizable parameters
    /// 
    /// # Arguments
    /// * `id` - Scheduling block identifier
    /// * `scheduled` - Whether the block is scheduled
    /// * `priority` - Priority value (0.0-20.0)
    /// 
    /// # Returns
    /// A SchedulingBlock with sensible defaults for testing
    pub fn create_test_block(id: &str, scheduled: bool, priority: f64) -> SchedulingBlock {
        let scheduled_start = if scheduled { Some(61892.0) } else { None };
        let scheduled_stop = if scheduled { Some(61893.0) } else { None };

        SchedulingBlock {
            scheduling_block_id: id.to_string(),
            priority,
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
            scheduled_period_start: scheduled_start,
            scheduled_period_stop: scheduled_stop,
            visibility: vec![VisibilityPeriod {
                start: 61892.0,
                stop: 61893.0,
            }],
            num_visibility_periods: 1,
            total_visibility_hours: 24.0,
            priority_bin: PriorityBin::from_priority(priority),
            scheduled_flag: scheduled,
            requested_hours: 1.0,
            elevation_range_deg: 30.0,
        }
    }

    /// Create a dataset of test blocks
    /// 
    /// # Arguments
    /// * `size` - Number of blocks to create
    /// * `scheduled_ratio` - Ratio of scheduled blocks (0.0-1.0)
    /// 
    /// # Returns
    /// Vector of scheduling blocks with varied properties
    pub fn create_test_dataset(size: usize, scheduled_ratio: f64) -> Vec<SchedulingBlock> {
        (0..size)
            .map(|i| {
                let scheduled = (i as f64) < (size as f64 * scheduled_ratio);
                let priority = 5.0 + (i as f64 % 10.0);
                create_test_block(&format!("block_{}", i), scheduled, priority)
            })
            .collect()
    }

    /// Create a block with insufficient visibility (impossible observation)
    pub fn create_impossible_block(id: &str) -> SchedulingBlock {
        SchedulingBlock {
            scheduling_block_id: id.to_string(),
            priority: 10.0,
            min_observation_time_in_sec: 36000.0, // 10 hours
            requested_duration_sec: 36000.0,
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
                start: 61892.0,
                stop: 61892.1, // Only 2.4 hours
            }],
            num_visibility_periods: 1,
            total_visibility_hours: 2.4,
            priority_bin: PriorityBin::High,
            scheduled_flag: false,
            requested_hours: 10.0,
            elevation_range_deg: 30.0,
        }
    }

    /// Create a block with custom visibility periods
    pub fn create_block_with_visibility(
        id: &str,
        visibility_periods: Vec<VisibilityPeriod>,
    ) -> SchedulingBlock {
        let total_hours: f64 = visibility_periods
            .iter()
            .map(|v| v.duration_hours())
            .sum();

        SchedulingBlock {
            scheduling_block_id: id.to_string(),
            priority: 8.5,
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
            scheduled_period_start: None,
            scheduled_period_stop: None,
            visibility: visibility_periods.clone(),
            num_visibility_periods: visibility_periods.len(),
            total_visibility_hours: total_hours,
            priority_bin: PriorityBin::MediumHigh,
            scheduled_flag: false,
            requested_hours: 1.0,
            elevation_range_deg: 30.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;

    #[test]
    fn test_create_test_block() {
        let block = create_test_block("test1", true, 9.5);
        assert_eq!(block.scheduling_block_id, "test1");
        assert!(block.scheduled_flag);
        assert_eq!(block.priority, 9.5);
        assert!(block.scheduled_period_start.is_some());
    }

    #[test]
    fn test_create_test_dataset() {
        let blocks = create_test_dataset(10, 0.6);
        assert_eq!(blocks.len(), 10);
        
        let scheduled_count = blocks.iter().filter(|b| b.scheduled_flag).count();
        assert_eq!(scheduled_count, 6);
    }

    #[test]
    fn test_create_impossible_block() {
        let block = create_impossible_block("impossible1");
        assert!(block.is_impossible(1.0));
        assert_eq!(block.total_visibility_hours, 2.4);
        assert_eq!(block.requested_hours, 10.0);
    }

    #[test]
    fn test_create_block_with_visibility() {
        use crate::models::schedule::VisibilityPeriod;
        
        let visibility = vec![
            VisibilityPeriod {
                start: 61892.0,
                stop: 61892.5,
            },
            VisibilityPeriod {
                start: 61893.0,
                stop: 61893.25,
            },
        ];
        
        let block = create_block_with_visibility("test_vis", visibility);
        assert_eq!(block.num_visibility_periods, 2);
        assert_eq!(block.total_visibility_hours, 18.0); // 12 + 6 hours
    }
}
