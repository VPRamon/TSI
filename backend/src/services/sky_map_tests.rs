#[cfg(test)]
mod tests {
    use crate::services::sky_map::compute_sky_map_data;
    use crate::api::LightweightBlock;

    fn create_test_block(
        id: &str,
        priority: f64,
        ra: f64,
        dec: f64,
        scheduled: bool,
    ) -> LightweightBlock {
        LightweightBlock {
            original_block_id: id.to_string(),
            priority,
            priority_bin: String::new(),
            requested_duration_seconds: qtty::Seconds::new(3600.0),
            target_ra_deg: qtty::Degrees::new(ra),
            target_dec_deg: qtty::Degrees::new(dec),
            scheduled_period: if scheduled {
                Some(crate::api::Period {
                    start: crate::api::ModifiedJulianDate::new(1000.0),
                    stop: crate::api::ModifiedJulianDate::new(2000.0),
                })
            } else {
                None
            },
        }
    }

    #[test]
    fn test_compute_sky_map_data_empty() {
        let result = compute_sky_map_data(vec![]);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.blocks.len(), 0);
        assert_eq!(data.priority_bins.len(), 0);
        assert_eq!(data.priority_min, 0.0);
        assert_eq!(data.priority_max, 10.0);
        assert_eq!(data.ra_min.value(), 0.0);
        assert_eq!(data.ra_max.value(), 360.0);
        assert_eq!(data.dec_min.value(), -90.0);
        assert_eq!(data.dec_max.value(), 90.0);
        assert_eq!(data.total_count, 0);
        assert_eq!(data.scheduled_count, 0);
        assert!(data.scheduled_time_min.is_none());
        assert!(data.scheduled_time_max.is_none());
    }

    #[test]
    fn test_compute_sky_map_data_single_block() {
        let blocks = vec![create_test_block("b1", 5.0, 180.0, 45.0, false)];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.blocks.len(), 1);
        assert_eq!(data.priority_bins.len(), 4);
        assert_eq!(data.priority_min, 5.0);
        assert_eq!(data.priority_max, 6.0); // Adjusted when min == max
        assert_eq!(data.total_count, 1);
        assert_eq!(data.scheduled_count, 0);
    }

    #[test]
    fn test_compute_sky_map_data_priority_range() {
        let blocks = vec![
            create_test_block("b1", 2.0, 0.0, -30.0, false),
            create_test_block("b2", 8.0, 120.0, 60.0, true),
            create_test_block("b3", 5.0, 240.0, 0.0, false),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.priority_min, 2.0);
        assert_eq!(data.priority_max, 8.0);
        assert_eq!(data.priority_bins.len(), 4);

        // Check bin properties
        let bin_width = (8.0 - 2.0) / 4.0;
        assert_eq!(data.priority_bins[0].min_priority, 2.0);
        assert_eq!(data.priority_bins[0].max_priority, 2.0 + bin_width);
        assert_eq!(data.priority_bins[3].max_priority, 8.0);
    }

    #[test]
    fn test_compute_sky_map_data_bin_assignment() {
        let blocks = vec![
            create_test_block("low", 1.0, 0.0, 0.0, false),
            create_test_block("high", 9.0, 90.0, 30.0, false),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        // Check that blocks have priority_bin assigned
        assert!(data.blocks[0].priority_bin.starts_with("Bin"));
        assert!(data.blocks[1].priority_bin.starts_with("Bin"));

        // Low priority should be in bin 1, high priority in bin 4
        assert!(data.blocks[0].priority_bin.contains("Bin 1"));
        assert!(data.blocks[1].priority_bin.contains("Bin 4"));
    }

    #[test]
    fn test_compute_sky_map_data_scheduled_tracking() {
        let blocks = vec![
            create_test_block("b1", 5.0, 0.0, 0.0, true),
            create_test_block("b2", 7.0, 90.0, 30.0, false),
            create_test_block("b3", 9.0, 180.0, -30.0, true),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.scheduled_count, 2);
        assert!(data.scheduled_time_min.is_some());
        assert!(data.scheduled_time_max.is_some());
        assert_eq!(data.scheduled_time_min.unwrap(), 1000.0);
        assert_eq!(data.scheduled_time_max.unwrap(), 1000.0);
    }

    #[test]
    fn test_compute_sky_map_data_ra_dec_ranges() {
        let blocks = vec![
            create_test_block("b1", 5.0, 30.0, -80.0, false),
            create_test_block("b2", 5.0, 270.0, 70.0, false),
            create_test_block("b3", 5.0, 150.0, 10.0, false),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.ra_min.value(), 30.0);
        assert_eq!(data.ra_max.value(), 270.0);
        assert_eq!(data.dec_min.value(), -80.0);
        assert_eq!(data.dec_max.value(), 70.0);
    }

    #[test]
    fn test_compute_sky_map_data_bin_colors() {
        let blocks = vec![create_test_block("b1", 5.0, 0.0, 0.0, false)];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        // Check that all 4 bins have colors assigned
        assert_eq!(data.priority_bins.len(), 4);
        assert_eq!(data.priority_bins[0].color, "#2ca02c"); // Green
        assert_eq!(data.priority_bins[1].color, "#1f77b4"); // Blue
        assert_eq!(data.priority_bins[2].color, "#ff7f0e"); // Orange
        assert_eq!(data.priority_bins[3].color, "#d62728"); // Red
    }

    #[test]
    fn test_compute_sky_map_data_edge_priority_goes_to_last_bin() {
        // When priority equals priority_max, it should go to the last bin
        let blocks = vec![
            create_test_block("b1", 0.0, 0.0, 0.0, false),
            create_test_block("b2", 10.0, 90.0, 30.0, false),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        // Block with max priority should be in the last bin
        let max_priority_block = data
            .blocks
            .iter()
            .find(|b| b.priority == 10.0)
            .unwrap();
        assert!(max_priority_block.priority_bin.contains("Bin 4"));
    }

    #[test]
    fn test_compute_sky_map_data_boundary_values() {
        let blocks = vec![
            create_test_block("b1", 0.0, 0.0, -90.0, false),
            create_test_block("b2", 10.0, 360.0, 90.0, false),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.ra_min.value(), 0.0);
        assert_eq!(data.ra_max.value(), 360.0);
        assert_eq!(data.dec_min.value(), -90.0);
        assert_eq!(data.dec_max.value(), 90.0);
    }
}
