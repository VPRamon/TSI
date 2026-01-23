// ============================================================================
// JSON Parsing Functions
// ============================================================================
//
// These functions parse schedule JSON using the astro crate format exclusively.
// The astro format uses `tasks` and `location` fields, with visibility periods
// computed on-the-fly from target constraints.
//
// Legacy format support has been removed. All schedules must use astro schema:
// See `backend/astro/schemas/schedule.schema.json` for the schema.

use crate::api;
use crate::services::visibility_computer;
use anyhow::{Context, Result};

/// Parse schedule from JSON string using astro format.
///
/// The astro format uses `tasks` and `location` fields. Visibility periods
/// are computed on-the-fly from target constraints.
///
/// # Arguments
///
/// * `json_schedule_json` - Schedule JSON string in astro format
///
/// # Returns
///
/// A fully populated `Schedule` with visibility periods and computed checksum.
pub fn parse_schedule_json_str(json_schedule_json: &str) -> Result<api::Schedule> {
    parse_astro_schedule_json_str(json_schedule_json)
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
fn parse_astro_schedule_json_str(json_str: &str) -> Result<api::Schedule> {
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
    use std::path::PathBuf;

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
    fn test_parse_astro_schedule_with_constraints() {
        let astro_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("astro")
            .join("schedule_with_constraints.json");

        if astro_path.exists() {
            let content = std::fs::read_to_string(&astro_path)
                .expect("Failed to read astro schedule with constraints fixture");

            let result = parse_schedule_json_str(&content);
            assert!(result.is_ok(), "Should parse astro fixture with constraints: {:?}", result.err());

            let schedule = result.unwrap();
            assert_eq!(schedule.blocks.len(), 1, "Should have 1 observation block");
            
            // Verify constraint was extracted
            let block = &schedule.blocks[0];
            assert_close(block.constraints.min_alt.value(), 30.0, "min altitude");
            assert_close(block.constraints.max_alt.value(), 70.0, "max altitude");
        }
    }

    #[test]
    fn test_invalid_json() {
        let schedule_json = "not valid json {";
        let result = parse_schedule_json_str(schedule_json);
        assert!(result.is_err(), "Should fail with invalid JSON");
    }

    #[test]
    fn test_missing_required_fields() {
        // Missing tasks field
        let schedule_json = r#"{
            "location": { "lat": 28.7624, "lon": -17.8892, "distance": 6373.396 },
            "period": { "start": 60676.0, "end": 60677.0 }
        }"#;
        let result = parse_schedule_json_str(schedule_json);
        assert!(result.is_err(), "Should fail without tasks field");
    }
}
