#[cfg(test)]
mod tests {
    use crate::services::compare::{compute_compare_data, compute_stats};
    use crate::api::CompareBlock;

    fn create_test_block(
        id: &str,
        priority: f64,
        scheduled: bool,
        hours: f64,
    ) -> CompareBlock {
        CompareBlock {
            scheduling_block_id: id.to_string(),
            priority,
            scheduled,
            requested_hours: qtty::Hours::new(hours),
        }
    }

    #[test]
    fn test_compute_stats_empty() {
        let blocks = vec![];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 0);
        assert_eq!(stats.unscheduled_count, 0);
        assert_eq!(stats.total_priority, 0.0);
        assert_eq!(stats.mean_priority, 0.0);
        assert_eq!(stats.median_priority, 0.0);
        assert_eq!(stats.total_hours.value(), 0.0);
    }

    #[test]
    fn test_compute_stats_all_unscheduled() {
        let blocks = vec![
            create_test_block("b1", 5.0, false, 1.0),
            create_test_block("b2", 7.0, false, 2.0),
        ];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 0);
        assert_eq!(stats.unscheduled_count, 2);
        assert_eq!(stats.total_priority, 0.0);
        assert_eq!(stats.mean_priority, 0.0);
        assert_eq!(stats.median_priority, 0.0);
        assert_eq!(stats.total_hours.value(), 0.0);
    }

    #[test]
    fn test_compute_stats_single_scheduled() {
        let blocks = vec![create_test_block("b1", 8.5, true, 3.5)];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 1);
        assert_eq!(stats.unscheduled_count, 0);
        assert_eq!(stats.total_priority, 8.5);
        assert_eq!(stats.mean_priority, 8.5);
        assert_eq!(stats.median_priority, 8.5);
        assert_eq!(stats.total_hours.value(), 3.5);
    }

    #[test]
    fn test_compute_stats_odd_count() {
        let blocks = vec![
            create_test_block("b1", 3.0, true, 1.0),
            create_test_block("b2", 5.0, true, 2.0),
            create_test_block("b3", 7.0, true, 3.0),
        ];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 3);
        assert_eq!(stats.unscheduled_count, 0);
        assert_eq!(stats.total_priority, 15.0);
        assert_eq!(stats.mean_priority, 5.0);
        assert_eq!(stats.median_priority, 5.0); // Middle value
        assert_eq!(stats.total_hours.value(), 6.0);
    }

    #[test]
    fn test_compute_stats_even_count() {
        let blocks = vec![
            create_test_block("b1", 2.0, true, 1.0),
            create_test_block("b2", 4.0, true, 1.5),
            create_test_block("b3", 6.0, true, 2.0),
            create_test_block("b4", 8.0, true, 2.5),
        ];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 4);
        assert_eq!(stats.unscheduled_count, 0);
        assert_eq!(stats.total_priority, 20.0);
        assert_eq!(stats.mean_priority, 5.0);
        assert_eq!(stats.median_priority, 5.0); // (4.0 + 6.0) / 2
        assert_eq!(stats.total_hours.value(), 7.0);
    }

    #[test]
    fn test_compute_stats_mixed() {
        let blocks = vec![
            create_test_block("b1", 3.0, true, 1.0),
            create_test_block("b2", 5.0, false, 2.0),
            create_test_block("b3", 7.0, true, 3.0),
            create_test_block("b4", 9.0, false, 4.0),
        ];
        let stats = compute_stats(&blocks);

        assert_eq!(stats.scheduled_count, 2);
        assert_eq!(stats.unscheduled_count, 2);
        assert_eq!(stats.total_priority, 10.0);
        assert_eq!(stats.mean_priority, 5.0);
        assert_eq!(stats.median_priority, 5.0); // (3.0 + 7.0) / 2
        assert_eq!(stats.total_hours.value(), 4.0);
    }

    #[test]
    fn test_compute_compare_data_empty() {
        let result = compute_compare_data(vec![], vec![], "Current".into(), "Comparison".into());

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.current_blocks.len(), 0);
        assert_eq!(data.comparison_blocks.len(), 0);
        assert_eq!(data.common_ids.len(), 0);
        assert_eq!(data.only_in_current.len(), 0);
        assert_eq!(data.only_in_comparison.len(), 0);
        assert_eq!(data.scheduling_changes.len(), 0);
    }

    #[test]
    fn test_compute_compare_data_identical() {
        let current = vec![
            create_test_block("b1", 5.0, true, 1.0),
            create_test_block("b2", 7.0, false, 2.0),
        ];
        let comparison = vec![
            create_test_block("b1", 5.0, true, 1.0),
            create_test_block("b2", 7.0, false, 2.0),
        ];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.common_ids.len(), 2);
        assert_eq!(data.only_in_current.len(), 0);
        assert_eq!(data.only_in_comparison.len(), 0);
        assert_eq!(data.scheduling_changes.len(), 0);
    }

    #[test]
    fn test_compute_compare_data_newly_scheduled() {
        let current = vec![create_test_block("b1", 5.0, false, 1.0)];
        let comparison = vec![create_test_block("b1", 5.0, true, 1.0)];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.scheduling_changes.len(), 1);
        assert_eq!(data.scheduling_changes[0].change_type, "newly_scheduled");
        assert_eq!(data.scheduling_changes[0].priority, 5.0);
    }

    #[test]
    fn test_compute_compare_data_newly_unscheduled() {
        let current = vec![create_test_block("b1", 8.0, true, 2.0)];
        let comparison = vec![create_test_block("b1", 8.0, false, 2.0)];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.scheduling_changes.len(), 1);
        assert_eq!(
            data.scheduling_changes[0].change_type,
            "newly_unscheduled"
        );
        assert_eq!(data.scheduling_changes[0].priority, 8.0);
    }

    #[test]
    fn test_compute_compare_data_only_in_current() {
        let current = vec![
            create_test_block("b1", 5.0, true, 1.0),
            create_test_block("b2", 7.0, false, 2.0),
        ];
        let comparison = vec![create_test_block("b1", 5.0, true, 1.0)];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.only_in_current.len(), 1);
        assert!(data.only_in_current.contains(&"b2".to_string()));
        assert_eq!(data.only_in_comparison.len(), 0);
    }

    #[test]
    fn test_compute_compare_data_only_in_comparison() {
        let current = vec![create_test_block("b1", 5.0, true, 1.0)];
        let comparison = vec![
            create_test_block("b1", 5.0, true, 1.0),
            create_test_block("b3", 9.0, false, 3.0),
        ];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.only_in_current.len(), 0);
        assert_eq!(data.only_in_comparison.len(), 1);
        assert!(data.only_in_comparison.contains(&"b3".to_string()));
    }

    #[test]
    fn test_compute_compare_data_complex() {
        let current = vec![
            create_test_block("common1", 3.0, false, 1.0),
            create_test_block("common2", 5.0, true, 2.0),
            create_test_block("only_current", 7.0, true, 3.0),
        ];
        let comparison = vec![
            create_test_block("common1", 3.0, true, 1.0), // Newly scheduled
            create_test_block("common2", 5.0, false, 2.0), // Newly unscheduled
            create_test_block("only_comparison", 9.0, false, 4.0),
        ];

        let result = compute_compare_data(current, comparison, "A".into(), "B".into());
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.common_ids.len(), 2);
        assert_eq!(data.only_in_current.len(), 1);
        assert_eq!(data.only_in_comparison.len(), 1);
        assert_eq!(data.scheduling_changes.len(), 2);

        // Check scheduling changes
        let newly_scheduled = data
            .scheduling_changes
            .iter()
            .find(|c| c.change_type == "newly_scheduled");
        assert!(newly_scheduled.is_some());
        assert_eq!(
            newly_scheduled.unwrap().scheduling_block_id,
            "common1".to_string()
        );

        let newly_unscheduled = data
            .scheduling_changes
            .iter()
            .find(|c| c.change_type == "newly_unscheduled");
        assert!(newly_unscheduled.is_some());
        assert_eq!(
            newly_unscheduled.unwrap().scheduling_block_id,
            "common2".to_string()
        );
    }
}
