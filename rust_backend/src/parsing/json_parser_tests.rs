#[cfg(test)]
mod tests {
    use crate::parsing::json_parser::{parse_schedule_json, parse_schedule_json_str};
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Test parsing JSON with string IDs
    #[test]
    fn test_parse_string_ids() {
        let json = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": "1000004990",
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
                }
            ]
        }"#;

        let result = parse_schedule_json_str(json);
        assert!(
            result.is_ok(),
            "Should parse string IDs: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].scheduling_block_id, "1000004990");
    }

    /// Test parsing JSON with integer IDs
    #[test]
    fn test_parse_integer_ids() {
        let json = r#"{
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
                }
            ]
        }"#;

        let result = parse_schedule_json_str(json);
        assert!(
            result.is_ok(),
            "Should parse integer IDs: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].scheduling_block_id, "1000004990");
    }

    /// Test parsing JSON without SchedulingBlock key
    #[test]
    fn test_missing_scheduling_block_key() {
        let json = r#"{
            "SomeOtherKey": []
        }"#;

        let result = parse_schedule_json_str(json);
        assert!(result.is_err(), "Should fail without SchedulingBlock key");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("SchedulingBlock"),
            "Error should mention missing key: {}",
            error_msg
        );
    }

    /// Test parsing JSON with missing required fields
    #[test]
    fn test_missing_priority() {
        let json = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": 1000004990,
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
                }
            ]
        }"#;

        let result = parse_schedule_json_str(json);
        assert!(result.is_err(), "Should fail with missing priority");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("priority") || error_msg.contains("missing field"),
            "Error should mention missing field: {}",
            error_msg
        );
    }

    /// Test parsing JSON with malformed constraints
    #[test]
    fn test_malformed_azimuth_constraint() {
        let json = r#"{
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
                                "minAzimuthAngleInDeg": "not_a_number"
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

        let result = parse_schedule_json_str(json);
        assert!(result.is_err(), "Should fail with malformed constraint");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("index 0") || error_msg.contains("SchedulingBlock"),
            "Error should include context about which block failed: {}",
            error_msg
        );
    }

    /// Test parsing JSON with fixed time windows
    #[test]
    fn test_fixed_time_windows() {
        let json = r#"{
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
                                "fixedStartTime": [{"value": 61894.0}],
                                "fixedStopTime": [{"value": 61894.5}],
                                "minObservationTimeInSec": 1200,
                                "requestedDurationSec": 1200
                            }
                        }
                    }
                }
            ]
        }"#;

        let result = parse_schedule_json_str(json);
        assert!(
            result.is_ok(),
            "Should parse fixed time windows: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
        assert!(
            blocks[0].fixed_time.is_some(),
            "Should have fixed_time populated"
        );
        let fixed_time = blocks[0].fixed_time.as_ref().unwrap();
        assert_eq!(fixed_time.start.value(), 61894.0);
        assert_eq!(fixed_time.stop.value(), 61894.5);
    }

    /// Test parsing JSON with scheduled period
    #[test]
    fn test_scheduled_period() {
        let json = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": 1000004990,
                    "priority": 8.5,
                    "scheduled_period": {
                        "startTime": {"value": 61894.19},
                        "stopTime": {"value": 61894.21}
                    },
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
                }
            ]
        }"#;

        let result = parse_schedule_json_str(json);
        assert!(
            result.is_ok(),
            "Should parse scheduled period: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert!(
            blocks[0].scheduled_period.is_some(),
            "Should have scheduled_period"
        );
        assert!(blocks[0].is_scheduled(), "Should be marked as scheduled");
    }

    /// Test parsing invalid JSON syntax
    #[test]
    fn test_invalid_json_syntax() {
        let json = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": 1000004990,
                    "priority": 8.5,
                    // This is invalid JSON (no comments allowed)
                }
            ]
        }"#;

        let result = parse_schedule_json_str(json);
        assert!(result.is_err(), "Should fail with invalid JSON syntax");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Invalid JSON") || error_msg.contains("syntax"),
            "Error should mention JSON syntax error: {}",
            error_msg
        );
    }

    /// Test parsing JSON file (integration with file system)
    #[test]
    fn test_parse_schedule_json_file() {
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
                }
            ]
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", json_content).unwrap();

        let result = parse_schedule_json(temp_file.path());
        assert!(
            result.is_ok(),
            "Should parse JSON from file: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 1);
    }

    /// Test parsing JSON file that doesn't exist
    #[test]
    fn test_parse_nonexistent_file() {
        let result = parse_schedule_json(std::path::Path::new("/nonexistent/file.json"));
        assert!(result.is_err(), "Should fail for nonexistent file");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Failed to read") || error_msg.contains("No such file"),
            "Error should mention file read failure: {}",
            error_msg
        );
    }

    /// Test parsing multiple blocks
    #[test]
    fn test_parse_multiple_blocks() {
        let json = r#"{
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

        let result = parse_schedule_json_str(json);
        assert!(
            result.is_ok(),
            "Should parse multiple blocks: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].scheduling_block_id, "1000004990");
        assert_eq!(blocks[1].scheduling_block_id, "1000004991");
        assert_eq!(blocks[0].priority, 8.5);
        assert_eq!(blocks[1].priority, 7.0);
    }

    /// Test empty SchedulingBlock array
    #[test]
    fn test_empty_scheduling_blocks() {
        let json = r#"{
            "SchedulingBlock": []
        }"#;

        let result = parse_schedule_json_str(json);
        assert!(
            result.is_ok(),
            "Should parse empty array: {:?}",
            result.err()
        );
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 0);
    }

    /// Test missing nested constraint fields
    #[test]
    fn test_missing_time_constraint() {
        let json = r#"{
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
                            }
                        }
                    }
                }
            ]
        }"#;

        let result = parse_schedule_json_str(json);
        assert!(result.is_err(), "Should fail with missing timeConstraint_");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("timeConstraint") || error_msg.contains("missing field"),
            "Error should mention missing constraint: {}",
            error_msg
        );
    }

    /// Test that error context includes block index
    #[test]
    fn test_error_context_includes_block_index() {
        let json = r#"{
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
                },
                {
                    "schedulingBlockId": 1000004991,
                    "priority": "invalid_priority",
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

        let result = parse_schedule_json_str(json);
        assert!(
            result.is_err(),
            "Should fail with invalid priority in second block"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("index 1") || error_msg.contains("SchedulingBlock"),
            "Error should indicate which block failed: {}",
            error_msg
        );
    }
}
