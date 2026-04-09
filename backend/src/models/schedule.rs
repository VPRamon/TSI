// ============================================================================
// JSON Parsing Functions
// ============================================================================
//
// These functions provide convenient file-based and string-based parsing with
// support for merging separate JSON blobs (possible_periods, dark_periods)
// when data is split across multiple files.

use crate::api;
use crate::services::visibility_service::{compute_block_visibility, VisibilityInput};
use anyhow::{Context, Result};
use std::collections::HashMap;

#[derive(serde::Deserialize)]
struct ScheduleInput {
    pub id: Option<i64>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub checksum: String,
    #[serde(default)]
    pub schedule_period: Option<api::Period>,
    #[serde(default)]
    pub dark_periods: Vec<api::Period>,
    pub geographic_location: api::GeographicLocation,
    #[serde(default)]
    pub blocks: Vec<api::SchedulingBlock>,
    #[serde(default)]
    pub possible_periods: Option<HashMap<String, Vec<api::Period>>>,
}

fn parse_input_schedule(schedule_json: &str) -> Result<ScheduleInput> {
    // TODO: original_block_id uniqueness & non empty strings
    let value: serde_json::Value =
        serde_json::from_str(schedule_json).context("Invalid schedule JSON")?;
    let has_blocks = value
        .as_object()
        .and_then(|obj| obj.get("blocks"))
        .is_some();
    if !has_blocks {
        anyhow::bail!("Missing required 'blocks' field");
    }

    serde_json::from_value(value).context("Failed to deserialize schedule JSON using Serde")
}

/// Validate schedule payload structure for the native TSI import format.
///
/// This performs structural validation only (JSON syntax + required fields + serde shape)
/// without running expensive visibility computations.
pub fn validate_schedule_json_str(schedule_json: &str) -> Result<()> {
    parse_input_schedule(schedule_json).map(|_| ())
}

/// Parse schedule from JSON string.
///
/// This function deserializes a schedule JSON string using Serde.
/// If the JSON contains an optional `possible_periods` field (map of block IDs to periods),
/// those visibility periods are merged into the corresponding blocks.
///
/// # Arguments
///
/// * `json_schedule_json` - Main schedule JSON (snake_case format matching schema)
///
/// # Returns
///
/// A fully populated `Schedule` with merged periods and computed checksum.
pub fn parse_schedule_json_str(json_schedule_json: &str) -> Result<api::Schedule> {
    let input = parse_input_schedule(json_schedule_json)?;

    let schedule_period = input
        .schedule_period
        .unwrap_or_else(|| infer_schedule_period(&input.dark_periods, &input.blocks));

    // Compute astronomical nights (location is required)
    let astronomical_nights = crate::services::astronomical_night::compute_astronomical_nights(
        &input.geographic_location,
        &schedule_period,
    );

    let mut schedule = api::Schedule {
        id: input.id,
        name: input.name,
        checksum: input.checksum,
        schedule_period,
        dark_periods: input.dark_periods,
        geographic_location: input.geographic_location,
        astronomical_nights,
        blocks: input.blocks,
    };

    // Compute checksum if not provided
    if schedule.checksum.is_empty() {
        schedule.checksum = compute_schedule_checksum(json_schedule_json);
    }

    // Hybrid visibility assignment:
    //
    // - If `possible_periods` is present in the JSON, use provided values for each block.
    //   For any block whose key is missing from the map, fall back to backend computation.
    // - If `possible_periods` is absent entirely, compute visibility in backend for all blocks.
    //
    // PERFORMANCE NOTE: For very large possible_periods maps (>100MB), this can be slow.
    // The map is deserialized entirely into memory, then cloned for each matching block.
    // Optimizations for extreme cases (not implemented yet):
    // - Use a streaming JSON parser to process blocks and periods incrementally
    // - Store large possible_periods in a separate compressed file/table
    // - Lazy-load visibility periods on demand rather than materializing all at once
    match input.possible_periods {
        Some(mut map) => {
            for block in &mut schedule.blocks {
                if let Some(periods) = map.remove(&block.original_block_id) {
                    // Provided: use as-is.
                    block.visibility_periods = periods;
                } else {
                    // Key missing from map: compute fallback for this block.
                    block.visibility_periods = compute_block_visibility(&VisibilityInput {
                        location: &schedule.geographic_location,
                        schedule_period: &schedule.schedule_period,
                        target_ra: block.target_ra,
                        target_dec: block.target_dec,
                        constraints: &block.constraints,
                        min_duration: block.min_observation,
                        astronomical_nights: Some(&schedule.astronomical_nights),
                    });
                }
            }
        }
        None => {
            // No possible_periods at all: compute visibility for every block.
            for block in &mut schedule.blocks {
                block.visibility_periods = compute_block_visibility(&VisibilityInput {
                    location: &schedule.geographic_location,
                    schedule_period: &schedule.schedule_period,
                    target_ra: block.target_ra,
                    target_dec: block.target_dec,
                    constraints: &block.constraints,
                    min_duration: block.min_observation,
                    astronomical_nights: Some(&schedule.astronomical_nights),
                });
            }
        }
    }

    Ok(schedule)
}

