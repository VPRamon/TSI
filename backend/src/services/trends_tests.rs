#[cfg(test)]
mod tests {
    use crate::services::trends::{compute_by_priority, compute_metrics, compute_by_bins, compute_smoothed_trend, compute_heatmap_bins};
    use crate::api::TrendsBlock;

    fn create_test_block(priority: f64, scheduled: bool, visibility_hours: f64, requested_hours: f64) -> TrendsBlock {
        TrendsBlock {
            scheduling_block_id: priority as i64,
            original_block_id: format!("block_{}", priority),
            priority,
            scheduled,
            total_visibility_hours: qtty::Hours::new(visibility_hours),
            requested_hours: qtty::Hours::new(requested_hours),
        }
    }

    #[test]
    fn test_compute_metrics_empty() {
        let blocks = vec![];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.total_count, 0);
        assert_eq!(metrics.scheduled_count, 0);
        assert_eq!(metrics.scheduling_rate, 0.0);
        assert_eq!(metrics.zero_visibility_count, 0);
    }

    #[test]
    fn test_compute_metrics_basic() {
        let blocks = vec![
            create_test_block(5.0, true, 10.0, 2.0),
            create_test_block(7.0, false, 15.0, 3.0),
            create_test_block(9.0, true, 20.0, 4.0),
        ];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.total_count, 3);
        assert_eq!(metrics.scheduled_count, 2);
        assert!((metrics.scheduling_rate - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!(metrics.zero_visibility_count, 0);
        assert_eq!(metrics.priority_min, 5.0);
        assert_eq!(metrics.priority_max, 9.0);
        assert!((metrics.priority_mean - 7.0).abs() < 1e-6);
        assert_eq!(metrics.visibility_min, 10.0);
        assert_eq!(metrics.visibility_max, 20.0);
    }

    #[test]
    fn test_compute_metrics_zero_visibility() {
        let blocks = vec![
            create_test_block(5.0, false, 0.0, 1.0),
            create_test_block(7.0, true, 10.0, 2.0),
            create_test_block(9.0, false, 0.0, 3.0),
        ];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.zero_visibility_count, 2);
    }

    #[test]
    fn test_compute_by_priority_empty() {
        let blocks = vec![];
        let rates = compute_by_priority(&blocks);

        assert_eq!(rates.len(), 0);
    }

    #[test]
    fn test_compute_by_priority_single_priority() {
        let blocks = vec![
            create_test_block(5.0, true, 10.0, 1.0),
            create_test_block(5.1, false, 15.0, 2.0),
            create_test_block(4.9, true, 20.0, 3.0),
        ];
        let rates = compute_by_priority(&blocks);

        // All should round to priority 5
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].mid_value, 5.0);
        assert_eq!(rates[0].count, 3);
        assert!((rates[0].scheduled_rate - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_by_priority_multiple() {
        let blocks = vec![
            create_test_block(3.0, true, 10.0, 1.0),
            create_test_block(3.4, false, 15.0, 2.0),
            create_test_block(5.0, true, 20.0, 3.0),
            create_test_block(5.2, true, 25.0, 4.0),
            create_test_block(7.8, false, 30.0, 5.0),
        ];
        let rates = compute_by_priority(&blocks);

        // Should have 3 bins: 3, 5, 8
        assert_eq!(rates.len(), 3);

        // Check they're sorted by priority
        assert!(rates[0].mid_value < rates[1].mid_value);
        assert!(rates[1].mid_value < rates[2].mid_value);

        // Check priority 5 bin
        let priority_5 = rates.iter().find(|r| r.mid_value == 5.0).unwrap();
        assert_eq!(priority_5.count, 2);
        assert_eq!(priority_5.scheduled_rate, 1.0); // Both scheduled
    }

    #[test]
    fn test_compute_by_priority_all_scheduled() {
        let blocks = vec![
            create_test_block(5.0, true, 10.0, 1.0),
            create_test_block(7.0, true, 15.0, 2.0),
        ];
        let rates = compute_by_priority(&blocks);

        assert!(rates.iter().all(|r| r.scheduled_rate == 1.0));
    }

    #[test]
    fn test_compute_by_priority_none_scheduled() {
        let blocks = vec![
            create_test_block(5.0, false, 10.0, 1.0),
            create_test_block(7.0, false, 15.0, 2.0),
        ];
        let rates = compute_by_priority(&blocks);

        assert!(rates.iter().all(|r| r.scheduled_rate == 0.0));
    }

    #[test]
    fn test_compute_metrics_all_scheduled() {
        let blocks = vec![
            create_test_block(5.0, true, 10.0, 1.0),
            create_test_block(7.0, true, 15.0, 2.0),
        ];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.scheduling_rate, 1.0);
        assert_eq!(metrics.scheduled_count, 2);
    }

    #[test]
    fn test_compute_metrics_none_scheduled() {
        let blocks = vec![
            create_test_block(5.0, false, 10.0, 1.0),
            create_test_block(7.0, false, 15.0, 2.0),
        ];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.scheduling_rate, 0.0);
        assert_eq!(metrics.scheduled_count, 0);
    }

    #[test]
    fn test_compute_metrics_single_block() {
        let blocks = vec![create_test_block(5.0, true, 10.0, 2.0)];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.total_count, 1);
        assert_eq!(metrics.scheduled_count, 1);
        assert_eq!(metrics.scheduling_rate, 1.0);
        assert_eq!(metrics.priority_min, 5.0);
        assert_eq!(metrics.priority_max, 5.0);
        assert_eq!(metrics.priority_mean, 5.0);
    }

    #[test]
    fn test_compute_by_bins_empty() {
        let blocks = vec![];
        let rates = compute_by_bins(&blocks, |b| b.priority, 5, "Priority");
        assert_eq!(rates.len(), 0);
    }

    #[test]
    fn test_compute_by_bins_single_value() {
        let blocks = vec![
            create_test_block(5.0, true, 10.0, 1.0),
            create_test_block(5.0, false, 15.0, 2.0),
        ];
        let rates = compute_by_bins(&blocks, |b| b.priority, 5, "Priority");
        assert_eq!(rates.len(), 1);
        assert!((rates[0].scheduled_rate - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compute_by_bins_multiple() {
        let blocks = vec![
            create_test_block(1.0, true, 10.0, 1.0),
            create_test_block(3.0, false, 15.0, 2.0),
            create_test_block(5.0, true, 20.0, 3.0),
            create_test_block(7.0, true, 25.0, 4.0),
            create_test_block(9.0, false, 30.0, 5.0),
        ];
        let rates = compute_by_bins(&blocks, |b| b.priority, 3, "Priority");
        assert!(rates.len() > 0);
        assert!(rates.iter().all(|r| r.count > 0));
    }

    #[test]
    fn test_compute_smoothed_trend_empty() {
        let blocks = vec![];
        let smoothed = compute_smoothed_trend(&blocks, |b| b.priority, 0.5, 10);
        assert_eq!(smoothed.len(), 0);
    }

    #[test]
    fn test_compute_smoothed_trend_single_value() {
        let blocks = vec![
            create_test_block(5.0, true, 10.0, 1.0),
            create_test_block(5.0, false, 15.0, 2.0),
        ];
        let smoothed = compute_smoothed_trend(&blocks, |b| b.priority, 0.5, 5);
        assert_eq!(smoothed.len(), 1);
        assert!((smoothed[0].y_smoothed - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_compute_smoothed_trend_multiple() {
        let blocks = vec![
            create_test_block(1.0, false, 10.0, 1.0),
            create_test_block(5.0, true, 20.0, 3.0),
            create_test_block(9.0, true, 30.0, 5.0),
        ];
        let smoothed = compute_smoothed_trend(&blocks, |b| b.priority, 0.3, 5);
        assert_eq!(smoothed.len(), 5);
        assert!(smoothed.iter().all(|p| p.y_smoothed >= 0.0 && p.y_smoothed <= 1.0));
    }

    #[test]
    fn test_compute_heatmap_bins_empty() {
        let blocks = vec![];
        let bins = compute_heatmap_bins(&blocks, 3);
        assert_eq!(bins.len(), 0);
    }

    #[test]
    fn test_compute_heatmap_bins_single_value() {
        let blocks = vec![
            create_test_block(5.0, true, 10.0, 2.0),
            create_test_block(7.0, false, 10.0, 2.0),
        ];
        let bins = compute_heatmap_bins(&blocks, 3);
        // Both blocks have same visibility and time, so no bins
        assert_eq!(bins.len(), 0);
    }

    #[test]
    fn test_compute_heatmap_bins_multiple() {
        let blocks = vec![
            create_test_block(5.0, true, 5.0, 1.0),
            create_test_block(6.0, false, 10.0, 2.0),
            create_test_block(7.0, true, 15.0, 3.0),
            create_test_block(8.0, true, 20.0, 4.0),
        ];
        let bins = compute_heatmap_bins(&blocks, 2);
        assert!(bins.len() > 0);
        assert!(bins.iter().all(|b| b.count > 0));
    }
}
