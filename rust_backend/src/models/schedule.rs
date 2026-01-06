// ============================================================================
// JSON Parsing Functions
// ============================================================================
//
// These functions provide convenient file-based and string-based parsing with
// support for merging separate JSON blobs (possible_periods, dark_periods)
// when data is split across multiple files.

use anyhow::{Context, Result};
use std::collections::HashMap;

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
/// * `dark_periods_json` - JSON with dark periods array or wrapper object
///
/// # Returns
///
/// A fully populated `Schedule` with merged periods and computed checksum.
pub fn parse_schedule_json_str(
    json_schedule_json: &str,
    possible_periods_json: Option<&str>,
    dark_periods_json: &str,
) -> Result<crate::api::Schedule> {
    // Deserialize schedule using Serde (snake_case JSON matching schema)
    let mut schedule =
        parse_schedule_direct(json_schedule_json).context("Failed to parse schedule JSON")?;

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
                        match serde_json::from_str::<HashMap<String, Vec<crate::api::Period>>>(
                            trimmed,
                        ) {
                            Ok(m) => Some(m),
                            Err(_) => None,
                        }
                    }
                };

            if let Some(map) = maybe_map {
                for block in &mut schedule.blocks {
                    if let Some(periods) = map.get(&block.id.to_string()) {
                        block.visibility_periods = periods.clone();
                    }
                }
            }
        }
    }

    // If dark periods were supplied separately (or the parsed schedule lacks them),
    // try to parse and merge from the provided blob. Accept either wrapper {"dark_periods": [...]}
    // or a direct array of periods.
    if schedule.dark_periods.is_empty() && !dark_periods_json.trim().is_empty() {
        let trimmed = dark_periods_json.trim();

        #[derive(serde::Deserialize)]
        struct DarkWrapper {
            dark_periods: Vec<crate::api::Period>,
        }

        if let Ok(wrapper) = serde_json::from_str::<DarkWrapper>(trimmed) {
            schedule.dark_periods = wrapper.dark_periods;
        } else if let Ok(vec) = serde_json::from_str::<Vec<crate::api::Period>>(trimmed) {
            schedule.dark_periods = vec;
        }
    }

    if schedule.checksum.is_empty() {
        schedule.checksum = compute_schedule_checksum(json_schedule_json);
    }

    Ok(schedule)
}

/// Parse schedule from new snake_case JSON format (matches schema)
pub fn parse_schedule_direct(json_str: &str) -> Result<crate::api::Schedule> {
    let mut schedule: crate::api::Schedule = serde_json::from_str(json_str)
        .context("Failed to deserialize schedule JSON using Serde")?;

    // Compute checksum if not provided
    if schedule.checksum.is_empty() {
        schedule.checksum = compute_schedule_checksum(json_str);
    }

    Ok(schedule)
}

// (Legacy adapters removed) The codebase now expects schedule JSON to match the
// snake_case schema in `rust_backend/docs/schedule.schema.json`. Legacy
// camelCase adapters were removed to simplify parsing and reduce maintenance.

/// Compute a checksum for the schedule JSON
fn compute_schedule_checksum(json_str: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(json_str.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Parses schedule JSON from files into a `Schedule` structure.
///
/// # Arguments
/// * `schedule_json_path` - Path to JSON file containing the scheduling blocks
/// * `possible_periods_json_path` - Optional path to JSON file with visibility periods per block ID
/// * `dark_periods_json_path` - Path to JSON file containing dark periods
///
/// # Returns
/// A `Schedule` containing all parsed blocks and dark periods
pub fn parse_schedule_json(
    schedule_json_path: &std::path::Path,
    possible_periods_json_path: Option<&std::path::Path>,
    dark_periods_json_path: &std::path::Path,
) -> Result<crate::api::Schedule> {
    // Read schedule JSON
    let schedule_json = std::fs::read_to_string(schedule_json_path).with_context(|| {
        format!(
            "Failed to read schedule file: {}",
            schedule_json_path.display()
        )
    })?;

    // Read possible periods if provided
    let possible_periods_json =
        if let Some(path) = possible_periods_json_path {
            Some(std::fs::read_to_string(path).with_context(|| {
                format!("Failed to read possible periods file: {}", path.display())
            })?)
        } else {
            None
        };

    // Read dark periods
    let dark_periods_json = std::fs::read_to_string(dark_periods_json_path).with_context(|| {
        format!(
            "Failed to read dark periods file: {}",
            dark_periods_json_path.display()
        )
    })?;

    // Parse using string-based function
    parse_schedule_json_str(
        &schedule_json,
        possible_periods_json.as_deref(),
        &dark_periods_json,
    )
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

    #[test]
    fn test_parse_minimal_schedule() {
        let schedule_json = r#"{
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

        let dark_periods_json = r#"{ "dark_periods": [ { "start": 61771.0, "stop": 61772.0 } ] }"#;

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

    #[test]
    fn test_parse_with_scheduled_period() {
        let schedule_json = r#"{
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

        let dark_periods_json = r#"{"dark_periods": []}"#;
        let result = parse_schedule_json_str(schedule_json, None, dark_periods_json);
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

        let dark_periods_json = r#"{"dark_periods": []}"#;
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

    #[test]
    fn test_missing_scheduling_block_key() {
        let schedule_json = r#"{"SomeOtherKey": []}"#;
        let dark_periods_json = r#"{"dark_periods": []}"#;
        let result = parse_schedule_json_str(schedule_json, None, dark_periods_json);
        assert!(result.is_err(), "Should fail without SchedulingBlock key");
    }

    #[test]
    fn test_invalid_json() {
        let schedule_json = "not valid json {";
        let dark_periods_json = r#"{"dark_periods": []}"#;
        let result = parse_schedule_json_str(schedule_json, None, dark_periods_json);
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
            .find(|block| block.id.0 == 1000004990)
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
            .find(|block| block.id.0 == 1000002662)
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
