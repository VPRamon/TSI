//! Integration tests for visibility histogram computation.
#![allow(clippy::len_zero)]

#[cfg(test)]
mod visibility_histogram_tests {
    use tsi_rust::api::{ModifiedJulianDate, Period};
    use tsi_rust::db::models::BlockHistogramData;
    use tsi_rust::services::visibility::compute_visibility_histogram_rust;

    /// Helper to create test block with visibility periods
    fn make_block_from_json(id: i64, priority: i32, vis_json: &str) -> BlockHistogramData {
        // Parse JSON to Vec<Period>
        let periods: Vec<serde_json::Value> = serde_json::from_str(vis_json).unwrap();
        let visibility_periods: Vec<Period> = periods
            .iter()
            .map(|p| Period {
                start: ModifiedJulianDate::new(p["start"].as_f64().unwrap()),
                stop: ModifiedJulianDate::new(p["stop"].as_f64().unwrap()),
            })
            .collect();

        BlockHistogramData {
            scheduling_block_id: id,
            priority,
            visibility_periods: Some(visibility_periods),
        }
    }

    #[test]
    fn test_empty_blocks() {
        let blocks: Vec<BlockHistogramData> = vec![];
        let bins = compute_visibility_histogram_rust(
            blocks.into_iter(),
            0,
            86400, // 1 day
            3600,  // 1 hour bins
            None,
            None,
        )
        .unwrap();

        assert_eq!(bins.len(), 24);
        assert!(bins.iter().all(|b| b.visible_count == 0));
    }

