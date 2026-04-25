//! Environment structure fingerprinting and matching.
//!
//! This module provides functionality to compute a stable fingerprint of a schedule's
//! structural characteristics (location, period, blocks) and to verify that schedules
//! match an environment's expected structure.

use crate::api::{EnvironmentStructure, Schedule};
use std::fmt;

/// Extracts the structural fingerprint from a schedule.
///
/// The fingerprint includes:
/// - Geographic location (latitude, longitude, elevation)
/// - Observation period (start and end MJD)
/// - Canonical hash of the blocks set
pub fn structure_from_schedule(schedule: &Schedule) -> EnvironmentStructure {
    // Extract location fields by serializing and deserializing
    // (Geodetic serializes as {"lon_deg": ..., "lat_deg": ..., "height": ...})
    let location_json = serde_json::to_value(schedule.geographic_location)
        .expect("Failed to serialize GeographicLocation");

    let lat_deg = location_json["lat_deg"]
        .as_f64()
        .expect("Missing lat_deg in GeographicLocation");
    let lon_deg = location_json["lon_deg"]
        .as_f64()
        .expect("Missing lon_deg in GeographicLocation");
    let elevation_m = location_json["height"]
        .as_f64()
        .expect("Missing height in GeographicLocation");

    EnvironmentStructure {
        period_start_mjd: schedule.schedule_period.start.value(),
        period_end_mjd: schedule.schedule_period.end.value(),
        lat_deg,
        lon_deg,
        elevation_m,
        blocks_hash: compute_blocks_hash(&schedule.blocks),
    }
}

/// Computes a stable hash of the blocks set.
///
/// The hash is computed by:
/// 1. Extracting canonical fields from each block (ID, priority, durations, target coords)
/// 2. Formatting each block as a pipe-separated string with stable float formatting
/// 3. Sorting the lines lexicographically
/// 4. Joining with newlines
/// 5. Computing SHA-256 and taking the first 16 hex characters
pub fn compute_blocks_hash(blocks: &[crate::api::SchedulingBlock]) -> String {
    use sha2::{Digest, Sha256};

    let mut lines: Vec<String> = blocks
        .iter()
        .map(|b| {
            format!(
                "{}|{}|{}|{}|{:.10}|{:.10}",
                b.original_block_id,
                b.priority,
                b.requested_duration.value() as i64, // seconds as integer
                b.min_observation.value() as i64,    // seconds as integer
                b.target_ra.value(),                 // degrees with fixed precision
                b.target_dec.value()                 // degrees with fixed precision
            )
        })
        .collect();

    // Sort lexicographically for stable hash
    lines.sort();

    let canonical = lines.join("\n");
    let hash = Sha256::digest(canonical.as_bytes());
    let hex = format!("{:x}", hash);
    hex.chars().take(16).collect()
}

/// Structure mismatch error.
///
/// Lists the fields that differ between an environment's expected structure
/// and a schedule's actual structure.
#[derive(Debug, Clone)]
pub struct StructureMismatch {
    pub fields: Vec<String>,
}

impl fmt::Display for StructureMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Structure mismatch: {}", self.fields.join(", "))
    }
}

impl std::error::Error for StructureMismatch {}

