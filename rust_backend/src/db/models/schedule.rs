//! Core schedule domain models.
//!
//! This module contains the primary domain types for the scheduling system:
//! - Schedule: Top-level schedule concept with metadata and blocks
//! - SchedulingBlock: Individual observing request
//! - Period: Time window representation
//! - Constraints: Observing constraints (altitude, azimuth, fixed time)
//! - ID types: Strongly-typed identifiers for database records

use siderust::{
    astro::ModifiedJulianDate,
    coordinates::spherical::direction::ICRS
};

use qtty::*;

macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident, $desc:literal) => {
        $(#[$meta])*
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct $name(pub i64);

        impl $name {
            pub fn new(value: i64) -> Self {
                Self(value)
            }

            pub fn value(&self) -> i64 {
                self.0
            }
        }
    };
}

id_type!(
    /// Strongly-typed identifier for a schedule record (maps to BIGINT).
    ScheduleId,
    "ScheduleId"
);
id_type!(
    /// Strongly-typed identifier for a target record.
    TargetId,
    "TargetId"
);
id_type!(
    /// Strongly-typed identifier for a constraints record.
    ConstraintsId,
    "ConstraintsId"
);
id_type!(
    /// Strongly-typed identifier for a scheduling block.
    SchedulingBlockId,
    "SchedulingBlockId"
);

/// Simple representation of a time window in Modified Julian Date.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Period {
    pub start: ModifiedJulianDate,
    pub stop: ModifiedJulianDate,
}

impl Period {
    pub fn new(start: ModifiedJulianDate, stop: ModifiedJulianDate) -> Option<Self> {
        if start.value() < stop.value() {
            Some(Self { start, stop })
        } else {
            None
        }
    }

    /// Length of the interval in days.
    pub fn duration(&self) -> Days {
        Days::new(self.stop.value() - self.start.value())
    }

    /// Check if a given MJD instant lies inside this interval (inclusive start, exclusive end).
    pub fn contains(&self, t_mjd: ModifiedJulianDate) -> bool {
        self.start.value() <= t_mjd.value() && t_mjd.value() < self.stop.value()
    }

    /// Check if this interval overlaps with another.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start.value() < other.stop.value() && other.start.value() < self.stop.value()
    }
}

#[derive(Debug, Clone)]
pub struct Constraints {
    pub min_alt: Degrees,
    pub max_alt: Degrees,
    pub min_az: Degrees,
    pub max_az: Degrees,
    pub fixed_time: Option<Period>,
}

/// Atomic observing request (mirrors scheduling_blocks).
#[derive(Debug, Clone)]
pub struct SchedulingBlock {
    pub id: SchedulingBlockId,
    pub original_block_id: Option<String>,
    pub target_ra: Degrees,
    pub target_dec: Degrees,
    pub constraints: Constraints,
    pub priority: f64,
    pub min_observation: Seconds,
    pub requested_duration: Seconds,
    pub visibility_periods: Vec<Period>,
    pub scheduled_period: Option<Period>,
}

impl SchedulingBlock {
    pub fn target(&self) -> ICRS {
        ICRS::new(self.target_ra, self.target_dec)
    }
}


/// Core "Schedule" concept:
/// - Metadata (name, checksum, etc.)
/// - Dark periods
/// - Assigned scheduling blocks with optional execution windows
#[derive(Debug, Clone)]
pub struct Schedule {
    pub id: Option<ScheduleId>,
    pub name: String,
    pub checksum: String,
    pub dark_periods: Vec<Period>,
    pub blocks: Vec<SchedulingBlock>,
}

// ============================================================================
// JSON Parsing (serde-based)
// ============================================================================

// Schedule JSON parsing using serde for declarative deserialization.
//
// This module provides functions to parse schedule JSON files into domain models.
// Uses serde_path_to_error for detailed error messages with JSON paths.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

/// Top-level schedule JSON structure
#[derive(Debug, Deserialize)]
struct ScheduleJson {
    #[serde(rename = "SchedulingBlock")]
    scheduling_blocks: Vec<SchedulingBlockJson>,
}

