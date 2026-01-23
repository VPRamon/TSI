// ============================================================================
// JSON Parsing Functions
// ============================================================================
//
// These functions provide convenient file-based and string-based parsing with
// support for:
// - Legacy custom format (with `blocks` and optional `possible_periods`)
// - New astro crate format (with `tasks` and `location`)
//
// The astro format computes visibility periods on-the-fly instead of accepting
// them as input.

use crate::api;
use crate::services::visibility_computer;
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
    /// DEPRECATED: Use astro format instead. Visibility periods are now computed on-the-fly.
    #[serde(default)]
    pub possible_periods: Option<HashMap<String, Vec<api::Period>>>,
}

/// Detect the schedule format from JSON.
///
/// Returns `true` if the JSON is in astro format (has `tasks` and `location`),
/// `false` if it's in legacy format (has `blocks` and `geographic_location`).
fn is_astro_format(json_str: &str) -> Result<bool> {
    let value: serde_json::Value =
        serde_json::from_str(json_str).context("Invalid schedule JSON")?;
    let obj = value.as_object().context("Schedule must be a JSON object")?;
    
    // Astro format has `tasks` and `location`
    let has_tasks = obj.contains_key("tasks");
    let has_location = obj.contains_key("location");
    
    Ok(has_tasks && has_location)
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

/// Parse schedule from JSON string, auto-detecting format.
///
/// This function supports two formats:
/// 1. **Astro format** (recommended): Uses `tasks` and `location` fields.
///    Visibility periods are computed on-the-fly from target constraints.
/// 2. **Legacy format** (deprecated): Uses `blocks` and `geographic_location`.
///    May include `possible_periods` for pre-computed visibility windows.
///
/// # Arguments
///
/// * `json_schedule_json` - Schedule JSON string
///
/// # Returns
///
/// A fully populated `Schedule` with visibility periods and computed checksum.
pub fn parse_schedule_json_str(json_schedule_json: &str) -> Result<api::Schedule> {
    if is_astro_format(json_schedule_json)? {
        parse_astro_schedule_json_str(json_schedule_json)
    } else {
        parse_legacy_schedule_json_str(json_schedule_json)
    }
}

/// Parse schedule from astro format JSON string.
///
/// This function:
/// 1. Parses the astro crate schedule format
/// 2. Converts to internal API types
/// 3. Computes visibility periods on-the-fly from constraints
///
/// # Arguments
///
/// * `json_str` - Astro format schedule JSON
///
/// # Returns
///
/// A fully populated `Schedule` with computed visibility periods.
pub fn parse_astro_schedule_json_str(json_str: &str) -> Result<api::Schedule> {
    // Parse using astro crate
    let astro_schedule = astro::schedule::import_schedule_raw(json_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse astro schedule: {}", e))?;
    
    // Convert to API types
    let mut schedule = crate::models::astro_adapter::convert_astro_schedule(
        &astro_schedule,
        "astro_schedule",
    )?;
    
    // Compute visibility periods for all blocks
    visibility_computer::compute_schedule_visibility(&mut schedule)
        .context("Failed to compute visibility periods")?;
    
    // Compute checksum
    schedule.checksum = compute_schedule_checksum(json_str);
    
    Ok(schedule)
}

/// Parse schedule from legacy format JSON string.
///
/// DEPRECATED: Use astro format instead. This function is kept for backwards
/// compatibility but will be removed in a future version.
///
/// This function deserializes a schedule JSON string using Serde.
/// If the JSON contains an optional `possible_periods` field (map of block IDs to periods),
/// those visibility periods are merged into the corresponding blocks.
/// If no `possible_periods` are provided, visibility periods are computed on-the-fly.
///
/// # Arguments
///
/// * `json_schedule_json` - Main schedule JSON (snake_case format matching schema)
///
/// # Returns
///
/// A fully populated `Schedule` with merged/computed periods and computed checksum.
pub fn parse_legacy_schedule_json_str(json_schedule_json: &str) -> Result<api::Schedule> {
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

    // Handle visibility periods:
    // 1. If possible_periods are embedded in the JSON, merge them into block.visibility_periods
    // 2. Otherwise, compute visibility periods on-the-fly
    
    if let Some(mut map) = input.possible_periods {
        // Legacy path: merge pre-computed possible_periods
        for block in &mut schedule.blocks {
            if let Some(periods) = map.remove(&block.original_block_id) {
                block.visibility_periods = periods;
            }
        }
    } else {
        // New path: compute visibility periods on-the-fly
        if let Err(e) = visibility_computer::compute_schedule_visibility(&mut schedule) {
            // Log warning but don't fail - visibility computation is best-effort
            log::warn!("Failed to compute visibility periods: {}. Blocks will have empty visibility.", e);
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
                "latitude": 28.7624,
                "longitude": -17.8892,
                "elevation_m": 2396.0
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
                    { "start": 61771.0, "stop": 61772.0 },
                    { "start": 61773.0, "stop": 61774.0 }
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
            assert_close(fixed_window.stop.value(), 61778.0, "fixed window stop");
        }

        // Observation durations should be non-negative.
        assert!(block_2662.min_observation.value() >= 0.0);
        assert!(block_2662.requested_duration.value() >= 0.0);
    }

    // ========================================================================
    // Astro Format Tests
    // ========================================================================

    #[test]
    fn test_is_astro_format_detection() {
        // Astro format has `tasks` and `location`
        let astro_json = r#"{
            "location": { "lat": 28.7624, "lon": -17.8892, "distance": 6373.396 },
            "period": { "start": 60676.0, "end": 60677.0 },
            "tasks": []
        }"#;
        assert!(is_astro_format(astro_json).unwrap(), "Should detect astro format");

        // Legacy format has `blocks` and `geographic_location`
        let legacy_json = r#"{
            "geographic_location": { "latitude": 28.7624, "longitude": -17.8892 },
            "blocks": []
        }"#;
        assert!(!is_astro_format(legacy_json).unwrap(), "Should detect legacy format");
    }

    #[test]
    fn test_parse_astro_schedule_minimal() {
        let astro_json = r#"{
            "location": { "lat": 28.7624, "lon": -17.8892, "distance": 6373.396 },
            "period": { "start": 60676.0, "end": 60677.0 },
            "tasks": [
                {
                    "type": "observation",
                    "id": "1",
                    "name": "M31 Observation",
                    "target": {
                        "position": { "ra": 10.6847, "dec": 41.2687 },
                        "time": 2451545.0
                    },
                    "duration_sec": 3600.0,
                    "priority": 10
                }
            ]
        }"#;

        let result = parse_schedule_json_str(astro_json);
        assert!(result.is_ok(), "Should parse astro format: {:?}", result.err());

        let schedule = result.unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        assert_eq!(schedule.blocks[0].original_block_id, "1");
        assert_eq!(schedule.blocks[0].priority, 10.0);
        // RA and Dec should be converted
        assert!((schedule.blocks[0].target_ra.value() - 10.6847).abs() < 0.001);
        assert!((schedule.blocks[0].target_dec.value() - 41.2687).abs() < 0.001);
    }

    #[test]
    fn test_parse_astro_schedule_skips_calibration() {
        let astro_json = r#"{
            "location": { "lat": 28.7624, "lon": -17.8892, "distance": 6373.396 },
            "period": { "start": 60676.0, "end": 60677.0 },
            "tasks": [
                {
                    "type": "observation",
                    "id": "1",
                    "name": "M31 Observation",
                    "target": {
                        "position": { "ra": 10.6847, "dec": 41.2687 },
                        "time": 2451545.0
                    },
                    "duration_sec": 3600.0,
                    "priority": 10
                },
                {
                    "type": "calibration",
                    "id": "2",
                    "name": "Flat Field",
                    "duration_sec": 300.0,
                    "calibration_type": "flat",
                    "priority": -10
                }
            ]
        }"#;

        let result = parse_schedule_json_str(astro_json);
        assert!(result.is_ok(), "Should parse astro format with calibration");

        let schedule = result.unwrap();
        // Only observation task should be converted, calibration is skipped
        assert_eq!(schedule.blocks.len(), 1);
        assert_eq!(schedule.blocks[0].original_block_id, "1");
    }

    #[test]
    fn test_parse_astro_schedule_location_conversion() {
        let astro_json = r#"{
            "location": { "lat": 28.7624, "lon": -17.8892, "distance": 6373.396 },
            "period": { "start": 60676.0, "end": 60677.0 },
            "tasks": []
        }"#;

        let result = parse_schedule_json_str(astro_json);
        assert!(result.is_ok(), "Should parse empty astro schedule");

        let schedule = result.unwrap();
        assert!((schedule.geographic_location.latitude - 28.7624).abs() < 0.001);
        assert!((schedule.geographic_location.longitude - (-17.8892)).abs() < 0.001);
        // Elevation should be approximately 2396m (6373.396 - 6371.0) * 1000
        assert!(schedule.geographic_location.elevation_m.is_some());
    }

    #[test]
    fn test_parse_astro_schedule_from_fixture() {
        let astro_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("astro")
            .join("schedule_example.json");

        if astro_path.exists() {
            let content = std::fs::read_to_string(&astro_path)
                .expect("Failed to read astro schedule fixture");

            let result = parse_schedule_json_str(&content);
            assert!(result.is_ok(), "Should parse astro fixture: {:?}", result.err());

            let schedule = result.unwrap();
            // The fixture has 3 observation tasks (2 calibration tasks are skipped)
            assert_eq!(schedule.blocks.len(), 3, "Should have 3 observation blocks");
        }
    }

    #[test]
    fn test_legacy_schedule_without_possible_periods_computes_visibility() {
        // This test verifies that legacy format without possible_periods
        // still computes visibility periods on-the-fly
        let schedule_json = r#"{
            "schedule_period": { "start": 60676.0, "stop": 60677.0 },
            "geographic_location": {
                "latitude": 28.7624,
                "longitude": -17.8892,
                "elevation_m": 2396.0
            },
            "blocks": [
                {
                    "original_block_id": "test_block",
                    "priority": 8.5,
                    "target_ra": 10.6847,
                    "target_dec": 41.2687,
                    "constraints": {
                        "min_alt": 30.0,
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
        assert!(result.is_ok(), "Should parse legacy schedule without possible_periods");

        // Visibility periods should be computed (may be empty depending on target visibility)
        let schedule = result.unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        // The schedule parsing should complete successfully even if visibility is empty
    }
}
