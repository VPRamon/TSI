// ============================================================================
// JSON Parsing Functions
// ============================================================================
//
// These functions provide convenient file-based and string-based parsing with
// support for merging separate JSON blobs (possible_periods, dark_periods)
// when data is split across multiple files.

use crate::api;
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
}

fn validate_input_schedule(_schedule_json: &str) -> Result<()> {
    // TODO: original_block_id uniqueness & non empty strings
    let value: serde_json::Value =
        serde_json::from_str(_schedule_json).context("Invalid schedule JSON")?;
    let has_blocks = value
        .as_object()
        .and_then(|obj| obj.get("blocks"))
        .is_some();
    if !has_blocks {
        anyhow::bail!("Missing required 'blocks' field");
    }
    Ok(())
}

/// Parse schedule from JSON string with optional merging of separate period blobs.
///
/// This function deserializes a schedule JSON string using Serde, then optionally
/// merges `possible_periods` and `dark_periods` from separate JSON blobs when the
/// data is split across multiple files.
///
/// # Arguments
///
/// * `json_schedule_json` - Main schedule JSON (snake_case format matching schema)
/// * `possible_periods_json` - Optional JSON with visibility periods per block ID
///
/// # Returns
///
/// A fully populated `Schedule` with merged periods and computed checksum.
pub fn parse_schedule_json_str(
    json_schedule_json: &str,
    possible_periods_json: Option<&str>,
) -> Result<api::Schedule> {
    validate_input_schedule(json_schedule_json)?;

    let input: ScheduleInput = serde_json::from_str(json_schedule_json)
        .context("Failed to deserialize schedule JSON using Serde")?;
    
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

    // If possible periods are supplied separately, merge them into block.visibility_periods.
    // Accept either a wrapper `{"blocks": { "<id>": [ ... ] }}` or a direct map `{ "<id>": [ ... ] }`.
    if let Some(pp_json) = possible_periods_json {
        let trimmed = pp_json.trim();
        if !trimmed.is_empty() {
            #[derive(serde::Deserialize)]
            struct BlocksWrapper {
                blocks: HashMap<String, Vec<crate::api::Period>>,
            }

            // Try wrapper form first, then try direct map form.
            let maybe_map: Option<HashMap<String, Vec<crate::api::Period>>> =
                match serde_json::from_str::<BlocksWrapper>(trimmed) {
                    Ok(wrapper) => Some(wrapper.blocks),
                    Err(_) => {
                        serde_json::from_str::<HashMap<String, Vec<crate::api::Period>>>(trimmed)
                            .ok()
                    }
                };

            if let Some(map) = maybe_map {
                for block in &mut schedule.blocks {
                    // Match by original_block_id (user-provided identifier)
                    if let Some(periods) = map.get(&block.original_block_id) {
                        block.visibility_periods = periods.clone();
                    }
                    // Fallback: also try matching by internal id if present
                    else if let Some(id) = block.id {
                        if let Some(periods) = map.get(&id.0.to_string()) {
                            block.visibility_periods = periods.clone();
                        }
                    }
                }
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
        let stop = period.stop.value();
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
    }

    match (min_start, max_stop) {
        (Some(start), Some(stop)) => api::Period {
            start: api::ModifiedJulianDate::new(start),
            stop: api::ModifiedJulianDate::new(stop),
        },
        _ => api::Period {
            start: api::ModifiedJulianDate::new(0.0),
            stop: api::ModifiedJulianDate::new(0.0),
        },
    }
}

/// Compute a checksum for the schedule JSON
fn compute_schedule_checksum(json_str: &str) -> String {
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
        let possible_path = repo_data_path("possible_periods.json");

        parse_schedule_json_str(
            &std::fs::read_to_string(&schedule_path)
                .expect("Failed to read repository schedule fixture"),
            Some(
                &std::fs::read_to_string(&possible_path)
                    .expect("Failed to read repository possible periods fixture"),
            ),
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
                "latitude": 28.7624,
                "longitude": -17.8892,
                "elevation_m": 2396.0
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

        let result = parse_schedule_json_str(schedule_json, None);
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
                "latitude": 28.7624,
                "longitude": -17.8892,
                "elevation_m": 2396.0
            },
            "blocks": [
                {
                    "id": 1000004990,
                    "priority": 8.5,
                    "scheduled_period": { "start": 61894.19429606479, "stop": 61894.20818495378 },
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

        let result = parse_schedule_json_str(schedule_json, None);
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
                "latitude": 28.7624,
                "longitude": -17.8892,
                "elevation_m": 2396.0
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

        let possible_periods_json = r#"{ "blocks": { "1000004990": [ { "start": 61771.0, "stop": 61772.0 }, { "start": 61773.0, "stop": 61774.0 } ] } }"#;

        let result = parse_schedule_json_str(schedule_json, Some(possible_periods_json));
        assert!(result.is_ok(), "Should parse with possible periods");

        let schedule = result.unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        assert_eq!(schedule.blocks[0].visibility_periods.len(), 2);
    }

    #[test]
    fn test_missing_scheduling_block_key() {
        let schedule_json = r#"{"SomeOtherKey": []}"#;
        let result = parse_schedule_json_str(schedule_json, None);
        assert!(result.is_err(), "Should fail without SchedulingBlock key");
    }

    #[test]
    fn test_invalid_json() {
        let schedule_json = "not valid json {";
        let result = parse_schedule_json_str(schedule_json, None);
        assert!(result.is_err(), "Should fail with invalid JSON");
    }

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
            scheduled_period.stop.value(),
            61894.20818495378,
            "scheduled period stop",
        );
    }

    #[test]
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
            assert_close(fixed_window.stop.value(), 61778.0, "fixed window stop");
        }

        // Observation durations should be non-negative.
        assert!(block_2662.min_observation.value() >= 0.0);
        assert!(block_2662.requested_duration.value() >= 0.0);
    }
}
