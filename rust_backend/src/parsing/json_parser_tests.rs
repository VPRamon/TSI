#[cfg(test)]
mod tests {
    use crate::db::models::Schedule;
    use crate::parsing::json_parser::*;
    use std::path::PathBuf;

    const DATA_DIR: &str = "../data";
    const EXPECTED_SCHEDULE_CHECKSUM: &str =
        "0c06e8a8ea614fb6393b7549f98abf973941f54012ac47a309a9d5a99876233a";

    fn repo_data_path(file_name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(DATA_DIR)
            .join(file_name)
    }

    fn parse_real_schedule_fixture() -> Schedule {
        let schedule_path = repo_data_path("schedule.json");
        let possible_path = repo_data_path("possible_periods.json");
        let dark_path = repo_data_path("dark_periods.json");

        parse_schedule_json(&schedule_path, Some(&possible_path), &dark_path)
            .expect("Failed to parse repository schedule fixtures")
    }

    fn assert_close(value: f64, expected: f64, label: &str) {
        let diff = (value - expected).abs();
        assert!(
            diff < 1e-9,
            "Mismatch for {}: expected {}, got {}",
            label,
            expected,
            value
        );
    }

    /// Test basic parsing with minimal schedule
    #[test]
    fn test_parse_minimal_schedule() {
        let schedule_json = r#"{
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
                                    "decInDeg": -68.03,
                                    "raProperMotionInMarcsecYear": 0.0,
                                    "decProperMotionInMarcsecYear": 0.0,
                                    "equinox": 2000.0
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

        let dark_periods_json = r#"{
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
                        "value": 61772.0
                    }
                }
            ]
        }"#;

        let result = parse_schedule_json_str(schedule_json, None, dark_periods_json);
        assert!(
            result.is_ok(),
            "Should parse minimal schedule: {:?}",
            result.err()
        );

        let schedule = result.unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        assert_eq!(schedule.dark_periods.len(), 1);
        assert_eq!(schedule.blocks[0].id.0, 1000004990);
        assert_eq!(schedule.blocks[0].priority, 8.5);
    }

    /// Test parsing with scheduled period
    #[test]
    fn test_parse_with_scheduled_period() {
        let schedule_json = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": 1000004990,
                    "priority": 8.5,
                    "scheduled_period": {
                        "durationInSec": 1200.0,
                        "startTime": {
                            "format": "MJD",
                            "scale": "UTC",
                            "value": 61894.19429606479
                        },
                        "stopTime": {
                            "format": "MJD",
                            "scale": "UTC",
                            "value": 61894.20818495378
                        }
                    },
                    "target": {
                        "id_": 10,
                        "name": "T32",
                        "position_": {
                            "coord": {
                                "celestial": {
                                    "raInDeg": 158.03,
                                    "decInDeg": -68.03,
                                    "raProperMotionInMarcsecYear": 0.0,
                                    "decProperMotionInMarcsecYear": 0.0,
                                    "equinox": 2000.0
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

        let dark_periods_json = r#"{
            "dark_periods": []
        }"#;

        let result = parse_schedule_json_str(schedule_json, None, dark_periods_json);
        assert!(result.is_ok(), "Should parse with scheduled period");

        let schedule = result.unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        assert!(schedule.blocks[0].scheduled_period.is_some());
    }

    /// Test parsing with possible periods
    #[test]
    fn test_parse_with_possible_periods() {
        let schedule_json = r#"{
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
                                    "decInDeg": -68.03,
                                    "raProperMotionInMarcsecYear": 0.0,
                                    "decProperMotionInMarcsecYear": 0.0,
                                    "equinox": 2000.0
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

        let possible_periods_json = r#"{
            "SchedulingBlock": {
                "1000004990": [
                    {
                        "startTime": {
                            "format": "MJD",
                            "scale": "UTC",
                            "value": 61771.0
                        },
                        "stopTime": {
                            "format": "MJD",
                            "scale": "UTC",
                            "value": 61772.0
                        }
                    },
                    {
                        "startTime": {
                            "format": "MJD",
                            "scale": "UTC",
                            "value": 61773.0
                        },
                        "stopTime": {
                            "format": "MJD",
                            "scale": "UTC",
                            "value": 61774.0
                        }
                    }
                ]
            }
        }"#;

        let dark_periods_json = r#"{
            "dark_periods": []
        }"#;

        let result = parse_schedule_json_str(
            schedule_json,
            Some(possible_periods_json),
            dark_periods_json,
        );
        assert!(result.is_ok(), "Should parse with possible periods");

        let schedule = result.unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        assert_eq!(schedule.blocks[0].visibility_periods.len(), 2);
    }

    /// Test error handling for missing required fields
    #[test]
    fn test_missing_scheduling_block_key() {
        let schedule_json = r#"{
            "SomeOtherKey": []
        }"#;

        let dark_periods_json = r#"{
            "dark_periods": []
        }"#;

        let result = parse_schedule_json_str(schedule_json, None, dark_periods_json);
        assert!(result.is_err(), "Should fail without SchedulingBlock key");
    }

    /// Test error handling for invalid JSON
    #[test]
    fn test_invalid_json() {
        let schedule_json = "not valid json {";
        let dark_periods_json = r#"{
            "dark_periods": []
        }"#;

        let result = parse_schedule_json_str(schedule_json, None, dark_periods_json);
        assert!(result.is_err(), "Should fail with invalid JSON");
    }

    /// Integration test using repository schedule fixtures to ensure we parse end-to-end data.
    #[test]
    fn test_parse_real_schedule_files() {
        let schedule = parse_real_schedule_fixture();

        assert_eq!(schedule.blocks.len(), 2647, "Unexpected block count");
        assert_eq!(
            schedule.dark_periods.len(),
            314,
            "Unexpected dark period count"
        );
        assert_eq!(schedule.name, "parsed_schedule");
        assert_eq!(schedule.checksum, EXPECTED_SCHEDULE_CHECKSUM);

        let first_dark = schedule
            .dark_periods
            .first()
            .expect("Dark periods should not be empty");
        assert_close(first_dark.start.value(), 61771.0, "first dark period start");
        assert_close(
            first_dark.stop.value(),
            61771.276910532266,
            "first dark period stop",
        );

        let block_4990 = schedule
            .blocks
            .iter()
            .find(|block| block.id.0 == 1000004990)
            .expect("Scheduling block 1000004990 should exist");
        assert_eq!(block_4990.priority, 8.5);
        assert_eq!(block_4990.visibility_periods.len(), 120);

        let scheduled_period = block_4990
            .scheduled_period
            .expect("Scheduling block 1000004990 should be scheduled");
        assert_close(
            scheduled_period.start.value(),
            61894.19429606479,
            "scheduled period start",
        );
        assert_close(
            scheduled_period.stop.value(),
            61894.20818495378,
            "scheduled period stop",
        );
    }

    /// Ensure we correctly capture fixed-time constraints and visibility periods from real data.
    #[test]
    fn test_parse_fixed_time_block_from_real_data() {
        let schedule = parse_real_schedule_fixture();

        let block_2662 = schedule
            .blocks
            .iter()
            .find(|block| block.id.0 == 1000002662)
            .expect("Scheduling block 1000002662 should exist");
        assert_eq!(
            block_2662.visibility_periods.len(),
            3,
            "Expected three possible periods"
        );
        assert_close(
            block_2662.constraints.min_alt.value(),
            45.0,
            "minimum altitude",
        );
        assert_close(
            block_2662.constraints.max_alt.value(),
            90.0,
            "maximum altitude",
        );
        assert_close(
            block_2662.constraints.min_az.value(),
            0.0,
            "minimum azimuth",
        );
        assert_close(
            block_2662.constraints.max_az.value(),
            360.0,
            "maximum azimuth",
        );

        let fixed_window = block_2662
            .constraints
            .fixed_time
            .expect("Block 1000002662 should have a fixed time window");
        assert_close(fixed_window.start.value(), 61771.0, "fixed window start");
        assert_close(fixed_window.stop.value(), 61778.0, "fixed window stop");

        assert_close(
            block_2662.min_observation.value(),
            1800.0,
            "min observation seconds",
        );
        assert_close(
            block_2662.requested_duration.value(),
            1800.0,
            "requested duration seconds",
        );
    }
}
