#[cfg(test)]
mod tests {
    use crate::io::loaders::{
        DarkPeriodsLoader, ScheduleLoadResult, ScheduleLoader, ScheduleSourceType,
    };
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper to create a temp JSON file
    fn create_temp_json_file() -> NamedTempFile {
        let json_content = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": 1000004990,
                    "priority": 8.5,
                    "target": {
                        "id_": 10,
                        "name": "T32",
                        "position_": {
                            "coord": {
                                "celestial": {
                                    "raInDeg": 158.03,
                                    "decInDeg": -68.03
                                }
                            }
                        }
                    },
                    "schedulingBlockConfiguration_": {
                        "constraints_": {
                            "azimuthConstraint_": {
                                "minAzimuthAngleInDeg": 0.0,
                                "maxAzimuthAngleInDeg": 360.0
                            },
                            "elevationConstraint_": {
                                "minElevationAngleInDeg": 60.0,
                                "maxElevationAngleInDeg": 90.0
                            },
                            "timeConstraint_": {
                                "fixedStartTime": [],
                                "fixedStopTime": [],
                                "minObservationTimeInSec": 1200,
                                "requestedDurationSec": 1200
                            }
                        }
                    }
                },
                {
                    "schedulingBlockId": 1000004991,
                    "priority": 7.0,
                    "target": {
                        "position_": {
                            "coord": {
                                "celestial": {
                                    "raInDeg": 200.0,
                                    "decInDeg": 45.0
                                }
                            }
                        }
                    },
                    "schedulingBlockConfiguration_": {
                        "constraints_": {
                            "azimuthConstraint_": {
                                "minAzimuthAngleInDeg": 0.0,
                                "maxAzimuthAngleInDeg": 360.0
                            },
                            "elevationConstraint_": {
                                "minElevationAngleInDeg": 30.0,
                                "maxElevationAngleInDeg": 85.0
                            },
                            "timeConstraint_": {
                                "fixedStartTime": [],
                                "fixedStopTime": [],
                                "minObservationTimeInSec": 600,
                                "requestedDurationSec": 600
                            }
                        }
                    }
                }
            ]
        }"#;

        let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
        write!(temp_file, "{}", json_content).unwrap();
        temp_file
    }

    /// Helper to create a temp CSV file
    fn create_temp_csv_file() -> NamedTempFile {
        let csv_content = "schedulingBlockId,priority,requestedDurationSec,minObservationTimeInSec,raInDeg,decInDeg\n\"1000004990\",8.5,1200,1200,158.03,-68.03\n\"1000004991\",7.0,600,600,200.0,45.0\n";

        let mut temp_file = NamedTempFile::with_suffix(".csv").unwrap();
        write!(temp_file, "{}", csv_content).unwrap();
        temp_file
    }

    /// Test ScheduleLoadResult::new
    #[test]
    fn test_schedule_load_result_new() {
        let csv_file = create_temp_csv_file();
        let df = crate::parsing::csv_parser::parse_schedule_csv(csv_file.path()).unwrap();

        let result = ScheduleLoadResult::new(df.clone(), ScheduleSourceType::Csv);

        assert_eq!(result.source_type, ScheduleSourceType::Csv);
        assert_eq!(result.num_blocks, df.height());
        assert_eq!(result.dataframe.height(), df.height());
    }

    /// Test load_from_file with JSON extension auto-detection
    #[test]
    fn test_load_from_file_json() {
        let json_file = create_temp_json_file();
        let result = ScheduleLoader::load_from_file(json_file.path());

        assert!(result.is_ok(), "Should load JSON file: {:?}", result.err());
        let load_result = result.unwrap();
        assert_eq!(load_result.source_type, ScheduleSourceType::Json);
        assert_eq!(load_result.num_blocks, 2);
        assert_eq!(load_result.dataframe.height(), 2);
    }

    /// Test load_from_file with CSV extension auto-detection
    #[test]
    fn test_load_from_file_csv() {
        let csv_file = create_temp_csv_file();
        let result = ScheduleLoader::load_from_file(csv_file.path());

        assert!(result.is_ok(), "Should load CSV file: {:?}", result.err());
        let load_result = result.unwrap();
        assert_eq!(load_result.source_type, ScheduleSourceType::Csv);
        assert_eq!(load_result.num_blocks, 2);
        assert_eq!(load_result.dataframe.height(), 2);
    }

    /// Test load_from_file with unsupported extension
    #[test]
    fn test_load_from_file_unsupported_extension() {
        let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
        write!(temp_file, "some content").unwrap();

        let result = ScheduleLoader::load_from_file(temp_file.path());

        assert!(result.is_err(), "Should fail with unsupported extension");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Unsupported file format") || error_msg.contains("txt"),
            "Error should mention unsupported format: {}",
            error_msg
        );
    }

    /// Test load_from_file with no extension
    #[test]
    fn test_load_from_file_no_extension() {
        use std::path::PathBuf;
        let path = PathBuf::from("/tmp/file_without_extension");

        let result = ScheduleLoader::load_from_file(&path);

        assert!(result.is_err(), "Should fail with no extension");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("no extension") || error_msg.contains("extension"),
            "Error should mention missing extension: {}",
            error_msg
        );
    }

    /// Test load_from_json
    #[test]
    fn test_load_from_json() {
        let json_file = create_temp_json_file();
        let result = ScheduleLoader::load_from_json(json_file.path());

        assert!(result.is_ok(), "Should load JSON: {:?}", result.err());
        let load_result = result.unwrap();
        assert_eq!(load_result.source_type, ScheduleSourceType::Json);
        assert_eq!(load_result.num_blocks, 2);
    }

    /// Test load_from_json_str
    #[test]
    fn test_load_from_json_str() {
        let json_str = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": 1000004990,
                    "priority": 8.5,
                    "target": {
                        "position_": {
                            "coord": {
                                "celestial": {
                                    "raInDeg": 158.03,
                                    "decInDeg": -68.03
                                }
                            }
                        }
                    },
                    "schedulingBlockConfiguration_": {
                        "constraints_": {
                            "azimuthConstraint_": {
                                "minAzimuthAngleInDeg": 0.0,
                                "maxAzimuthAngleInDeg": 360.0
                            },
                            "elevationConstraint_": {
                                "minElevationAngleInDeg": 60.0,
                                "maxElevationAngleInDeg": 90.0
                            },
                            "timeConstraint_": {
                                "fixedStartTime": [],
                                "fixedStopTime": [],
                                "minObservationTimeInSec": 1200,
                                "requestedDurationSec": 1200
                            }
                        }
                    }
                }
            ]
        }"#;

        let result = ScheduleLoader::load_from_json_str(json_str);

        assert!(
            result.is_ok(),
            "Should load from JSON string: {:?}",
            result.err()
        );
        let load_result = result.unwrap();
        assert_eq!(load_result.source_type, ScheduleSourceType::Json);
        assert_eq!(load_result.num_blocks, 1);
    }

    /// Test load_from_csv
    #[test]
    fn test_load_from_csv() {
        let csv_file = create_temp_csv_file();
        let result = ScheduleLoader::load_from_csv(csv_file.path());

        assert!(result.is_ok(), "Should load CSV: {:?}", result.err());
        let load_result = result.unwrap();
        assert_eq!(load_result.source_type, ScheduleSourceType::Csv);
        assert_eq!(load_result.num_blocks, 2);
    }

    /// Test load_blocks_from_csv
    #[test]
    fn test_load_blocks_from_csv() {
        let csv_file = create_temp_csv_file();
        let result = ScheduleLoader::load_blocks_from_csv(csv_file.path());

        assert!(
            result.is_ok(),
            "Should load blocks from CSV: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].scheduling_block_id, "1000004990");
        assert_eq!(blocks[1].scheduling_block_id, "1000004991");
    }

    /// Test load_blocks_from_json
    #[test]
    fn test_load_blocks_from_json() {
        let json_file = create_temp_json_file();
        let result = ScheduleLoader::load_blocks_from_json(json_file.path());

        assert!(
            result.is_ok(),
            "Should load blocks from JSON: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].scheduling_block_id, "1000004990");
        assert_eq!(blocks[1].scheduling_block_id, "1000004991");
    }

    /// Test that num_blocks matches DataFrame height
    #[test]
    fn test_num_blocks_matches_df_height() {
        let json_file = create_temp_json_file();
        let result = ScheduleLoader::load_from_file(json_file.path()).unwrap();

        assert_eq!(result.num_blocks, result.dataframe.height());
    }

    /// Test DarkPeriodsLoader::load_from_file
    #[test]
    fn test_dark_periods_load_from_file() {
        let json_content = r#"{
            "dark_periods": [
                {
                    "start": 61771.0,
                    "stop": 61771.5
                },
                {
                    "start": 61772.0,
                    "stop": 61772.5
                }
            ]
        }"#;

        let mut temp_file = NamedTempFile::with_suffix(".json").unwrap();
        write!(temp_file, "{}", json_content).unwrap();

        let result = DarkPeriodsLoader::load_from_file(temp_file.path());

        assert!(
            result.is_ok(),
            "Should load dark periods from file: {:?}",
            result.err()
        );
        let df = result.unwrap();
        assert_eq!(df.height(), 2);

        // Verify columns exist
        assert!(df.column("start_dt").is_ok());
        assert!(df.column("stop_dt").is_ok());
        assert!(df.column("start_mjd").is_ok());
        assert!(df.column("stop_mjd").is_ok());
        assert!(df.column("duration_hours").is_ok());
        assert!(df.column("months").is_ok());
    }

    /// Test DarkPeriodsLoader::load_from_str
    #[test]
    fn test_dark_periods_load_from_str() {
        let json_str = r#"{
            "dark_periods": [
                {
                    "start": 61771.0,
                    "stop": 61771.5
                },
                {
                    "start": 61802.0,
                    "stop": 61833.0
                }
            ]
        }"#;

        let result = DarkPeriodsLoader::load_from_str(json_str);

        assert!(
            result.is_ok(),
            "Should load dark periods from string: {:?}",
            result.err()
        );
        let df = result.unwrap();
        assert_eq!(df.height(), 2);

        // Verify months column exists (as mentioned in docstring)
        assert!(
            df.column("months").is_ok(),
            "Should have months list column"
        );

        // Verify it's a list type
        let months_col = df.column("months").unwrap();
        assert!(
            matches!(months_col.dtype(), polars::prelude::DataType::List(_)),
            "Months should be List type"
        );
    }

    /// Test DarkPeriodsLoader with empty input
    #[test]
    fn test_dark_periods_load_from_str_empty() {
        let json_str = r#"{
            "dark_periods": []
        }"#;

        let result = DarkPeriodsLoader::load_from_str(json_str);

        assert!(
            result.is_ok(),
            "Should handle empty periods: {:?}",
            result.err()
        );
        let df = result.unwrap();
        assert_eq!(df.height(), 0);

        // Verify schema still exists
        assert!(df.column("start_dt").is_ok());
        assert!(df.column("months").is_ok());
    }

    /// Test error propagation for malformed JSON
    #[test]
    fn test_load_from_json_str_malformed() {
        let json_str = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": 1000004990,
                    "priority": "not_a_number"
                }
            ]
        }"#;

        let result = ScheduleLoader::load_from_json_str(json_str);

        assert!(result.is_err(), "Should fail with malformed JSON");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to parse")
                || error_msg.contains("convert")
                || error_msg.contains("index"),
            "Error should mention parse failure: {}",
            error_msg
        );
    }

    /// Test error propagation for missing SchedulingBlock key
    #[test]
    fn test_load_from_json_str_missing_key() {
        let json_str = r#"{
            "SomeOtherKey": []
        }"#;

        let result = ScheduleLoader::load_from_json_str(json_str);

        assert!(result.is_err(), "Should fail without SchedulingBlock key");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("SchedulingBlock") || error_msg.contains("Failed to parse"),
            "Error should mention missing key: {}",
            error_msg
        );
    }

    /// Test that DataFrame contains expected columns from JSON
    #[test]
    fn test_json_dataframe_columns() {
        let json_file = create_temp_json_file();
        let result = ScheduleLoader::load_from_json(json_file.path()).unwrap();

        let df = result.dataframe;
        let col_names = df.get_column_names();

        // Check required columns
        assert!(col_names.iter().any(|&name| name == "schedulingBlockId"));
        assert!(col_names.iter().any(|&name| name == "priority"));
        assert!(col_names.iter().any(|&name| name == "requestedDurationSec"));
        assert!(col_names.iter().any(|&name| name == "scheduled_flag"));
        assert!(col_names.iter().any(|&name| name == "priority_bin"));
        assert!(col_names
            .iter()
            .any(|&name| name == "total_visibility_hours"));
    }

    /// Test that DataFrame contains expected columns from CSV
    #[test]
    fn test_csv_dataframe_columns() {
        let csv_file = create_temp_csv_file();
        let result = ScheduleLoader::load_from_csv(csv_file.path()).unwrap();

        let df = result.dataframe;
        let col_names = df.get_column_names();

        // Check required columns
        assert!(col_names.iter().any(|&name| name == "schedulingBlockId"));
        assert!(col_names.iter().any(|&name| name == "priority"));
        assert!(col_names.iter().any(|&name| name == "requestedDurationSec"));
    }

    /// Test DarkPeriodsLoader error handling for nonexistent file
    #[test]
    fn test_dark_periods_nonexistent_file() {
        use std::path::Path;
        let result = DarkPeriodsLoader::load_from_file(Path::new("/nonexistent/file.json"));

        assert!(result.is_err(), "Should fail for nonexistent file");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to parse") || error_msg.contains("Failed to read"),
            "Error should mention file issue: {}",
            error_msg
        );
    }

    /// Test DarkPeriodsLoader error handling for invalid JSON
    #[test]
    fn test_dark_periods_invalid_json() {
        let json_str = r#"{
            "dark_periods": [
                {
                    "start": 61771.0,
                    // Invalid comment
                }
            ]
        }"#;

        let result = DarkPeriodsLoader::load_from_str(json_str);

        assert!(result.is_err(), "Should fail with invalid JSON");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to parse") || error_msg.contains("Failed to convert"),
            "Error should mention parse failure: {}",
            error_msg
        );
    }

    /// Test ScheduleSourceType equality
    #[test]
    fn test_schedule_source_type_equality() {
        assert_eq!(ScheduleSourceType::Json, ScheduleSourceType::Json);
        assert_eq!(ScheduleSourceType::Csv, ScheduleSourceType::Csv);
        assert_ne!(ScheduleSourceType::Json, ScheduleSourceType::Csv);
    }

    /// Test case-insensitive extension detection
    #[test]
    fn test_case_insensitive_extension() {
        let mut temp_file = NamedTempFile::with_suffix(".JSON").unwrap();
        write!(temp_file, "{{\n\"SchedulingBlock\": [{{\n\"schedulingBlockId\": 1,\n\"priority\": 8.5,\n\"target\": {{\"position_\": {{\"coord\": {{\"celestial\": {{\"raInDeg\": 158.03, \"decInDeg\": -68.03}}}}}}}},\n\"schedulingBlockConfiguration_\": {{\"constraints_\": {{\"azimuthConstraint_\": {{\"minAzimuthAngleInDeg\": 0.0, \"maxAzimuthAngleInDeg\": 360.0}}, \"elevationConstraint_\": {{\"minElevationAngleInDeg\": 60.0, \"maxElevationAngleInDeg\": 90.0}}, \"timeConstraint_\": {{\"fixedStartTime\": [], \"fixedStopTime\": [], \"minObservationTimeInSec\": 1200, \"requestedDurationSec\": 1200}}}}}}}}\n]}}\n").unwrap();

        let result = ScheduleLoader::load_from_file(temp_file.path());

        assert!(
            result.is_ok(),
            "Should handle uppercase .JSON extension: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().source_type, ScheduleSourceType::Json);
    }
}
