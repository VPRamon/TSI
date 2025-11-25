#[cfg(test)]
mod tests {
    use crate::parsing::dark_periods_parser::{
        parse_dark_periods_file, parse_dark_periods_str, periods_to_dataframe,
    };
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Test parsing flat array format
    #[test]
    fn test_parse_flat_array() {
        let json = r#"[
            {
                "start": 61771.0,
                "stop": 61771.5
            },
            {
                "start": 61772.0,
                "stop": 61772.5
            }
        ]"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse flat array: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 2);
        assert_eq!(periods[0].start.value(), 61771.0);
        assert_eq!(periods[1].start.value(), 61772.0);
    }

    /// Test parsing nested object format with dark_periods key
    #[test]
    fn test_parse_nested_dark_periods() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": 61771.0,
                    "stop": 61771.5
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse nested format: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
    }

    /// Test parsing with camelCase keys (darkPeriods)
    #[test]
    fn test_parse_camel_case_key() {
        let json = r#"{
            "darkPeriods": [
                {
                    "startTime": 61771.0,
                    "stopTime": 61771.5
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(result.is_ok(), "Should parse camelCase: {:?}", result.err());
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
    }

    /// Test parsing with snake_case keys
    #[test]
    fn test_parse_snake_case_keys() {
        let json = r#"{
            "dark_periods": [
                {
                    "start_mjd": 61771.0,
                    "stop_mjd": 61771.5
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse snake_case: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
    }

    /// Test parsing with nested value objects
    #[test]
    fn test_parse_nested_value_objects() {
        let json = r#"{
            "dark_periods": [
                {
                    "startTime": {
                        "format": "MJD",
                        "scale": "UTC",
                        "value": 61771.0
                    },
                    "stopTime": {
                        "format": "MJD",
                        "scale": "UTC",
                        "value": 61771.276910532266
                    }
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse nested value objects: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
        assert_eq!(periods[0].start.value(), 61771.0);
        assert_eq!(periods[0].stop.value(), 61771.276910532266);
    }

    /// Test parsing with ISO timestamp strings
    #[test]
    fn test_parse_iso_timestamps() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": "2028-01-01T00:00:00Z",
                    "stop": "2028-01-01T12:00:00Z"
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse ISO timestamps: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
        // Verify it's a valid MJD value
        assert!(periods[0].start.value() > 60000.0 && periods[0].start.value() < 70000.0);
    }

    /// Test parsing with array format [start, stop]
    #[test]
    fn test_parse_array_format() {
        let json = r#"{
            "dark_periods": [
                [61771.0, 61771.5],
                [61772.0, 61772.5]
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse array format: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 2);
        assert_eq!(periods[0].start.value(), 61771.0);
        assert_eq!(periods[0].stop.value(), 61771.5);
    }

    /// Test rejecting reversed start/stop (stop before start)
    #[test]
    fn test_reject_reversed_period() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": 61771.5,
                    "stop": 61771.0
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse but filter out reversed: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        // Reversed period should be filtered out
        assert_eq!(periods.len(), 0, "Reversed periods should be rejected");
    }

    /// Test parsing empty array
    #[test]
    fn test_parse_empty_array() {
        let json = r#"{
            "dark_periods": []
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse empty array: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 0);
    }

    /// Test parsing with mixed valid and invalid periods
    #[test]
    fn test_parse_mixed_periods() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": 61771.0,
                    "stop": 61771.5
                },
                {
                    "start": 61772.5,
                    "stop": 61772.0
                },
                {
                    "start": 61773.0,
                    "stop": 61773.5
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse mixed periods: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        // Only valid periods (1st and 3rd) should be included
        assert_eq!(periods.len(), 2);
        assert_eq!(periods[0].start.value(), 61771.0);
        assert_eq!(periods[1].start.value(), 61773.0);
    }

    /// Test parsing from file
    #[test]
    fn test_parse_dark_periods_file() {
        let json_content = r#"{
            "dark_periods": [
                {
                    "start": 61771.0,
                    "stop": 61771.5
                }
            ]
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_content).unwrap();

        let result = parse_dark_periods_file(temp_file.path());
        assert!(result.is_ok(), "Should parse from file: {:?}", result.err());
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
    }

    /// Test parsing nonexistent file
    #[test]
    fn test_parse_nonexistent_file() {
        let result = parse_dark_periods_file(std::path::Path::new("/nonexistent/file.json"));
        assert!(result.is_err(), "Should fail for nonexistent file");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to read") || error_msg.contains("No such file"));
    }

    /// Test parsing invalid JSON
    #[test]
    fn test_parse_invalid_json() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": 61771.0,
                    // Invalid comment
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(result.is_err(), "Should fail with invalid JSON");
    }

    /// Test periods_to_dataframe with valid periods
    #[test]
    fn test_periods_to_dataframe() {
        let json = r#"{
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

        let periods = parse_dark_periods_str(json).unwrap();
        let result = periods_to_dataframe(periods);

        assert!(
            result.is_ok(),
            "Should convert to DataFrame: {:?}",
            result.err()
        );
        let df = result.unwrap();

        assert_eq!(df.height(), 2);

        // Check columns exist
        assert!(df.column("start_dt").is_ok(), "Should have start_dt column");
        assert!(df.column("stop_dt").is_ok(), "Should have stop_dt column");
        assert!(
            df.column("start_mjd").is_ok(),
            "Should have start_mjd column"
        );
        assert!(df.column("stop_mjd").is_ok(), "Should have stop_mjd column");
        assert!(
            df.column("duration_hours").is_ok(),
            "Should have duration_hours column"
        );
        assert!(df.column("months").is_ok(), "Should have months column");

        // Check MJD values
        let start_mjds = df.column("start_mjd").unwrap().f64().unwrap();
        assert_eq!(start_mjds.get(0), Some(61771.0));
        assert_eq!(start_mjds.get(1), Some(61802.0));

        // Check duration is calculated
        let durations = df.column("duration_hours").unwrap().f64().unwrap();
        let duration_0 = durations.get(0).unwrap();
        assert!(duration_0 > 0.0, "Duration should be positive");
        assert_eq!(duration_0, (61771.5 - 61771.0) * 24.0);
    }

    /// Test periods_to_dataframe with empty input
    #[test]
    fn test_periods_to_dataframe_empty() {
        let periods = vec![];
        let result = periods_to_dataframe(periods);

        assert!(
            result.is_ok(),
            "Should handle empty periods: {:?}",
            result.err()
        );
        let df = result.unwrap();

        assert_eq!(df.height(), 0, "DataFrame should be empty");

        // Verify schema exists with UTC timezone
        assert!(df.column("start_dt").is_ok(), "Should have start_dt column");
        assert!(df.column("stop_dt").is_ok(), "Should have stop_dt column");

        // Verify datetime columns have UTC timezone
        let start_dt = df.column("start_dt").unwrap();
        match start_dt.dtype() {
            polars::prelude::DataType::Datetime(_, tz) => {
                assert!(tz.is_some(), "Datetime should have timezone");
                if let Some(tz_name) = tz {
                    assert_eq!(tz_name.as_str(), "UTC", "Timezone should be UTC");
                }
            }
            _ => panic!("start_dt should be Datetime type"),
        }
    }

    /// Test periods_to_dataframe months column
    #[test]
    fn test_periods_to_dataframe_months() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": 61771.0,
                    "stop": 61802.0
                }
            ]
        }"#;

        let periods = parse_dark_periods_str(json).unwrap();
        let result = periods_to_dataframe(periods);

        assert!(
            result.is_ok(),
            "Should convert to DataFrame: {:?}",
            result.err()
        );
        let df = result.unwrap();

        // Verify months column exists and is a list type
        let months_col = df.column("months");
        assert!(months_col.is_ok(), "Should have months column");

        // The months column should be a list of strings
        let months = months_col.unwrap();
        assert!(
            matches!(months.dtype(), polars::prelude::DataType::List(_)),
            "Months should be List type"
        );
    }

    /// Test parsing with alternative key names (periods)
    #[test]
    fn test_parse_periods_key() {
        let json = r#"{
            "periods": [
                {
                    "start": 61771.0,
                    "stop": 61771.5
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse with 'periods' key: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
    }

    /// Test parsing with DarkPeriods (capitalized)
    #[test]
    fn test_parse_capitalized_key() {
        let json = r#"{
            "DarkPeriods": [
                {
                    "start": 61771.0,
                    "stop": 61771.5
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse with capitalized key: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
    }

    /// Test parsing with end/endTime instead of stop
    #[test]
    fn test_parse_end_time_keys() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": 61771.0,
                    "end": 61771.5
                },
                {
                    "startTime": 61772.0,
                    "endTime": 61772.5
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse with end/endTime: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 2);
    }

    /// Test parsing with string MJD values
    #[test]
    fn test_parse_string_mjd() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": "61771.0",
                    "stop": "61771.5"
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse string MJD: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
        assert_eq!(periods[0].start.value(), 61771.0);
    }

    /// Test parsing with integer MJD values
    #[test]
    fn test_parse_integer_mjd() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": 61771,
                    "stop": 61772
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse integer MJD: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
        assert_eq!(periods[0].start.value(), 61771.0);
    }

    /// Test parsing with stopUTC/startUTC variations
    #[test]
    fn test_parse_utc_suffix_keys() {
        let json = r#"{
            "dark_periods": [
                {
                    "startUTC": 61771.0,
                    "stopUTC": 61771.5
                },
                {
                    "startUtc": 61772.0,
                    "stopUtc": 61772.5
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse UTC suffix keys: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 2);
    }

    /// Test parsing with naive datetime format (pandas-compatible)
    #[test]
    fn test_parse_naive_datetime() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": "2028-01-01 00:00:00",
                    "stop": "2028-01-01 12:00:00"
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse naive datetime: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
    }

    /// Test parsing with incomplete array (missing stop)
    #[test]
    fn test_parse_incomplete_array() {
        let json = r#"{
            "dark_periods": [
                [61771.0]
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse but skip incomplete: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        // Incomplete array should be skipped
        assert_eq!(periods.len(), 0);
    }

    /// Test parsing with missing stop key in object
    #[test]
    fn test_parse_missing_stop_key() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": 61771.0
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse but skip incomplete: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        // Period without stop should be skipped
        assert_eq!(periods.len(), 0);
    }

    /// Test parsing with mjd nested key
    #[test]
    fn test_parse_mjd_nested_key() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": {"mjd": 61771.0},
                    "stop": {"MJD": 61771.5}
                }
            ]
        }"#;

        let result = parse_dark_periods_str(json);
        assert!(
            result.is_ok(),
            "Should parse mjd nested key: {:?}",
            result.err()
        );
        let periods = result.unwrap();
        assert_eq!(periods.len(), 1);
    }

    /// Test that datetime columns in DataFrame have correct timezone
    #[test]
    fn test_dataframe_datetime_timezone() {
        let json = r#"{
            "dark_periods": [
                {
                    "start": 61771.0,
                    "stop": 61771.5
                }
            ]
        }"#;

        let periods = parse_dark_periods_str(json).unwrap();
        let df = periods_to_dataframe(periods).unwrap();

        // Check start_dt has UTC timezone
        let start_dt = df.column("start_dt").unwrap();
        match start_dt.dtype() {
            polars::prelude::DataType::Datetime(_, tz) => {
                assert!(tz.is_some(), "start_dt should have timezone");
                assert_eq!(tz.as_ref().unwrap().as_str(), "UTC");
            }
            _ => panic!("start_dt should be Datetime type"),
        }

        // Check stop_dt has UTC timezone
        let stop_dt = df.column("stop_dt").unwrap();
        match stop_dt.dtype() {
            polars::prelude::DataType::Datetime(_, tz) => {
                assert!(tz.is_some(), "stop_dt should have timezone");
                assert_eq!(tz.as_ref().unwrap().as_str(), "UTC");
            }
            _ => panic!("stop_dt should be Datetime type"),
        }
    }
}
