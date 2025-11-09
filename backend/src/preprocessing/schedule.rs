/// Preprocessing logic for scheduling blocks

use crate::models::schedule::{PriorityBin, SchedulingBlock};

#[cfg(test)]
use crate::models::schedule::VisibilityPeriod;

/// Progress callback signature for SSE streaming
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send + Sync>;

/// Preprocess a scheduling block: compute derived fields
/// This is the core preprocessing logic ported from Python's SchedulePreprocessor
pub fn preprocess_block(block: &mut SchedulingBlock) {
    // Compute derived fields
    block.num_visibility_periods = block.visibility.len();
    block.total_visibility_hours = block
        .visibility
        .iter()
        .map(|v| v.duration_hours())
        .sum();
    block.priority_bin = PriorityBin::from_priority(block.priority);
    block.scheduled_flag = block.scheduled_period_start.is_some() && block.scheduled_period_stop.is_some();
    block.requested_hours = block.requested_duration_sec / 3600.0;
    block.elevation_range_deg = block.max_elevation_angle_in_deg - block.min_elevation_angle_in_deg;
}

/// Preprocess a batch of scheduling blocks with optional progress reporting
pub fn preprocess_blocks(blocks: &mut [SchedulingBlock], progress: Option<&ProgressCallback>) {
    let total = blocks.len();
    for (i, block) in blocks.iter_mut().enumerate() {
        preprocess_block(block);
        
        // Report progress every 100 blocks
        if let Some(cb) = &progress {
            if i % 100 == 0 || i == total - 1 {
                cb(i + 1, total, "Preprocessing scheduling blocks");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block() -> SchedulingBlock {
        SchedulingBlock {
            scheduling_block_id: "test".to_string(),
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
            scheduled_period_start: Some(61892.0),
            scheduled_period_stop: Some(61893.0),
            visibility: vec![
                VisibilityPeriod {
                    start: 61892.0,
                    stop: 61892.5,
                },
                VisibilityPeriod {
                    start: 61893.0,
                    stop: 61893.25,
                },
            ],
            num_visibility_periods: 0, // Will be computed
            total_visibility_hours: 0.0, // Will be computed
            priority_bin: PriorityBin::NoPriority, // Will be computed
            scheduled_flag: false, // Will be computed
            requested_hours: 0.0, // Will be computed
            elevation_range_deg: 0.0, // Will be computed
        }
    }

    #[test]
    fn test_preprocess_block() {
        let mut block = create_test_block();
        preprocess_block(&mut block);

        assert_eq!(block.num_visibility_periods, 2);
        assert!((block.total_visibility_hours - 18.0).abs() < 1e-9); // 12 + 6 hours
        assert!(matches!(block.priority_bin, PriorityBin::MediumHigh));
        assert!(block.scheduled_flag);
        assert!((block.requested_hours - 1.0).abs() < 1e-9); // 3600 / 3600
        assert!((block.elevation_range_deg - 30.0).abs() < 1e-9); // 90 - 60
    }

    #[test]
    fn test_preprocess_unscheduled() {
        let mut block = create_test_block();
        block.scheduled_period_start = None;
        block.scheduled_period_stop = None;
        preprocess_block(&mut block);

        assert!(!block.scheduled_flag);
    }
}