    #[test]
    fn test_single_block_full_day() {
        // MJD 40587 = Unix epoch (1970-01-01)
        let block = make_block_from_json(1, 5, r#"[{"start": 40587.0, "stop": 40588.0}]"#);

        let bins = compute_visibility_histogram_rust(
            vec![block].into_iter(),
            0,     // 1970-01-01 00:00:00
            86400, // 1970-01-02 00:00:00
            3600,  // 1 hour bins
            None,
            None,
        )
        .unwrap();

        assert_eq!(bins.len(), 24);
        // All bins should have the block visible
        assert!(bins.iter().all(|b| b.visible_count == 1));
    }

    #[test]
    fn test_priority_filtering() {
        let blocks = vec![
            make_block_from_json(1, 3, r#"[{"start": 40587.0, "stop": 40587.5}]"#),
            make_block_from_json(2, 5, r#"[{"start": 40587.0, "stop": 40587.5}]"#),
            make_block_from_json(3, 7, r#"[{"start": 40587.0, "stop": 40587.5}]"#),
            make_block_from_json(4, 10, r#"[{"start": 40587.0, "stop": 40587.5}]"#),
        ];

        // Filter for priority >= 5 and <= 8
        let bins =
            compute_visibility_histogram_rust(blocks.into_iter(), 0, 86400, 3600, Some(5), Some(8))
                .unwrap();

        // Only blocks 2 (priority 5) and 3 (priority 7) should be counted
        let max_count = bins.iter().map(|b| b.visible_count).max().unwrap();
        assert_eq!(max_count, 2);
    }

    #[test]
    fn test_multiple_periods_same_block() {
        // Block with multiple non-overlapping visibility windows
        let block = make_block_from_json(
            1,
            5,
            r#"[
                {"start": 40587.0, "stop": 40587.1},
                {"start": 40587.3, "stop": 40587.4},
                {"start": 40587.7, "stop": 40587.8}
            ]"#,
        );

        let bins = compute_visibility_histogram_rust(
            vec![block].into_iter(),
            0,
            86400,
            3600, // 1 hour bins
            None,
            None,
        )
        .unwrap();

        // Each period is ~2.4 hours
        // Should cover multiple bins but count block only once per bin
        let visible_bins: Vec<_> = bins.iter().filter(|b| b.visible_count > 0).collect();
        assert!(visible_bins.len() > 0);
        // Each bin should count the block at most once
        assert!(visible_bins.iter().all(|b| b.visible_count == 1));
    }

    #[test]
    fn test_overlapping_blocks_different_ids() {
        let blocks = vec![
            make_block_from_json(1, 5, r#"[{"start": 40587.0, "stop": 40587.25}]"#),
            make_block_from_json(2, 5, r#"[{"start": 40587.1, "stop": 40587.35}]"#),
            make_block_from_json(3, 5, r#"[{"start": 40587.2, "stop": 40587.45}]"#),
        ];

        let bins =
            compute_visibility_histogram_rust(blocks.into_iter(), 0, 86400, 3600, None, None)
                .unwrap();

        // Find bins with overlapping periods
        let max_count = bins.iter().map(|b| b.visible_count).max().unwrap();
        // All three blocks overlap in some time range
        assert!(max_count >= 2);
    }

    #[test]
    fn test_edge_case_period_touches_bin_boundary() {
        // Period that exactly touches bin boundaries
        let block = make_block_from_json(1, 5, r#"[{"start": 40587.0, "stop": 40587.041666667}]"#);
        // 0.041666667 days = 1 hour

        let bins = compute_visibility_histogram_rust(
            vec![block].into_iter(),
            0,
            86400,
            3600, // 1 hour bins
            None,
            None,
        )
        .unwrap();

        // Should be visible in first bin
        assert_eq!(bins[0].visible_count, 1);
        // Should not be visible in second bin (period ends exactly at bin boundary)
        assert_eq!(bins[1].visible_count, 0);
    }

    #[test]
    fn test_period_spanning_multiple_bins() {
        // Period spanning 5 hours
        let block = make_block_from_json(1, 5, r#"[{"start": 40587.0, "stop": 40587.208333333}]"#);
        // 0.208333333 days = 5 hours

        let bins = compute_visibility_histogram_rust(
            vec![block].into_iter(),
            0,
            86400,
            3600, // 1 hour bins
            None,
            None,
        )
        .unwrap();

        // First 5 bins should have the block
        assert_eq!(bins[0].visible_count, 1);
        assert_eq!(bins[1].visible_count, 1);
        assert_eq!(bins[2].visible_count, 1);
        assert_eq!(bins[3].visible_count, 1);
        assert_eq!(bins[4].visible_count, 1);
        assert_eq!(bins[5].visible_count, 0);
    }

    #[test]
    fn test_invalid_period_ignored() {
        // Period with start >= stop should be ignored
        let block = make_block_from_json(
            1,
            5,
            r#"[
                {"start": 40587.5, "stop": 40587.2},
                {"start": 40587.0, "stop": 40587.1}
            ]"#,
        );

        let bins =
            compute_visibility_histogram_rust(vec![block].into_iter(), 0, 86400, 3600, None, None)
                .unwrap();

        // Only the valid period should be counted
        let visible_bins: Vec<_> = bins.iter().filter(|b| b.visible_count > 0).collect();
        assert!(visible_bins.len() > 0);
    }

    #[test]
    fn test_none_visibility_periods_handled() {
        // Test that blocks with None visibility periods (parse errors at repo layer) are handled
        let block = BlockHistogramData {
            scheduling_block_id: 1,
            priority: 5,
            visibility_periods: None,
        };

        let bins =
            compute_visibility_histogram_rust(vec![block].into_iter(), 0, 86400, 3600, None, None)
                .unwrap();

        // Should not crash, just produce empty histogram
        assert!(bins.iter().all(|b| b.visible_count == 0));
    }

    #[test]
    fn test_empty_visibility_periods_handled() {
        // Test that blocks with empty visibility periods are handled
        let block = BlockHistogramData {
            scheduling_block_id: 1,
            priority: 5,
            visibility_periods: Some(vec![]),
        };

        let bins =
            compute_visibility_histogram_rust(vec![block].into_iter(), 0, 86400, 3600, None, None)
                .unwrap();

        // Should handle gracefully
        assert!(bins.iter().all(|b| b.visible_count == 0));
    }

    #[test]
    fn test_block_ids_filter() {
        let blocks = vec![
            make_block_from_json(100, 5, r#"[{"start": 40587.0, "stop": 40587.5}]"#),
            make_block_from_json(200, 5, r#"[{"start": 40587.0, "stop": 40587.5}]"#),
            make_block_from_json(300, 5, r#"[{"start": 40587.0, "stop": 40587.5}]"#),
        ];

        // Note: block_ids filtering happens at DB level, not in compute function
        // This test validates the computation with specific blocks
        let filtered_blocks: Vec<_> = blocks
            .into_iter()
            .filter(|b| b.scheduling_block_id == 100 || b.scheduling_block_id == 200)
            .collect();

        let bins = compute_visibility_histogram_rust(
            filtered_blocks.into_iter(),
            0,
            86400,
            3600,
            None,
            None,
        )
        .unwrap();

        let max_count = bins.iter().map(|b| b.visible_count).max().unwrap();
        assert_eq!(max_count, 2); // Only 2 blocks should be counted
    }

    #[test]
    fn test_variable_bin_sizes() {
        let block = make_block_from_json(1, 5, r#"[{"start": 40587.0, "stop": 40587.5}]"#);

        // Test with 30-minute bins
        let bins_30min = compute_visibility_histogram_rust(
            vec![block.clone()].into_iter(),
            0,
            86400,
            1800, // 30 minutes
            None,
            None,
        )
        .unwrap();

        // Test with 2-hour bins
        let bins_2hr = compute_visibility_histogram_rust(
            vec![block].into_iter(),
            0,
            86400,
            7200, // 2 hours
            None,
            None,
        )
        .unwrap();

        assert_eq!(bins_30min.len(), 48); // 48 half-hour bins in a day
        assert_eq!(bins_2hr.len(), 12); // 12 two-hour bins in a day
    }

    #[test]
    fn test_validation_errors() {
        let blocks = vec![];

        // start >= end
        assert!(compute_visibility_histogram_rust(
            blocks.clone().into_iter(),
            100,
            50,
            3600,
            None,
            None
        )
        .is_err());

        // zero bin duration
        assert!(
            compute_visibility_histogram_rust(blocks.into_iter(), 0, 100, 0, None, None).is_err()
        );
    }

    #[test]
    fn test_realistic_scenario() {
        // Simulate a realistic scenario with 10 blocks over 3 days
        let mut blocks = vec![];
        for i in 0..10 {
            let start_mjd = 40587.0 + (i as f64) * 0.1; // Staggered start times
            let stop_mjd = start_mjd + 0.3; // Each visible for ~7.2 hours
            let json = format!(r#"[{{"start": {}, "stop": {}}}]"#, start_mjd, stop_mjd);
            blocks.push(make_block_from_json(i, 5 + (i % 5) as i32, &json));
        }

        let bins = compute_visibility_histogram_rust(
            blocks.into_iter(),
            0,
            259200, // 3 days
            3600,   // 1 hour bins
            None,
            None,
        )
        .unwrap();

        assert_eq!(bins.len(), 72); // 72 hours in 3 days

        // At least some bins should have multiple visible blocks
        let max_count = bins.iter().map(|b| b.visible_count).max().unwrap();
        assert!(max_count > 1);

        // Some bins should have zero visibility
        let has_empty_bins = bins.iter().any(|b| b.visible_count == 0);
        assert!(has_empty_bins);
    }
}
