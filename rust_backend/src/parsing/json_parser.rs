use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

use crate::core::domain::SchedulingBlock;

/// Raw JSON structure for time values
#[derive(Debug, Deserialize)]
struct TimeValue {
    value: f64,  // MJD value
}

/// Raw JSON structure for a scheduled period
#[derive(Debug, Deserialize)]
struct ScheduledPeriod {
    #[serde(rename = "startTime")]
    start_time: TimeValue,
    #[serde(rename = "stopTime")]
    stop_time: TimeValue,
}

/// Raw JSON structure for celestial coordinates
#[derive(Debug, Deserialize)]
struct Celestial {
    #[serde(rename = "raInDeg")]
    ra_in_deg: f64,
    #[serde(rename = "decInDeg")]
    dec_in_deg: f64,
}

/// Raw JSON structure for position coordinate
#[derive(Debug, Deserialize)]
struct Coord {
    celestial: Celestial,
}

/// Raw JSON structure for position
#[derive(Debug, Deserialize)]
struct Position {
    coord: Coord,
}

/// Raw JSON structure for target information
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Target {
    #[serde(rename = "id_")]
    id: Option<i64>,
    name: Option<String>,
    #[serde(rename = "position_")]
    position: Position,
}

/// Raw JSON structure for azimuth constraint
#[derive(Debug, Deserialize)]
struct AzimuthConstraint {
    #[serde(rename = "minAzimuthAngleInDeg")]
    min_azimuth_angle_in_deg: f64,
    #[serde(rename = "maxAzimuthAngleInDeg")]
    max_azimuth_angle_in_deg: f64,
}

/// Raw JSON structure for elevation constraint
#[derive(Debug, Deserialize)]
struct ElevationConstraint {
    #[serde(rename = "minElevationAngleInDeg")]
    min_elevation_angle_in_deg: f64,
    #[serde(rename = "maxElevationAngleInDeg")]
    max_elevation_angle_in_deg: f64,
}

/// Raw JSON structure for time constraint
#[derive(Debug, Deserialize)]
struct TimeConstraint {
    #[serde(rename = "minObservationTimeInSec")]
    min_observation_time_in_sec: Option<f64>,
    #[serde(rename = "requestedDurationSec")]
    requested_duration_sec: f64,
    #[serde(rename = "fixedStartTime")]
    fixed_start_time: Vec<f64>,  // Empty array if not fixed
    #[serde(rename = "fixedStopTime")]
    fixed_stop_time: Vec<f64>,   // Empty array if not fixed
}

/// Raw JSON structure for constraints
#[derive(Debug, Deserialize)]
struct Constraints {
    #[serde(rename = "azimuthConstraint_")]
    azimuth_constraint: AzimuthConstraint,
    #[serde(rename = "elevationConstraint_")]
    elevation_constraint: ElevationConstraint,
    #[serde(rename = "timeConstraint_")]
    time_constraint: TimeConstraint,
}

/// Raw JSON structure for scheduling block configuration
#[derive(Debug, Deserialize)]
struct SchedulingBlockConfiguration {
    #[serde(rename = "constraints_")]
    constraints: Constraints,
}

/// Raw JSON structure as it comes from schedule.json
#[derive(Debug, Deserialize)]
struct RawSchedulingBlock {
    #[serde(rename = "schedulingBlockId")]
    scheduling_block_id: i64,
    priority: f64,
    #[serde(rename = "scheduled_period")]
    scheduled_period: Option<ScheduledPeriod>,
    target: Target,
    #[serde(rename = "schedulingBlockConfiguration_")]
    scheduling_block_configuration: SchedulingBlockConfiguration,
}

/// Container for the JSON file structure
#[derive(Debug, Deserialize)]
struct ScheduleJson {
    #[serde(rename = "SchedulingBlock")]
    scheduling_blocks: Vec<RawSchedulingBlock>,
}

/// Parse schedule.json file into SchedulingBlock structures
pub fn parse_schedule_json(json_path: &Path) -> Result<Vec<SchedulingBlock>> {
    let json_content = std::fs::read_to_string(json_path)
        .with_context(|| format!("Failed to read JSON file: {}", json_path.display()))?;
    
    parse_schedule_json_str(&json_content)
}