/// Top-level dark periods JSON structure
#[derive(Debug, Deserialize)]
struct DarkPeriodsJson {
    dark_periods: Vec<PeriodJson>,
}

/// Top-level possible periods JSON structure
#[derive(Debug, Deserialize)]
struct PossiblePeriodsJson {
    #[serde(rename = "SchedulingBlock")]
    scheduling_blocks: HashMap<String, Vec<PeriodJson>>,
}

/// Intermediate representation of a scheduling block from JSON
#[derive(Debug, Deserialize)]
struct SchedulingBlockJson {
    #[serde(rename = "schedulingBlockId", deserialize_with = "deserialize_flexible_id")]
    scheduling_block_id: (i64, Option<String>),
    priority: f64,
    target: TargetJson,
    #[serde(rename = "schedulingBlockConfiguration_")]
    configuration: SchedulingBlockConfiguration,
    scheduled_period: Option<PeriodJson>,
}

/// Custom deserializer for flexible ID handling (string/i64/f64)
fn deserialize_flexible_id<'de, D>(deserializer: D) -> Result<(i64, Option<String>), D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    
    let id = match &value {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i
            } else if let Some(f) = n.as_f64() {
                f as i64
            } else {
                return Err(D::Error::custom(format!("Invalid number for schedulingBlockId: {}", n)));
            }
        }
        Value::String(s) => s.parse::<i64>()
            .map_err(|e| D::Error::custom(format!("Cannot parse schedulingBlockId '{}': {}", s, e)))?,
        _ => return Err(D::Error::custom(format!("schedulingBlockId must be a number or string, got: {}", value))),
    };

    let original = Some(value.to_string().trim_matches('"').to_string());
    Ok((id, original))
}

#[derive(Debug, Deserialize)]
struct TargetJson {
    #[serde(rename = "position_")]
    position: PositionJson,
}

#[derive(Debug, Deserialize)]
struct PositionJson {
    coord: CoordJson,
}

