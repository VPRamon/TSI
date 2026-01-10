#[cfg(test)]
mod tests {
    use crate::services::insights::{compute_metrics, compute_spearman_correlation, compute_correlations};
    use crate::api::{InsightsBlock, ModifiedJulianDate};

    fn create_test_block(priority: f64, scheduled: bool, visibility: f64, requested: f64) -> InsightsBlock {
        InsightsBlock {
            scheduling_block_id: 1,
            original_block_id: "test".to_string(),
            priority,
            total_visibility_hours: qtty::Hours::new(visibility),
            requested_hours: qtty::Hours::new(requested),
            elevation_range_deg: qtty::Degrees::new(30.0),
            scheduled,
            scheduled_start_mjd: None,
            scheduled_stop_mjd: None,
        }
    }

    #[test]
    fn test_compute_metrics_empty() {
        let blocks = vec![];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.total_observations, 0);
        assert_eq!(metrics.scheduled_count, 0);
        assert_eq!(metrics.scheduling_rate, 0.0);
    }

    #[test]
    fn test_compute_metrics_basic() {
        let blocks = vec![
            create_test_block(5.0, true, 10.0, 2.0),
            create_test_block(7.0, false, 15.0, 3.0),
            create_test_block(9.0, true, 20.0, 4.0),
        ];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.total_observations, 3);
        assert_eq!(metrics.scheduled_count, 2);
        assert_eq!(metrics.unscheduled_count, 1);
        assert!((metrics.scheduling_rate - 2.0/3.0).abs() < 1e-6);
        assert_eq!(metrics.mean_priority, 7.0);
        assert_eq!(metrics.median_priority, 7.0);
    }

    #[test]
    fn test_compute_metrics_median_even() {
        let blocks = vec![
            create_test_block(2.0, true, 10.0, 1.0),
            create_test_block(4.0, false, 15.0, 2.0),
            create_test_block(6.0, true, 20.0, 3.0),
            create_test_block(8.0, false, 25.0, 4.0),
        ];
        let metrics = compute_metrics(&blocks);

        assert_eq!(metrics.median_priority, 5.0); // (4.0 + 6.0) / 2
    }

    #[test]
    fn test_compute_spearman_correlation_empty() {
        let corr = compute_spearman_correlation(&[], &[]);
        assert_eq!(corr, 0.0);
    }

    #[test]
    fn test_compute_spearman_correlation_perfect_positive() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let corr = compute_spearman_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_spearman_correlation_perfect_negative() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let corr = compute_spearman_correlation(&x, &y);
        assert!((corr + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_compute_spearman_correlation_no_relationship() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![5.0, 5.0, 5.0];
        let corr = compute_spearman_correlation(&x, &y);
        // When one variable is constant, Spearman will assign tied ranks
        // Result may vary but should be valid
        assert!(corr >= -1.0 && corr <= 1.0);
    }

    #[test]
    fn test_compute_spearman_correlation_length_mismatch() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![1.0, 2.0];
        let corr = compute_spearman_correlation(&x, &y);
        assert_eq!(corr, 0.0);
    }

    #[test]
    fn test_compute_correlations_empty() {
        let blocks = vec![];
        let corrs = compute_correlations(&blocks);
        assert_eq!(corrs.len(), 0);
    }

    #[test]
    fn test_compute_correlations_single_block() {
        let blocks = vec![create_test_block(5.0, true, 10.0, 2.0)];
        let corrs = compute_correlations(&blocks);
        assert_eq!(corrs.len(), 0);
    }

    #[test]
    fn test_compute_correlations_multiple() {
        let blocks = vec![
            create_test_block(1.0, false, 5.0, 1.0),
            create_test_block(5.0, true, 15.0, 3.0),
            create_test_block(9.0, true, 25.0, 5.0),
        ];
        let corrs = compute_correlations(&blocks);
        // Should have some correlations computed
        assert!(corrs.len() > 0);
        assert!(corrs.iter().all(|c| c.correlation >= -1.0 && c.correlation <= 1.0));
    }
}