/// Parse schedule JSON from a string
pub fn parse_schedule_json_str(json_str: &str) -> Result<Vec<SchedulingBlock>> {
    let schedule_json: ScheduleJson = serde_json::from_str(json_str)
        .with_context(|| {
            // Try to get more detailed error info
            let preview = if json_str.len() > 200 {
                format!("{}...", &json_str[..200])
            } else {
                json_str.to_string()
            };
            format!("Failed to parse schedule JSON. Preview: {}", preview)
        })?;
    
    Ok(schedule_json
        .scheduling_blocks
        .into_iter()
        .map(convert_raw_to_domain)
        .collect())
}

/// Convert raw JSON structure to domain model
fn convert_raw_to_domain(raw: RawSchedulingBlock) -> SchedulingBlock {
    use crate::core::domain::Period;
    use siderust::astro::ModifiedJulianDate;
    use siderust::coordinates::spherical::direction::ICRS;
    use siderust::units::{
        time::*,
        angular::Degrees,
    };

    let (scheduled_start, scheduled_stop) = raw
        .scheduled_period
        .map(|p| (Some(p.start_time.value), Some(p.stop_time.value)))
        .unwrap_or((None, None));
    
    let constraints = &raw.scheduling_block_configuration.constraints;
    let time_constraint = &constraints.time_constraint;
    
    // Get fixed times if they exist
    let fixed_start_time = time_constraint.fixed_start_time.first().copied();
    let fixed_stop_time = time_constraint.fixed_stop_time.first().copied();

    SchedulingBlock {
        scheduling_block_id: raw.scheduling_block_id.to_string(),
        priority: raw.priority,
        requested_duration: Seconds::new(time_constraint.requested_duration_sec),
        min_observation_time: Seconds::new(time_constraint.min_observation_time_in_sec.unwrap_or(0.0)),
        fixed_time: match (fixed_start_time, fixed_stop_time) {
            (Some(start), Some(stop)) => Some(Period::new(
                ModifiedJulianDate::new(start),
                ModifiedJulianDate::new(stop),
            )),
            _ => None,
        },
        coordinates: Some(ICRS::new(
            Degrees::new(raw.target.position.coord.celestial.ra_in_deg),
            Degrees::new(raw.target.position.coord.celestial.dec_in_deg),
        )),
        min_azimuth_angle: Some(Degrees::new(constraints.azimuth_constraint.min_azimuth_angle_in_deg)),
        max_azimuth_angle: Some(Degrees::new(constraints.azimuth_constraint.max_azimuth_angle_in_deg)),
        min_elevation_angle: Some(Degrees::new(constraints.elevation_constraint.min_elevation_angle_in_deg)),
        max_elevation_angle: Some(Degrees::new(constraints.elevation_constraint.max_elevation_angle_in_deg)),
        scheduled_period: match (scheduled_start, scheduled_stop) {
            (Some(start), Some(stop)) => Some(Period::new(
                ModifiedJulianDate::new(start),
                ModifiedJulianDate::new(stop),
            )),
            _ => None,
        },
        visibility_periods: vec![], // Will be enriched later with visibility data
    }
}

#[cfg(all(test, not(feature = "extension-module")))]
mod tests {
    use super::*;

    #[test]
    fn test_parse_schedule_json_str() {
        let json = r#"{
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
                                    "raInDeg": 158.03297990185885,
                                    "decInDeg": -68.02521140748772,
                                    "equinox": 2000.0,
                                    "raProperMotionInMarcsecYear": 0.0,
                                    "decProperMotionInMarcsecYear": 0.0
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

        let blocks = parse_schedule_json_str(json).unwrap();
        assert_eq!(blocks.len(), 1);
        
        let block = &blocks[0];
        assert_eq!(block.scheduling_block_id, "1000004990");
        assert_eq!(block.priority, 8.5);
        assert_eq!(block.requested_duration.value(), 1200.0);
        assert_eq!(block.min_observation_time.value(), 1200.0);
        
        // Check coordinates
        if let Some(coords) = &block.coordinates {
            assert!((coords.ra().value() - 158.03297990185885).abs() < 1e-6);
            assert!((coords.dec().value() - (-68.02521140748772)).abs() < 1e-6);
        } else {
            panic!("Expected coordinates to be present");
        }
        
        // Check scheduled period
        if let Some(period) = &block.scheduled_period {
            assert!((period.start.value() - 61894.19429606479).abs() < 1e-6);
            assert!((period.stop.value() - 61894.20818495378).abs() < 1e-6);
        } else {
            panic!("Expected scheduled period to be present");
        }
        
        assert!(block.is_scheduled());
    }
}
