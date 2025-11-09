/// JSON loader for raw schedule data with preprocessing
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;

use crate::models::schedule::{SchedulingBlock, VisibilityPeriod};
use crate::preprocessing::preprocess_block;

/// Raw scheduling block from JSON
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSchedulingBlock {
    scheduling_block_id: serde_json::Value, // Can be string or number
    priority: f64,
    #[serde(default)]
    scheduled_period: Option<ScheduledPeriod>,
    #[serde(rename = "schedulingBlockConfiguration_")]
    scheduling_block_configuration: SchedulingBlockConfiguration,
    target: Target,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScheduledPeriod {
    start_time: TimeValue,
    stop_time: TimeValue,
}

#[derive(Debug, serde::Deserialize)]
struct TimeValue {
    value: f64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchedulingBlockConfiguration {
    #[serde(rename = "constraints_")]
    constraints: Constraints,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Constraints {
    #[serde(rename = "timeConstraint_")]
    time_constraint: TimeConstraint,
    #[serde(rename = "azimuthConstraint_")]
    azimuth_constraint: AzimuthConstraint,
    #[serde(rename = "elevationConstraint_")]
    elevation_constraint: ElevationConstraint,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimeConstraint {
    min_observation_time_in_sec: f64,
    requested_duration_sec: f64,
    #[serde(default)]
    fixed_start_time: Vec<TimeValue>,
    #[serde(default)]
    fixed_stop_time: Vec<TimeValue>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzimuthConstraint {
    min_azimuth_angle_in_deg: f64,
    max_azimuth_angle_in_deg: f64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ElevationConstraint {
    min_elevation_angle_in_deg: f64,
    max_elevation_angle_in_deg: f64,
}

#[derive(Debug, serde::Deserialize)]
struct Target {
    #[serde(rename = "position_")]
    position: Position,
}

#[derive(Debug, serde::Deserialize)]
struct Position {
    coord: Coord,
}

#[derive(Debug, serde::Deserialize)]
struct Coord {
    celestial: Celestial,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Celestial {
    dec_in_deg: f64,
    ra_in_deg: f64,
}

/// Parse visibility data from possible_periods.json structure
fn parse_visibility_data(json_str: &str) -> Result<HashMap<String, Vec<VisibilityPeriod>>> {
    let data: Value = serde_json::from_str(json_str)?;
    let mut visibility_map = HashMap::new();

    if let Some(scheduling_blocks) = data.get("SchedulingBlock").and_then(|v| v.as_object()) {
        for (sb_id, periods) in scheduling_blocks {
            if let Some(period_list) = periods.as_array() {
                let mut vis_periods = Vec::new();
                for period in period_list {
                    if let (Some(start_val), Some(stop_val)) = (
                        period.get("startTime").and_then(|t| t.get("value")).and_then(|v| v.as_f64()),
                        period.get("stopTime").and_then(|t| t.get("value")).and_then(|v| v.as_f64()),
                    ) {
                        vis_periods.push(VisibilityPeriod {
                            start: start_val,
                            stop: stop_val,
                        });
                    }
                }
                visibility_map.insert(sb_id.clone(), vis_periods);
            }
        }
    }

    Ok(visibility_map)
}

/// Load and preprocess scheduling blocks from raw schedule JSON
/// Optionally merge with visibility data from possible_periods.json
pub fn load_json(schedule_json: &str, visibility_json: Option<&str>) -> Result<Vec<SchedulingBlock>> {
    // Parse visibility data if provided
    let visibility_map = if let Some(vis_json) = visibility_json {
        parse_visibility_data(vis_json)
            .context("Failed to parse visibility JSON")?
    } else {
        HashMap::new()
    };

    // Parse schedule JSON
    let data: Value = serde_json::from_str(schedule_json)
        .context("Failed to parse schedule JSON")?;

    let raw_blocks: Vec<RawSchedulingBlock> = serde_json::from_value(
        data.get("SchedulingBlock")
            .context("Missing 'SchedulingBlock' key")?
            .clone(),
    )
    .context("Failed to deserialize scheduling blocks")?;

    let mut blocks = Vec::with_capacity(raw_blocks.len());

    for raw in raw_blocks {
        // Convert ID to string (handles both numeric and string IDs)
        let sb_id = match raw.scheduling_block_id {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s,
            _ => continue, // Skip invalid IDs
        };

        // Get visibility periods for this block
        let visibility = visibility_map
            .get(&sb_id)
            .cloned()
            .unwrap_or_default();

        // Extract scheduled period (filter out sentinel value 51910.5)
        let (sched_start, sched_stop) = if let Some(sched) = &raw.scheduled_period {
            let start = sched.start_time.value;
            if start == 51910.5 {
                (None, None) // Sentinel value indicates unscheduled
            } else {
                (Some(start), Some(sched.stop_time.value))
            }
        } else {
            (None, None)
        };

        // Extract fixed times
        let fixed_start = raw.scheduling_block_configuration
            .constraints
            .time_constraint
            .fixed_start_time
            .first()
            .map(|t| t.value);

        let fixed_stop = raw.scheduling_block_configuration
            .constraints
            .time_constraint
            .fixed_stop_time
            .first()
            .map(|t| t.value);

        // Create block with minimal data (preprocessing will compute derived fields)
        let mut block = SchedulingBlock {
            scheduling_block_id: sb_id,
            priority: raw.priority,
            min_observation_time_in_sec: raw.scheduling_block_configuration
                .constraints
                .time_constraint
                .min_observation_time_in_sec,
            requested_duration_sec: raw.scheduling_block_configuration
                .constraints
                .time_constraint
                .requested_duration_sec,
            fixed_start_time: fixed_start,
            fixed_stop_time: fixed_stop,
            dec_in_deg: raw.target.position.coord.celestial.dec_in_deg,
            ra_in_deg: raw.target.position.coord.celestial.ra_in_deg,
            min_azimuth_angle_in_deg: raw.scheduling_block_configuration
                .constraints
                .azimuth_constraint
                .min_azimuth_angle_in_deg,
            max_azimuth_angle_in_deg: raw.scheduling_block_configuration
                .constraints
                .azimuth_constraint
                .max_azimuth_angle_in_deg,
            min_elevation_angle_in_deg: raw.scheduling_block_configuration
                .constraints
                .elevation_constraint
                .min_elevation_angle_in_deg,
            max_elevation_angle_in_deg: raw.scheduling_block_configuration
                .constraints
                .elevation_constraint
                .max_elevation_angle_in_deg,
            scheduled_period_start: sched_start,
            scheduled_period_stop: sched_stop,
            visibility,
            // Derived fields (will be computed by preprocessing)
            num_visibility_periods: 0,
            total_visibility_hours: 0.0,
            priority_bin: crate::models::schedule::PriorityBin::NoPriority,
            scheduled_flag: false,
            requested_hours: 0.0,
            elevation_range_deg: 0.0,
        };

        // Preprocess to compute derived fields
        preprocess_block(&mut block);

        blocks.push(block);
    }

    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_json_minimal() {
        let json = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": 1000004990,
                    "priority": 8.5,
                    "schedulingBlockConfiguration_": {
                        "constraints_": {
                            "timeConstraint_": {
                                "minObservationTimeInSec": 1200,
                                "requestedDurationSec": 1200,
                                "fixedStartTime": [],
                                "fixedStopTime": []
                            },
                            "azimuthConstraint_": {
                                "minAzimuthAngleInDeg": 0.0,
                                "maxAzimuthAngleInDeg": 360.0
                            },
                            "elevationConstraint_": {
                                "minElevationAngleInDeg": 60.0,
                                "maxElevationAngleInDeg": 90.0
                            }
                        }
                    },
                    "target": {
                        "position_": {
                            "coord": {
                                "celestial": {
                                    "decInDeg": -68.02521140748772,
                                    "raInDeg": 158.03297990185885
                                }
                            }
                        }
                    }
                }
            ]
        }"#;

        let blocks = load_json(json, None).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].scheduling_block_id, "1000004990");
        assert_eq!(blocks[0].priority, 8.5);
        assert!((blocks[0].requested_hours - 0.333333).abs() < 0.001);
        assert_eq!(blocks[0].elevation_range_deg, 30.0);
    }
}