/// Checks whether a schedule matches an environment's structure.
///
/// Tolerances:
/// - MJD fields: 1e-9 days
/// - Latitude/longitude: 1e-6 degrees
/// - Elevation: 0.5 meters
/// - Blocks hash: exact string equality
///
/// Returns `Ok(())` if the schedule matches, or `Err(StructureMismatch)` with the
/// list of differing fields.
pub fn matches(env: &EnvironmentStructure, schedule: &Schedule) -> Result<(), StructureMismatch> {
    let schedule_structure = structure_from_schedule(schedule);
    let mut mismatches = Vec::new();

    const MJD_TOLERANCE: f64 = 1e-9;
    const COORD_TOLERANCE: f64 = 1e-6;
    const ELEVATION_TOLERANCE: f64 = 0.5;

    if (env.period_start_mjd - schedule_structure.period_start_mjd).abs() > MJD_TOLERANCE {
        mismatches.push("period_start_mjd".to_string());
    }
    if (env.period_end_mjd - schedule_structure.period_end_mjd).abs() > MJD_TOLERANCE {
        mismatches.push("period_end_mjd".to_string());
    }
    if (env.lat_deg - schedule_structure.lat_deg).abs() > COORD_TOLERANCE {
        mismatches.push("lat_deg".to_string());
    }
    if (env.lon_deg - schedule_structure.lon_deg).abs() > COORD_TOLERANCE {
        mismatches.push("lon_deg".to_string());
    }
    if (env.elevation_m - schedule_structure.elevation_m).abs() > ELEVATION_TOLERANCE {
        mismatches.push("elevation_m".to_string());
    }
    if env.blocks_hash != schedule_structure.blocks_hash {
        mismatches.push("blocks_hash".to_string());
    }

    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(StructureMismatch { fields: mismatches })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Constraints, ModifiedJulianDate, Period, SchedulingBlock};
    use qtty::{Degrees, Meters, Seconds};
    use siderust::coordinates::centers::Geodetic;
    use siderust::coordinates::frames::ECEF;

    fn make_test_schedule() -> Schedule {
        let block1 = SchedulingBlock {
            id: None,
            original_block_id: "block1".to_string(),
            block_name: "Test Block 1".to_string(),
            target_ra: Degrees::new(45.0),
            target_dec: Degrees::new(-30.0),
            constraints: Constraints::new(
                Degrees::new(20.0),
                Degrees::new(80.0),
                Degrees::new(0.0),
                Degrees::new(360.0),
                None,
            ),
            priority: 10.0,
            min_observation: Seconds::new(300.0),
            requested_duration: Seconds::new(600.0),
            visibility_periods: vec![],
            scheduled_period: None,
        };

        let block2 = SchedulingBlock {
            id: None,
            original_block_id: "block2".to_string(),
            block_name: "Test Block 2".to_string(),
            target_ra: Degrees::new(90.0),
            target_dec: Degrees::new(10.0),
            constraints: Constraints::new(
                Degrees::new(20.0),
                Degrees::new(80.0),
                Degrees::new(0.0),
                Degrees::new(360.0),
                None,
            ),
            priority: 5.0,
            min_observation: Seconds::new(200.0),
            requested_duration: Seconds::new(400.0),
            visibility_periods: vec![],
            scheduled_period: None,
        };

        Schedule {
            id: None,
            name: "Test Schedule".to_string(),
            checksum: "test_checksum".to_string(),
            schedule_period: Period {
                start: ModifiedJulianDate::new(60000.0),
                end: ModifiedJulianDate::new(60007.0),
            },
            dark_periods: vec![],
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.89),
                Degrees::new(28.76),
                Meters::new(2200.0),
            ),
            astronomical_nights: vec![],
            blocks: vec![block1, block2],
        }
    }

    #[test]
    fn test_compute_blocks_hash_is_order_invariant() {
        let schedule1 = make_test_schedule();
        let mut schedule2 = make_test_schedule();
        // Reverse block order
        schedule2.blocks.reverse();

        let hash1 = compute_blocks_hash(&schedule1.blocks);
        let hash2 = compute_blocks_hash(&schedule2.blocks);

        assert_eq!(hash1, hash2, "Hash should be order-invariant");
    }

    #[test]
    fn test_structure_from_schedule_then_matches() {
        let schedule = make_test_schedule();
        let structure = structure_from_schedule(&schedule);

        // A schedule should always match its own structure
        let result = matches(&structure, &schedule);
        assert!(result.is_ok());
    }

    #[test]
    fn test_modified_block_causes_mismatch() {
        let schedule = make_test_schedule();
        let structure = structure_from_schedule(&schedule);

        // Modify one block's priority
        let mut modified_schedule = schedule.clone();
        modified_schedule.blocks[0].priority = 999.0;

        let result = matches(&structure, &modified_schedule);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.fields.contains(&"blocks_hash".to_string()),
            "Expected blocks_hash mismatch, got: {:?}",
            err.fields
        );
    }

    #[test]
    fn test_different_latitude_causes_mismatch() {
        let schedule = make_test_schedule();
        let structure = structure_from_schedule(&schedule);

        // Change latitude by more than tolerance (1e-6 deg)
        // Geodetic::new takes (longitude, latitude, height)
        let mut modified_schedule = schedule.clone();
        modified_schedule.geographic_location = Geodetic::<ECEF>::new(
            Degrees::new(-17.89),        // Longitude unchanged
            Degrees::new(28.76 + 0.001), // Latitude changed by 0.001 degrees
            Meters::new(2200.0),
        );

        let result = matches(&structure, &modified_schedule);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.fields.contains(&"lat_deg".to_string()),
            "Expected lat_deg mismatch, got: {:?}",
            err.fields
        );
    }
}
