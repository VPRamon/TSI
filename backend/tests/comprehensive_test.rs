//! Comprehensive integration tests for the backend API
//! 
//! These tests verify the complete behavior of the API endpoints
//! including error handling, edge cases, and data validation.

use tsi_backend::{
    analytics::metrics::compute_metrics,
    models::schedule::SchedulingBlock,
    state::AppState,
};

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create test blocks
    fn create_test_block(id: &str, scheduled: bool, priority: f64) -> SchedulingBlock {
        use tsi_backend::models::schedule::{PriorityBin, VisibilityPeriod};
        
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
            scheduled_period_start: if scheduled { Some(61892.0) } else { None },
            scheduled_period_stop: if scheduled { Some(61893.0) } else { None },
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

    #[test]
    fn test_state_lifecycle() {
        let state = AppState::new();
        
        // Initially empty
        assert!(!state.has_dataset());
        
        // Load dataset
        let blocks = vec![
            create_test_block("block1", true, 10.0),
            create_test_block("block2", false, 5.0),
        ];
        
        let metadata = state.load_dataset(blocks.clone(), "test.csv".to_string())
            .expect("Failed to load dataset");
        
        assert_eq!(metadata.num_blocks, 2);
        assert_eq!(metadata.num_scheduled, 1);
        assert_eq!(metadata.num_unscheduled, 1);
        assert!(state.has_dataset());
        
        // Get dataset
        let (loaded_blocks, loaded_meta) = state.get_dataset()
            .expect("Failed to get dataset")
            .expect("No dataset found");
        
        assert_eq!(loaded_blocks.len(), 2);
        assert_eq!(loaded_meta.filename, "test.csv");
        
        // Clear dataset
        state.clear_dataset().expect("Failed to clear dataset");
        assert!(!state.has_dataset());
    }

    #[test]
    fn test_comparison_dataset_workflow() {
        let state = AppState::new();
        
        // Load primary dataset
        let primary_blocks = vec![
            create_test_block("block1", true, 10.0),
            create_test_block("block2", false, 5.0),
        ];
        state.load_dataset(primary_blocks, "primary.csv".to_string())
            .expect("Failed to load primary");
        
        // Load comparison dataset
        let comparison_blocks = vec![
            create_test_block("block1", false, 10.0),
            create_test_block("block2", true, 5.0),
            create_test_block("block3", true, 8.0),
        ];
        state.load_comparison_dataset(comparison_blocks, "comparison.csv".to_string())
            .expect("Failed to load comparison");
        
        // Verify both datasets exist
        assert!(state.get_dataset().expect("Error getting dataset").is_some());
        assert!(state.get_comparison_dataset().expect("Error getting comparison").is_some());
        
        // Clear comparison
        state.clear_comparison_dataset().expect("Failed to clear comparison");
        assert!(state.get_comparison_dataset().expect("Error getting comparison").is_none());
        assert!(state.get_dataset().expect("Error getting dataset").is_some());
    }

    #[test]
    fn test_metrics_computation() {
        let blocks = vec![
            create_test_block("block1", true, 10.0),
            create_test_block("block2", true, 8.5),
            create_test_block("block3", false, 5.0),
            create_test_block("block4", false, 3.0),
        ];
        
        let metrics = compute_metrics(&blocks);
        
        assert_eq!(metrics.total_blocks, 4);
        assert_eq!(metrics.scheduled_blocks, 2);
        assert_eq!(metrics.unscheduled_blocks, 2);
        assert!((metrics.scheduling_rate - 0.5).abs() < 1e-9);
        
        assert_eq!(metrics.priority_stats.count, 4);
        assert!((metrics.priority_stats.mean - 6.625).abs() < 0.01);
    }

    #[test]
    fn test_with_dataset_helper() {
        let state = AppState::new();
        
        // Should error when no dataset loaded
        let result = state.with_dataset(|blocks| blocks.len());
        assert!(result.is_err());
        
        // Load dataset
        let blocks = vec![
            create_test_block("block1", true, 10.0),
            create_test_block("block2", false, 5.0),
        ];
        state.load_dataset(blocks, "test.csv".to_string())
            .expect("Failed to load dataset");
        
        // Should work now
        let count = state.with_dataset(|blocks| blocks.len())
            .expect("Failed to access dataset");
        assert_eq!(count, 2);
        
        // Can compute on the fly
        let scheduled_count = state.with_dataset(|blocks| {
            blocks.iter().filter(|b| b.scheduled_flag).count()
        }).expect("Failed to access dataset");
        assert_eq!(scheduled_count, 1);
    }

    #[test]
    fn test_concurrent_read_access() {
        use std::sync::Arc;
        use std::thread;
        
        let state = Arc::new(AppState::new());
        
        // Load dataset
        let blocks = vec![
            create_test_block("block1", true, 10.0),
            create_test_block("block2", false, 5.0),
        ];
        state.load_dataset(blocks, "test.csv".to_string())
            .expect("Failed to load dataset");
        
        // Spawn multiple readers
        let mut handles = vec![];
        for _ in 0..5 {
            let state_clone = Arc::clone(&state);
            let handle = thread::spawn(move || {
                let count = state_clone.with_dataset(|blocks| blocks.len())
                    .expect("Failed to read");
                assert_eq!(count, 2);
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    fn test_empty_dataset() {
        let state = AppState::new();
        let empty_blocks: Vec<SchedulingBlock> = vec![];
        
        let metadata = state.load_dataset(empty_blocks, "empty.csv".to_string())
            .expect("Failed to load empty dataset");
        
        assert_eq!(metadata.num_blocks, 0);
        assert_eq!(metadata.num_scheduled, 0);
        assert_eq!(metadata.num_unscheduled, 0);
        
        let metrics = state.with_dataset(|blocks| compute_metrics(blocks))
            .expect("Failed to compute metrics");
        
        assert_eq!(metrics.total_blocks, 0);
        assert_eq!(metrics.scheduling_rate, 0.0);
    }

    #[test]
    fn test_large_dataset_performance() {
        let state = AppState::new();
        
        // Create large dataset
        let mut blocks = Vec::with_capacity(10000);
        for i in 0..10000 {
            blocks.push(create_test_block(
                &format!("block_{}", i),
                i % 2 == 0,
                5.0 + (i % 10) as f64,
            ));
        }
        
        // Should handle large datasets efficiently
        let start = std::time::Instant::now();
        let metadata = state.load_dataset(blocks, "large.csv".to_string())
            .expect("Failed to load large dataset");
        let load_time = start.elapsed();
        
        assert_eq!(metadata.num_blocks, 10000);
        assert!(load_time.as_millis() < 1000, "Loading took too long: {:?}", load_time);
        
        // Metrics computation should also be fast
        let start = std::time::Instant::now();
        let _metrics = state.with_dataset(|blocks| compute_metrics(blocks))
            .expect("Failed to compute metrics");
        let metrics_time = start.elapsed();
        
        assert!(metrics_time.as_millis() < 500, "Metrics computation took too long: {:?}", metrics_time);
    }
}