fn infer_schedule_period(
    dark_periods: &[api::Period],
    blocks: &[api::SchedulingBlock],
) -> api::Period {
    let mut min_start: Option<f64> = None;
    let mut max_stop: Option<f64> = None;

    let mut consider_period = |period: &api::Period| {
        let start = period.start.value();
        let stop = period.end.value();
        min_start = Some(min_start.map_or(start, |v| v.min(start)));
        max_stop = Some(max_stop.map_or(stop, |v| v.max(stop)));
    };

    for period in dark_periods {
        consider_period(period);
    }

    for block in blocks {
        for period in &block.visibility_periods {
            consider_period(period);
        }
        if let Some(period) = &block.scheduled_period {
            consider_period(period);
        }
        if let Some(period) = &block.constraints.fixed_time {
            consider_period(period);
        }
    }

    match (min_start, max_stop) {
        (Some(start), Some(stop)) => api::Period {
            start: api::ModifiedJulianDate::new(start),
            end: api::ModifiedJulianDate::new(stop),
        },
        _ => api::Period {
            start: api::ModifiedJulianDate::new(0.0),
            end: api::ModifiedJulianDate::new(0.0),
        },
    }
}

/// Compute a checksum for the schedule JSON
pub fn compute_schedule_checksum(json_str: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(json_str.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Schedule;
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

        parse_schedule_json_str(
            &std::fs::read_to_string(&schedule_path)
                .expect("Failed to read repository schedule fixture"),
        )
        .expect("Failed to parse real schedule fixture")
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

    #[test]
    fn test_parse_minimal_schedule() {
        let schedule_json = r#"{
            "geographic_location": {
                "lat_deg": 28.7624,
                "lon_deg": -17.8892,
                "height": 2396.0
            },
            "blocks": [
                {
                    "id": 1000004990,
                    "priority": 8.5,
                    "target_ra": 158.03,
                    "target_dec": -68.03,
                    "constraints": {
                        "min_alt": 60.0,
                        "max_alt": 90.0,
                        "min_az": 0.0,
                        "max_az": 360.0,
                        "fixed_time": null
                    },
                    "min_observation": 1200,
                    "requested_duration": 1200
                }
            ]
        }"#;

        let result = parse_schedule_json_str(schedule_json);
        assert!(
            result.is_ok(),
            "Should parse minimal schedule: {:?}",
            result.err()
        );

        let schedule = result.unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        assert_eq!(schedule.blocks[0].id.map(|id| id.0), Some(1000004990));
        assert_eq!(schedule.blocks[0].priority, 8.5);
    }

    #[test]
    fn test_parse_with_scheduled_period() {
        let schedule_json = r#"{
            "geographic_location": {
                "lat_deg": 28.7624,
                "lon_deg": -17.8892,
                "height": 2396.0
            },
            "blocks": [
                {
                    "id": 1000004990,
                    "priority": 8.5,
                    "scheduled_period": { "start_mjd": 61894.19429606479, "end_mjd": 61894.20818495378 },
                    "target_ra": 158.03,
                    "target_dec": -68.03,
                    "constraints": {
                        "min_alt": 60.0,
                        "max_alt": 90.0,
                        "min_az": 0.0,
                        "max_az": 360.0,
                        "fixed_time": null
                    },
                    "min_observation": 1200,
                    "requested_duration": 1200
                }
            ]
        }"#;

        let result = parse_schedule_json_str(schedule_json);
        assert!(
            result.is_ok(),
            "Should parse with scheduled period: {:?}",
            result.err()
        );

        let schedule = result.unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        assert!(schedule.blocks[0].scheduled_period.is_some());
    }

    #[test]
    fn test_parse_with_possible_periods() {
        let schedule_json = r#"{
            "geographic_location": {
                "lat_deg": 28.7624,
                "lon_deg": -17.8892,
                "height": 2396.0
            },
            "blocks": [
                {
                    "original_block_id": "1000004990",
                    "priority": 8.5,
                    "target_ra": 158.03,
                    "target_dec": -68.03,
                    "constraints": {
                        "min_alt": 60.0,
                        "max_alt": 90.0,
                        "min_az": 0.0,
                        "max_az": 360.0,
                        "fixed_time": null
                    },
                    "min_observation": 1200,
                    "requested_duration": 1200
                }
            ],
            "possible_periods": {
                "1000004990": [
                    { "start_mjd": 61771.0, "end_mjd": 61772.0 },
                    { "start_mjd": 61773.0, "end_mjd": 61774.0 }
                ]
            }
        }"#;

        let result = parse_schedule_json_str(schedule_json);
        assert!(result.is_ok(), "Should parse with possible periods");

        let schedule = result.unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        assert_eq!(schedule.blocks[0].visibility_periods.len(), 2);
    }

    #[test]
    fn test_provided_possible_periods_are_preserved() {
        // When possible_periods is present and a key matches, the provided values
        // must be used verbatim — no backend computation should replace them.
        let schedule_json = r#"{
            "geographic_location": {
                "lat_deg": 28.7624,
                "lon_deg": -17.8892,
                "height": 2396.0
            },
            "schedule_period": { "start_mjd": 60694.0, "end_mjd": 60701.0 },
            "blocks": [
                {
                    "original_block_id": "block_a",
                    "priority": 5.0,
                    "target_ra": 95.988,
                    "target_dec": -52.696,
                    "constraints": {
                        "min_alt": 0.0, "max_alt": 90.0,
                        "min_az": 0.0, "max_az": 360.0,
                        "fixed_time": null
                    },
                    "min_observation": 600,
                    "requested_duration": 600
                }
            ],
            "possible_periods": {
                "block_a": [
                    { "start_mjd": 60694.5, "end_mjd": 60694.8 },
                    { "start_mjd": 60695.5, "end_mjd": 60695.9 }
                ]
            }
        }"#;

        let schedule = parse_schedule_json_str(schedule_json).expect("Should parse");
        let vp = &schedule.blocks[0].visibility_periods;
        assert_eq!(vp.len(), 2, "Provided periods must be preserved exactly");
        assert!((vp[0].start.value() - 60694.5).abs() < 1e-9);
        assert!((vp[0].end.value() - 60694.8).abs() < 1e-9);
        assert!((vp[1].start.value() - 60695.5).abs() < 1e-9);
        assert!((vp[1].end.value() - 60695.9).abs() < 1e-9);
    }

    #[test]
    fn test_fallback_computed_when_no_possible_periods() {
        // When possible_periods is absent, backend must compute visibility.
        // A target visible from Roque over a week should produce non-empty periods.
        let schedule_json = r#"{
            "geographic_location": {
                "lat_deg": 28.7624,
                "lon_deg": -17.8892,
                "height": 2396.0
            },
            "schedule_period": { "start_mjd": 60694.0, "end_mjd": 60701.0 },
            "blocks": [
                {
                    "original_block_id": "b1",
                    "priority": 5.0,
                    "target_ra": 95.988,
                    "target_dec": -52.696,
                    "constraints": {
                        "min_alt": 0.0, "max_alt": 90.0,
                        "min_az": 0.0, "max_az": 360.0,
                        "fixed_time": null
                    },
                    "min_observation": 0,
                    "requested_duration": 600
                }
            ]
        }"#;

        let schedule = parse_schedule_json_str(schedule_json).expect("Should parse");
        assert!(
            !schedule.blocks[0].visibility_periods.is_empty(),
            "Backend should compute visibility periods when possible_periods is absent"
        );
    }

    #[test]
    fn test_partial_possible_periods_triggers_per_block_fallback() {
        // possible_periods present but missing key for block_b → block_b gets computed.
        let schedule_json = r#"{
            "geographic_location": {
                "lat_deg": 28.7624,
                "lon_deg": -17.8892,
                "height": 2396.0
            },
            "schedule_period": { "start_mjd": 60694.0, "end_mjd": 60701.0 },
            "blocks": [
                {
                    "original_block_id": "block_provided",
                    "priority": 5.0,
                    "target_ra": 95.988,
                    "target_dec": -52.696,
                    "constraints": {
                        "min_alt": 0.0, "max_alt": 90.0,
                        "min_az": 0.0, "max_az": 360.0,
                        "fixed_time": null
                    },
                    "min_observation": 0,
                    "requested_duration": 600
                },
                {
                    "original_block_id": "block_missing",
                    "priority": 3.0,
                    "target_ra": 95.988,
                    "target_dec": -52.696,
                    "constraints": {
                        "min_alt": 0.0, "max_alt": 90.0,
                        "min_az": 0.0, "max_az": 360.0,
                        "fixed_time": null
                    },
                    "min_observation": 0,
                    "requested_duration": 600
                }
            ],
            "possible_periods": {
                "block_provided": [
                    { "start_mjd": 60694.5, "end_mjd": 60694.8 }
                ]
            }
        }"#;

        let schedule = parse_schedule_json_str(schedule_json).expect("Should parse");

        // block_provided: exactly the 1 provided period
        let provided = &schedule.blocks[0].visibility_periods;
        assert_eq!(
            provided.len(),
            1,
            "Provided block must have exactly 1 period"
        );
        assert!((provided[0].start.value() - 60694.5).abs() < 1e-9);

        // block_missing: computed by backend — should have periods for this visible target
        let computed = &schedule.blocks[1].visibility_periods;
        assert!(
            !computed.is_empty(),
            "Missing-key block must have backend-computed visibility periods"
        );
    }

    #[test]
    fn test_missing_scheduling_block_key() {
        let schedule_json = r#"{"SomeOtherKey": []}"#;
        let result = parse_schedule_json_str(schedule_json);
        assert!(result.is_err(), "Should fail without SchedulingBlock key");
    }

    #[test]
    fn test_invalid_json() {
        let schedule_json = "not valid json {";
        let result = parse_schedule_json_str(schedule_json);
        assert!(result.is_err(), "Should fail with invalid JSON");
    }

    #[test]
    #[ignore = "reason: too slow for regular test runs"]
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
            first_dark.end.value(),
            61771.276910532266,
            "first dark period stop",
        );

        let block_4990 = schedule
            .blocks
            .iter()
            .find(|block| block.id.map(|id| id.0) == Some(1000004990))
            .expect("Scheduling block 1000004990 should exist");
        assert_eq!(block_4990.priority, 8.5);
        assert_eq!(block_4990.visibility_periods.len(), 120);

        let scheduled_period = block_4990
            .scheduled_period
            .clone()
            .expect("Scheduling block 1000004990 should be scheduled");
        assert_close(
            scheduled_period.start.value(),
            61894.19429606479,
            "scheduled period start",
        );
        assert_close(
            scheduled_period.end.value(),
            61894.20818495378,
            "scheduled period stop",
        );
    }

    #[test]
    #[ignore = "reason: too slow for regular test runs"]
    fn test_parse_fixed_time_block_from_real_data() {
        let schedule = parse_real_schedule_fixture();

        let block_2662 = schedule
            .blocks
            .iter()
            .find(|block| block.id.map(|id| id.0) == Some(1000002662))
            .expect("Scheduling block 1000002662 should exist");
        assert_eq!(
            block_2662.visibility_periods.len(),
            3,
            "Expected three possible periods"
        );
        // Validate constraint ranges (accept fixtures converted to snake_case).
        assert!(block_2662.constraints.min_alt.value() <= block_2662.constraints.max_alt.value());
        assert!(block_2662.constraints.min_az.value() <= block_2662.constraints.max_az.value());

        // Fixed window is optional in converted fixtures; if present, validate values.
        if let Some(fixed_window) = &block_2662.constraints.fixed_time {
            assert_close(fixed_window.start.value(), 61771.0, "fixed window start");
            assert_close(fixed_window.end.value(), 61778.0, "fixed window stop");
        }

        // Observation durations should be non-negative.
        assert!(block_2662.min_observation.value() >= 0.0);
        assert!(block_2662.requested_duration.value() >= 0.0);
    }
}