#[derive(Debug, Deserialize)]
struct CoordJson {
    celestial: CelestialJson,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CelestialJson {
    ra_in_deg: f64,
    dec_in_deg: f64,
}

#[derive(Debug, Deserialize)]
struct SchedulingBlockConfiguration {
    #[serde(rename = "constraints_")]
    constraints: ConstraintsJson,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConstraintsJson {
    #[serde(rename = "elevationConstraint_")]
    elevation_constraint: ElevationConstraintJson,
    #[serde(rename = "azimuthConstraint_")]
    azimuth_constraint: AzimuthConstraintJson,
    #[serde(rename = "timeConstraint_")]
    time_constraint: TimeConstraintJson,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ElevationConstraintJson {
    min_elevation_angle_in_deg: f64,
    max_elevation_angle_in_deg: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzimuthConstraintJson {
    min_azimuth_angle_in_deg: f64,
    max_azimuth_angle_in_deg: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimeConstraintJson {
    min_observation_time_in_sec: f64,
    requested_duration_sec: f64,
    #[serde(default)]
    fixed_start_time: Vec<TimeValueJson>,
    #[serde(default)]
    fixed_stop_time: Vec<TimeValueJson>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TimeValueJson {
    Object { 
        #[serde(rename = "startTime")] 
        start_time: Option<MjdValue>, 
        #[serde(rename = "stopTime")] 
        stop_time: Option<MjdValue>, 
        value: Option<f64> 
    },
    Direct(MjdValue),
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MjdValue {
    value: f64,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    scale: Option<String>,
}

impl TimeValueJson {
    fn to_mjd(&self) -> Option<f64> {
        match self {
            TimeValueJson::Object { start_time, stop_time, value } => {
                value.or_else(|| start_time.as_ref().map(|t| t.value))
                     .or_else(|| stop_time.as_ref().map(|t| t.value))
            }
            TimeValueJson::Direct(mjd) => Some(mjd.value),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PeriodJson {
    start_time: MjdValue,
    stop_time: MjdValue,
    #[serde(default)]
    duration_in_sec: Option<f64>,
}

impl PeriodJson {
    fn to_period(&self) -> Option<Period> {
        Period::new(
            ModifiedJulianDate::new(self.start_time.value),
            ModifiedJulianDate::new(self.stop_time.value),
        )
    }
}

impl SchedulingBlockJson {
    fn to_scheduling_block(
        &self,
        possible_periods_map: Option<&HashMap<i64, Vec<Period>>>,
    ) -> Result<SchedulingBlock> {
        let (block_id, ref original_block_id) = self.scheduling_block_id;

        // Parse fixed time constraint
        let fixed_time = if !self.configuration.constraints.time_constraint.fixed_start_time.is_empty()
            && !self.configuration.constraints.time_constraint.fixed_stop_time.is_empty()
        {
            let start_mjd = self.configuration.constraints.time_constraint.fixed_start_time[0]
                .to_mjd()
                .context("Missing start time value")?;
            let stop_mjd = self.configuration.constraints.time_constraint.fixed_stop_time[0]
                .to_mjd()
                .context("Missing stop time value")?;
            Period::new(
                ModifiedJulianDate::new(start_mjd),
                ModifiedJulianDate::new(stop_mjd),
            )
        } else {
            None
        };

        let constraints = Constraints {
            min_alt: Degrees::new(self.configuration.constraints.elevation_constraint.min_elevation_angle_in_deg),
            max_alt: Degrees::new(self.configuration.constraints.elevation_constraint.max_elevation_angle_in_deg),
            min_az: Degrees::new(self.configuration.constraints.azimuth_constraint.min_azimuth_angle_in_deg),
            max_az: Degrees::new(self.configuration.constraints.azimuth_constraint.max_azimuth_angle_in_deg),
            fixed_time,
        };

        // Get visibility periods from possible_periods_map
        let visibility_periods = if let Some(map) = possible_periods_map {
            map.get(&block_id).cloned().unwrap_or_default()
        } else {
            Vec::new()
        };

        let scheduled_period = self.scheduled_period.as_ref().and_then(|p| p.to_period());

        Ok(SchedulingBlock {
            id: SchedulingBlockId(block_id),
            original_block_id: original_block_id.clone(),
            target_ra: Degrees::new(self.target.position.coord.celestial.ra_in_deg),
            target_dec: Degrees::new(self.target.position.coord.celestial.dec_in_deg),
            constraints,
            priority: self.priority,
            min_observation: Seconds::new(self.configuration.constraints.time_constraint.min_observation_time_in_sec),
            requested_duration: Seconds::new(self.configuration.constraints.time_constraint.requested_duration_sec),
            visibility_periods,
            scheduled_period,
        })
    }
}

/// Parses schedule JSON from a string into a `Schedule` structure.
///
/// # Arguments
/// * `json_schedule_json` - JSON string containing the scheduling blocks
/// * `possible_periods_json` - Optional JSON string with visibility periods per block ID
/// * `dark_periods_json` - JSON string containing dark periods
///
/// # Returns
/// A `Schedule` containing all parsed blocks and dark periods
pub fn parse_schedule_json_str(
    json_schedule_json: &str,
    possible_periods_json: Option<&str>,
    dark_periods_json: &str,
) -> Result<Schedule> {
    // Parse dark periods
    let dark_periods =
        parse_dark_periods_from_str(dark_periods_json).context("Failed to parse dark periods")?;

    // Parse possible periods if provided
    let possible_periods_map = if let Some(pp_json) = possible_periods_json {
        Some(parse_possible_periods_from_str(pp_json).context("Failed to parse possible periods")?)
    } else {
        None
    };

    // Parse scheduling blocks
    let blocks =
        parse_scheduling_blocks_from_str(json_schedule_json, possible_periods_map.as_ref())
            .context("Failed to parse scheduling blocks")?;

    // Create schedule with a default name and checksum
    let checksum = compute_schedule_checksum(json_schedule_json);

    Ok(Schedule {
        id: None,
        name: "parsed_schedule".to_string(),
        checksum,
        dark_periods,
        blocks,
    })
}

/// Parse dark periods from JSON string using serde
fn parse_dark_periods_from_str(json_str: &str) -> Result<Vec<Period>> {
    let jd = &mut serde_json::Deserializer::from_str(json_str);
    let dark_periods_json: DarkPeriodsJson = serde_path_to_error::deserialize(jd)
        .context("Failed to parse dark periods JSON")?;

    let periods: Vec<Period> = dark_periods_json
        .dark_periods
        .iter()
        .filter_map(|p| p.to_period())
        .collect();

    Ok(periods)
}

/// Parse possible periods (visibility windows) from JSON string using serde
/// Returns a map of scheduling_block_id -> Vec<Period>
fn parse_possible_periods_from_str(json_str: &str) -> Result<HashMap<i64, Vec<Period>>> {
    // Handle empty or whitespace-only strings
    let trimmed = json_str.trim();
    if trimmed.is_empty() {
        return Ok(HashMap::new());
    }

    let jd = &mut serde_json::Deserializer::from_str(trimmed);
    let possible_periods_json: PossiblePeriodsJson =
        serde_path_to_error::deserialize(jd).with_context(|| {
            let preview = if trimmed.len() > 200 {
                format!("{}...", &trimmed[..200])
            } else {
                trimmed.to_string()
            };
            format!(
                "Failed to parse possible periods JSON. First 200 chars: {}",
                preview
            )
        })?;

    let mut result = HashMap::new();

    for (block_id_str, periods_json) in possible_periods_json.scheduling_blocks {
        let block_id: i64 = block_id_str
            .parse()
            .with_context(|| format!("Invalid block ID in SchedulingBlock: {}", block_id_str))?;

        let periods: Vec<Period> = periods_json.iter().filter_map(|p| p.to_period()).collect();

        result.insert(block_id, periods);
    }

    Ok(result)
}

/// Parse scheduling blocks from JSON string using serde
fn parse_scheduling_blocks_from_str(
    json_str: &str,
    possible_periods_map: Option<&HashMap<i64, Vec<Period>>>,
) -> Result<Vec<SchedulingBlock>> {
    let jd = &mut serde_json::Deserializer::from_str(json_str);
    let schedule_json: ScheduleJson = serde_path_to_error::deserialize(jd)
        .context("Failed to parse schedule JSON")?;

    let mut blocks = Vec::new();

    for block_json in &schedule_json.scheduling_blocks {
        let block = block_json
            .to_scheduling_block(possible_periods_map)
            .context("Failed to convert scheduling block")?;
        blocks.push(block);
    }

    Ok(blocks)
}

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
) -> Result<Schedule> {
    // Read schedule JSON
    let schedule_json = std::fs::read_to_string(schedule_json_path).with_context(|| {
        format!("Failed to read schedule file: {}", schedule_json_path.display())
    })?;

    // Read possible periods if provided
    let possible_periods_json = if let Some(path) = possible_periods_json_path {
        Some(std::fs::read_to_string(path).with_context(|| {
            format!("Failed to read possible periods file: {}", path.display())
        })?)
    } else {
        None
    };

    // Read dark periods
    let dark_periods_json = std::fs::read_to_string(dark_periods_json_path).with_context(|| {
        format!("Failed to read dark periods file: {}", dark_periods_json_path.display())
    })?;

    // Parse using string-based function
    parse_schedule_json_str(&schedule_json, possible_periods_json.as_deref(), &dark_periods_json)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
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
            label, expected, value
        );
    }

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
                    "startTime": {"format": "MJD", "scale": "UTC", "value": 61771.0},
                    "stopTime": {"format": "MJD", "scale": "UTC", "value": 61772.0}
                }
            ]
        }"#;

        let result = parse_schedule_json_str(schedule_json, None, dark_periods_json);
        assert!(result.is_ok(), "Should parse minimal schedule: {:?}", result.err());

        let schedule = result.unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        assert_eq!(schedule.dark_periods.len(), 1);
        assert_eq!(schedule.blocks[0].id.0, 1000004990);
        assert_eq!(schedule.blocks[0].priority, 8.5);
    }

    #[test]
    fn test_parse_with_scheduled_period() {
        let schedule_json = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": 1000004990,
                    "priority": 8.5,
                    "scheduled_period": {
                        "durationInSec": 1200.0,
                        "startTime": {"format": "MJD", "scale": "UTC", "value": 61894.19429606479},
                        "stopTime": {"format": "MJD", "scale": "UTC", "value": 61894.20818495378}
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

        let dark_periods_json = r#"{"dark_periods": []}"#;
        let result = parse_schedule_json_str(schedule_json, None, dark_periods_json);
        assert!(result.is_ok(), "Should parse with scheduled period: {:?}", result.err());

        let schedule = result.unwrap();
        assert_eq!(schedule.blocks.len(), 1);
        assert!(schedule.blocks[0].scheduled_period.is_some());
    }

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
                        "position_": {"coord": {"celestial": {"raInDeg": 158.03, "decInDeg": -68.03, "raProperMotionInMarcsecYear": 0.0, "decProperMotionInMarcsecYear": 0.0, "equinox": 2000.0}}}
                    },
                    "schedulingBlockConfiguration_": {
                        "constraints_": {
                            "azimuthConstraint_": {"minAzimuthAngleInDeg": 0.0, "maxAzimuthAngleInDeg": 360.0},
                            "elevationConstraint_": {"minElevationAngleInDeg": 60.0, "maxElevationAngleInDeg": 90.0},
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
                    {"startTime": {"format": "MJD", "scale": "UTC", "value": 61771.0}, "stopTime": {"format": "MJD", "scale": "UTC", "value": 61772.0}},
                    {"startTime": {"format": "MJD", "scale": "UTC", "value": 61773.0}, "stopTime": {"format": "MJD", "scale": "UTC", "value": 61774.0}}
                ]
            }
        }"#;

        let dark_periods_json = r#"{"dark_periods": []}"#;
        let result = parse_schedule_json_str(schedule_json, Some(possible_periods_json), dark_periods_json);
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
        assert_eq!(schedule.dark_periods.len(), 314, "Unexpected dark period count");
        assert_eq!(schedule.name, "parsed_schedule");
        assert_eq!(schedule.checksum, EXPECTED_SCHEDULE_CHECKSUM);

        let first_dark = schedule.dark_periods.first().expect("Dark periods should not be empty");
        assert_close(first_dark.start.value(), 61771.0, "first dark period start");
        assert_close(first_dark.stop.value(), 61771.276910532266, "first dark period stop");

        let block_4990 = schedule.blocks.iter().find(|block| block.id.0 == 1000004990)
            .expect("Scheduling block 1000004990 should exist");
        assert_eq!(block_4990.priority, 8.5);
        assert_eq!(block_4990.visibility_periods.len(), 120);

        let scheduled_period = block_4990.scheduled_period
            .expect("Scheduling block 1000004990 should be scheduled");
        assert_close(scheduled_period.start.value(), 61894.19429606479, "scheduled period start");
        assert_close(scheduled_period.stop.value(), 61894.20818495378, "scheduled period stop");
    }

    #[test]
    fn test_parse_fixed_time_block_from_real_data() {
        let schedule = parse_real_schedule_fixture();

        let block_2662 = schedule.blocks.iter().find(|block| block.id.0 == 1000002662)
            .expect("Scheduling block 1000002662 should exist");
        assert_eq!(block_2662.visibility_periods.len(), 3, "Expected three possible periods");
        assert_close(block_2662.constraints.min_alt.value(), 45.0, "minimum altitude");
        assert_close(block_2662.constraints.max_alt.value(), 90.0, "maximum altitude");
        assert_close(block_2662.constraints.min_az.value(), 0.0, "minimum azimuth");
        assert_close(block_2662.constraints.max_az.value(), 360.0, "maximum azimuth");

        let fixed_window = block_2662.constraints.fixed_time
            .expect("Block 1000002662 should have a fixed time window");
        assert_close(fixed_window.start.value(), 61771.0, "fixed window start");
        assert_close(fixed_window.stop.value(), 61778.0, "fixed window stop");

        assert_close(block_2662.min_observation.value(), 1800.0, "min observation seconds");
        assert_close(block_2662.requested_duration.value(), 1800.0, "requested duration seconds");
    }
}
