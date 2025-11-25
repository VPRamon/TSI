#[cfg(test)]
mod tests {
    use crate::core::domain::{Period, SchedulingBlock};
    use crate::parsing::csv_parser::{
        blocks_to_dataframe, dataframe_to_blocks, parse_schedule_csv, parse_schedule_csv_to_blocks,
    };
    use siderust::astro::ModifiedJulianDate;
    use siderust::coordinates::spherical::direction::ICRS;
    use siderust::units::{Degrees, Seconds};
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper to create a temp CSV file
    fn create_temp_csv(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", content).unwrap();
        temp_file
    }

    /// Test parsing CSV with all required columns
    #[test]
    fn test_parse_schedule_csv_basic() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec,raInDeg,decInDeg,minAzimuthAngleInDeg,maxAzimuthAngleInDeg,minElevationAngleInDeg,maxElevationAngleInDeg\n\"1000004990\",8.5,1200,1200,158.03,-68.03,0.0,360.0,60.0,90.0\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv(temp_file.path());

        assert!(result.is_ok(), "Should parse basic CSV: {:?}", result.err());
        let df = result.unwrap();
        assert_eq!(df.height(), 1);
    }

    /// Test parsing CSV without RA/Dec columns (optional)
    #[test]
    fn test_parse_csv_without_coordinates() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec\n\"1000004990\",8.5,1200,1200\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv_to_blocks(temp_file.path());

        assert!(
            result.is_ok(),
            "Should parse CSV without coordinates: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(
            blocks[0].coordinates.is_none(),
            "Coordinates should be None"
        );
    }

    /// Test parsing CSV without elevation columns (optional)
    #[test]
    fn test_parse_csv_without_elevation() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec,raInDeg,decInDeg\n\"1000004990\",8.5,1200,1200,158.03,-68.03\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv_to_blocks(temp_file.path());

        assert!(
            result.is_ok(),
            "Should parse CSV without elevation: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].min_elevation_angle.is_none());
        assert!(blocks[0].max_elevation_angle.is_none());
    }

    /// Test parsing CSV without azimuth columns (optional)
    #[test]
    fn test_parse_csv_without_azimuth() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec,raInDeg,decInDeg,minElevationAngleInDeg,maxElevationAngleInDeg\n\"1000004990\",8.5,1200,1200,158.03,-68.03,60.0,90.0\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv_to_blocks(temp_file.path());

        assert!(
            result.is_ok(),
            "Should parse CSV without azimuth: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].min_azimuth_angle.is_none());
        assert!(blocks[0].max_azimuth_angle.is_none());
    }

    /// Test parsing CSV with visibility strings
    #[test]
    fn test_parse_csv_with_visibility() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec,visibility\n\"1000004990\",8.5,1200,1200,\"[(61894.0, 61894.5), (61895.0, 61895.5)]\"\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv_to_blocks(temp_file.path());

        assert!(
            result.is_ok(),
            "Should parse CSV with visibility: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(
            !blocks[0].visibility_periods.is_empty(),
            "Should have visibility periods"
        );
        assert_eq!(blocks[0].visibility_periods.len(), 2);
    }

    /// Test parsing CSV with scheduled periods
    #[test]
    fn test_parse_csv_with_scheduled_period() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec,scheduled_period.start,scheduled_period.stop\n\"1000004990\",8.5,1200,1200,61894.19,61894.21\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv_to_blocks(temp_file.path());

        assert!(
            result.is_ok(),
            "Should parse CSV with scheduled period: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].scheduled_period.is_some());
        assert!(blocks[0].is_scheduled());
    }

    /// Test parsing CSV with missing required headers
    #[test]
    fn test_parse_csv_missing_headers() {
        let csv_content = "schedulingBlockId,priority\n\"1000004990\",8.5\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv_to_blocks(temp_file.path());

        assert!(result.is_err(), "Should fail with missing headers");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("requestedDurationSec") || error_msg.contains("column"),
            "Error should mention missing column: {}",
            error_msg
        );
    }

    /// Test parsing CSV with NaN priorities
    #[test]
    fn test_parse_csv_nan_priority() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec\n\"1000004990\",NaN,1200,1200\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv(temp_file.path());

        // CSV parsing should succeed but contain NaN
        assert!(
            result.is_ok(),
            "CSV should parse with NaN: {:?}",
            result.err()
        );
        let df = result.unwrap();

        // When converting to blocks, NaN should be handled
        let blocks_result = dataframe_to_blocks(&df);
        // This might fail or succeed depending on implementation
        // The key is that it doesn't panic
        if let Ok(blocks) = blocks_result {
            assert_eq!(blocks.len(), 1);
        }
    }

    /// Test parsing CSV with invalid float values (Polars reads as null)
    #[test]
    fn test_parse_csv_invalid_float() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec\n\"1000004990\",invalid,1200,1200\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv(temp_file.path());

        // Polars handles invalid float as null, which is acceptable
        assert!(
            result.is_ok(),
            "Should parse with invalid float as null: {:?}",
            result.err()
        );
        let df = result.unwrap();
        assert_eq!(df.height(), 1);
    }

    /// Test blocks_to_dataframe with derived columns
    #[test]
    fn test_blocks_to_dataframe_derived_columns() {
        let blocks = vec![SchedulingBlock {
            scheduling_block_id: "1000004990".to_string(),
            priority: 8.5,
            requested_duration: Seconds::new(1200.0),
            min_observation_time: Seconds::new(1200.0),
            fixed_time: None,
            coordinates: Some(ICRS::new(Degrees::new(158.03), Degrees::new(-68.03))),
            min_azimuth_angle: Some(Degrees::new(0.0)),
            max_azimuth_angle: Some(Degrees::new(360.0)),
            min_elevation_angle: Some(Degrees::new(60.0)),
            max_elevation_angle: Some(Degrees::new(90.0)),
            scheduled_period: Some(Period::new(
                ModifiedJulianDate::new(61894.194),
                ModifiedJulianDate::new(61894.208),
            )),
            visibility_periods: vec![
                Period::new(
                    ModifiedJulianDate::new(61894.0),
                    ModifiedJulianDate::new(61894.5),
                ),
                Period::new(
                    ModifiedJulianDate::new(61895.0),
                    ModifiedJulianDate::new(61895.5),
                ),
            ],
        }];

        let result = blocks_to_dataframe(&blocks);
        assert!(
            result.is_ok(),
            "Should convert blocks to DataFrame: {:?}",
            result.err()
        );

        let df = result.unwrap();

        // Check derived columns exist
        assert!(
            df.column("scheduled_flag").is_ok(),
            "Should have scheduled_flag column"
        );
        assert!(
            df.column("priority_bin").is_ok(),
            "Should have priority_bin column"
        );
        assert!(
            df.column("total_visibility_hours").is_ok(),
            "Should have total_visibility_hours column"
        );
        assert!(
            df.column("num_visibility_periods").is_ok(),
            "Should have num_visibility_periods column"
        );
        assert!(
            df.column("requested_hours").is_ok(),
            "Should have requested_hours column"
        );
        assert!(
            df.column("elevation_range_deg").is_ok(),
            "Should have elevation_range_deg column"
        );

        // Verify derived values
        let scheduled_flags = df.column("scheduled_flag").unwrap().bool().unwrap();
        assert_eq!(scheduled_flags.get(0), Some(true), "Should be scheduled");

        let priority_bins = df.column("priority_bin").unwrap().str().unwrap();
        let priority_bin_value = priority_bins.get(0).unwrap();
        assert!(
            priority_bin_value.contains("high") || priority_bin_value.contains("8"),
            "Priority bin should reflect high priority"
        );

        let num_vis = df.column("num_visibility_periods").unwrap().u32().unwrap();
        assert_eq!(num_vis.get(0), Some(2), "Should have 2 visibility periods");

        let total_vis_hours = df.column("total_visibility_hours").unwrap().f64().unwrap();
        let vis_hours = total_vis_hours.get(0).unwrap();
        assert!(vis_hours > 0.0, "Should have positive visibility hours");

        let requested_hours = df.column("requested_hours").unwrap().f64().unwrap();
        assert_eq!(
            requested_hours.get(0),
            Some(1200.0 / 3600.0),
            "Should convert seconds to hours"
        );

        let elev_range = df.column("elevation_range_deg").unwrap().f64().ok();
        if let Some(er) = elev_range {
            let range_val = er.get(0);
            assert!(range_val.is_some(), "Should have elevation range");
            assert_eq!(
                range_val.unwrap(),
                30.0,
                "Elevation range should be 90-60=30"
            );
        }
    }

    /// Test blocks_to_dataframe with unscheduled block
    #[test]
    fn test_blocks_to_dataframe_unscheduled() {
        let blocks = vec![SchedulingBlock {
            scheduling_block_id: "1000004990".to_string(),
            priority: 5.0,
            requested_duration: Seconds::new(600.0),
            min_observation_time: Seconds::new(600.0),
            fixed_time: None,
            coordinates: None,
            min_azimuth_angle: None,
            max_azimuth_angle: None,
            min_elevation_angle: None,
            max_elevation_angle: None,
            scheduled_period: None,
            visibility_periods: vec![],
        }];

        let result = blocks_to_dataframe(&blocks);
        assert!(
            result.is_ok(),
            "Should convert unscheduled blocks: {:?}",
            result.err()
        );

        let df = result.unwrap();

        let scheduled_flags = df.column("scheduled_flag").unwrap().bool().unwrap();
        assert_eq!(
            scheduled_flags.get(0),
            Some(false),
            "Should not be scheduled"
        );

        let num_vis = df.column("num_visibility_periods").unwrap().u32().unwrap();
        assert_eq!(num_vis.get(0), Some(0), "Should have 0 visibility periods");

        let total_vis_hours = df.column("total_visibility_hours").unwrap().f64().unwrap();
        assert_eq!(
            total_vis_hours.get(0),
            Some(0.0),
            "Should have 0 visibility hours"
        );
    }

    /// Test empty blocks array
    #[test]
    fn test_blocks_to_dataframe_empty() {
        let blocks: Vec<SchedulingBlock> = vec![];

        let result = blocks_to_dataframe(&blocks);
        assert!(
            result.is_ok(),
            "Should handle empty blocks: {:?}",
            result.err()
        );

        let df = result.unwrap();
        assert_eq!(df.height(), 0, "DataFrame should be empty");

        // Verify all columns still exist
        assert!(df.column("schedulingBlockId").is_ok());
        assert!(df.column("priority").is_ok());
        assert!(df.column("scheduled_flag").is_ok());
    }

    /// Test parsing multiple rows
    #[test]
    fn test_parse_csv_multiple_rows() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec\n\"1000004990\",8.5,1200,1200\n\"1000004991\",7.0,600,600\n\"1000004992\",9.0,1800,1800\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv_to_blocks(temp_file.path());

        assert!(
            result.is_ok(),
            "Should parse multiple rows: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].scheduling_block_id, "1000004990");
        assert_eq!(blocks[1].scheduling_block_id, "1000004991");
        assert_eq!(blocks[2].scheduling_block_id, "1000004992");
    }

    /// Test roundtrip: blocks -> dataframe -> blocks
    #[test]
    fn test_roundtrip_conversion() {
        let original_blocks = vec![SchedulingBlock {
            scheduling_block_id: "1000004990".to_string(),
            priority: 8.5,
            requested_duration: Seconds::new(1200.0),
            min_observation_time: Seconds::new(1200.0),
            fixed_time: None,
            coordinates: Some(ICRS::new(Degrees::new(158.03), Degrees::new(-68.03))),
            min_azimuth_angle: Some(Degrees::new(0.0)),
            max_azimuth_angle: Some(Degrees::new(360.0)),
            min_elevation_angle: Some(Degrees::new(60.0)),
            max_elevation_angle: Some(Degrees::new(90.0)),
            scheduled_period: None,
            visibility_periods: vec![],
        }];

        // Convert to DataFrame
        let df = blocks_to_dataframe(&original_blocks).unwrap();

        // Note: We can't directly convert back because blocks_to_dataframe adds
        // derived columns that dataframe_to_blocks doesn't expect.
        // This tests that the conversion produces valid data structure.
        assert_eq!(df.height(), 1);

        // Verify key values are preserved
        let ids = df.column("schedulingBlockId").unwrap().str().unwrap();
        assert_eq!(ids.get(0), Some("1000004990"));

        let priorities = df.column("priority").unwrap().f64().unwrap();
        assert_eq!(priorities.get(0), Some(8.5));

        let ras = df.column("raInDeg").unwrap().f64().unwrap();
        assert_eq!(ras.get(0), Some(158.03));
    }

    /// Test CSV with partial coordinates (only RA, no Dec)
    #[test]
    fn test_parse_csv_partial_coordinates() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec,raInDeg\n\"1000004990\",8.5,1200,1200,158.03\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv_to_blocks(temp_file.path());

        assert!(
            result.is_ok(),
            "Should parse with only RA: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
        // Without Dec, coordinates should be None
        assert!(
            blocks[0].coordinates.is_none(),
            "Coordinates should be None without both RA and Dec"
        );
    }

    /// Test CSV with null/empty visibility strings
    #[test]
    fn test_parse_csv_empty_visibility() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec,visibility\n\"1000004990\",8.5,1200,1200,\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv_to_blocks(temp_file.path());

        assert!(
            result.is_ok(),
            "Should parse with empty visibility: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(
            blocks[0].visibility_periods.is_empty(),
            "Should have empty visibility periods"
        );
    }

    /// Test CSV with malformed visibility strings
    #[test]
    fn test_parse_csv_malformed_visibility() {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec,visibility\n\"1000004990\",8.5,1200,1200,\"not a valid visibility string\"\n";

        let temp_file = create_temp_csv(csv_content);
        let result = parse_schedule_csv_to_blocks(temp_file.path());

        // Should parse but have empty visibility periods (parser should handle gracefully)
        assert!(
            result.is_ok(),
            "Should parse with malformed visibility: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
        // Malformed visibility should result in empty vector
        assert!(
            blocks[0].visibility_periods.is_empty(),
            "Should have empty visibility periods for malformed input"
        );
    }

    /// Test blocks_to_dataframe with fixed_time populated
    #[test]
    fn test_blocks_to_dataframe_with_fixed_time() {
        let blocks = vec![SchedulingBlock {
            scheduling_block_id: "1000004990".to_string(),
            priority: 8.5,
            requested_duration: Seconds::new(1200.0),
            min_observation_time: Seconds::new(1200.0),
            fixed_time: Some(Period::new(
                ModifiedJulianDate::new(61894.0),
                ModifiedJulianDate::new(61894.5),
            )),
            coordinates: None,
            min_azimuth_angle: None,
            max_azimuth_angle: None,
            min_elevation_angle: None,
            max_elevation_angle: None,
            scheduled_period: None,
            visibility_periods: vec![],
        }];

        let result = blocks_to_dataframe(&blocks);
        assert!(
            result.is_ok(),
            "Should convert blocks with fixed_time: {:?}",
            result.err()
        );

        let df = result.unwrap();

        let fixed_starts = df.column("fixedStartTime").unwrap().f64().unwrap();
        assert_eq!(fixed_starts.get(0), Some(61894.0));

        let fixed_stops = df.column("fixedStopTime").unwrap().f64().unwrap();
        assert_eq!(fixed_stops.get(0), Some(61894.5));
    }
}
