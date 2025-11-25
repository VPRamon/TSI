use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer};
use std::path::Path;

use crate::core::domain::SchedulingBlock;

/// Custom deserializer that accepts either string or integer for scheduling block ID
fn deserialize_scheduling_block_id<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        String(String),
        Int(i64),
    }
    
    match StringOrInt::deserialize(deserializer)? {
        StringOrInt::String(s) => s.parse::<i64>().map_err(D::Error::custom),
        StringOrInt::Int(i) => Ok(i),
    }
}

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
    fixed_start_time: Vec<TimeValue>,  // Array of TimeValue objects (can be empty)
    #[serde(rename = "fixedStopTime")]
    fixed_stop_time: Vec<TimeValue>,   // Array of TimeValue objects (can be empty)
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
    #[serde(rename = "schedulingBlockId", deserialize_with = "deserialize_scheduling_block_id")]
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
    // First validate that it's valid JSON
    let json_value: serde_json::Value = serde_json::from_str(json_str)
        .with_context(|| {
            let preview = if json_str.len() > 500 {
                format!("{}...", &json_str[..500])
            } else {
                json_str.to_string()
            };
            format!("Invalid JSON syntax. First 500 chars: {}", preview)
        })?;
    
    // Check if SchedulingBlock key exists
    if !json_value.is_object() || !json_value.as_object().unwrap().contains_key("SchedulingBlock") {
        anyhow::bail!(
            "JSON must contain a 'SchedulingBlock' key. Found keys: {:?}",
            json_value.as_object().map(|o| o.keys().collect::<Vec<_>>())
        );
    }
    
    // Now try to deserialize with detailed error handling
    let schedule_json: ScheduleJson = serde_json::from_value(json_value.clone())
        .map_err(|e| {
            // Provide detailed error information
            let error_msg = format!("JSON deserialization error: {}", e);
            
            // Try to identify which block is causing the issue
            if let Some(blocks) = json_value.get("SchedulingBlock").and_then(|v| v.as_array()) {
                // Try to deserialize blocks one by one to find the problematic one
                for (idx, block) in blocks.iter().enumerate() {
                    if let Err(block_err) = serde_json::from_value::<RawSchedulingBlock>(block.clone()) {
                        return anyhow::anyhow!(
                            "{}\nError in SchedulingBlock at index {}: {}\nBlock data: {}",
                            error_msg,
                            idx,
                            block_err,
                            serde_json::to_string_pretty(block).unwrap_or_else(|_| "cannot display".to_string())
                        );
                    }
                }
            }
            
            anyhow::anyhow!("{}", error_msg)
        })?;
    
    Ok(schedule_json
        .scheduling_blocks
        .into_iter()
        .enumerate()
        .map(|(idx, raw)| {
            // Wrap conversion to add context about which block failed
            convert_raw_to_domain(raw, idx)
        })
        .collect())
}

/// Convert raw JSON structure to domain model
fn convert_raw_to_domain(raw: RawSchedulingBlock, _idx: usize) -> SchedulingBlock {
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
    
    // Get fixed times if they exist - extract the .value from TimeValue
    let fixed_start_time = time_constraint.fixed_start_time.first().map(|tv| tv.value);
    let fixed_stop_time = time_constraint.fixed_stop_time.first().map(|tv| tv.value);

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
