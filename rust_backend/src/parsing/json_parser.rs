use crate::db::models::*;
use anyhow::{Context, Result};
use serde_json::Value;
use siderust::{
    astro::ModifiedJulianDate, coordinates::spherical::direction::ICRS, units::angular::Degrees,
    units::time::Seconds,
};
use std::collections::HashMap;

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

/// Parse dark periods from JSON string
fn parse_dark_periods_from_str(json_str: &str) -> Result<Vec<Period>> {
    let value: Value =
        serde_json::from_str(json_str).context("Failed to parse dark periods JSON")?;

    let dark_periods_array = value
        .get("dark_periods")
        .and_then(|v| v.as_array())
        .context("Missing or invalid 'dark_periods' array")?;

    let mut periods = Vec::new();

    for period_value in dark_periods_array {
        if let Some(period) = parse_period_from_value(period_value)? {
            periods.push(period);
        }
    }

    Ok(periods)
}

/// Parse possible periods (visibility windows) from JSON string
/// Returns a map of scheduling_block_id -> Vec<Period>
fn parse_possible_periods_from_str(json_str: &str) -> Result<HashMap<i64, Vec<Period>>> {
    let value: Value =
        serde_json::from_str(json_str).context("Failed to parse possible periods JSON")?;

    let scheduling_block_obj = value
        .get("SchedulingBlock")
        .and_then(|v| v.as_object())
        .context("Missing or invalid 'SchedulingBlock' object")?;

    let mut result = HashMap::new();

    for (block_id_str, periods_array) in scheduling_block_obj {
        let block_id: i64 = block_id_str
            .parse()
            .with_context(|| format!("Invalid block ID: {}", block_id_str))?;

        let periods_arr = periods_array
            .as_array()
            .context("Expected array of periods")?;

        let mut periods = Vec::new();
        for period_value in periods_arr {
            if let Some(period) = parse_period_from_value(period_value)? {
                periods.push(period);
            }
        }

        result.insert(block_id, periods);
    }

    Ok(result)
}

/// Parse scheduling blocks from JSON string
fn parse_scheduling_blocks_from_str(
    json_str: &str,
    possible_periods_map: Option<&HashMap<i64, Vec<Period>>>,
) -> Result<Vec<SchedulingBlock>> {
    let value: Value = serde_json::from_str(json_str).context("Failed to parse schedule JSON")?;

    let blocks_array = value
        .get("SchedulingBlock")
        .and_then(|v| v.as_array())
        .context("Missing or invalid 'SchedulingBlock' array")?;

    let mut blocks = Vec::new();

    for block_value in blocks_array {
        let block = parse_scheduling_block(block_value, possible_periods_map)
            .context("Failed to parse scheduling block")?;
        blocks.push(block);
    }

    Ok(blocks)
}

/// Parse a single scheduling block from JSON
fn parse_scheduling_block(
    value: &Value,
    possible_periods_map: Option<&HashMap<i64, Vec<Period>>>,
) -> Result<SchedulingBlock> {
    // Extract block ID
    let block_id = value
        .get("schedulingBlockId")
        .and_then(|v| v.as_i64())
        .context("Missing or invalid 'schedulingBlockId'")?;

    // Store original block ID as string for database tracking
    let original_block_id = value
        .get("schedulingBlockId")
        .and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else if let Some(i) = v.as_i64() {
                Some(i.to_string())
            } else if let Some(f) = v.as_f64() {
                Some(f.to_string())
            } else {
                None
            }
        });

    // Extract priority
    let priority = value
        .get("priority")
        .and_then(|v| v.as_f64())
        .context("Missing or invalid 'priority'")?;

    // Extract target coordinates
    let target = parse_target(value)?;

    // Extract constraints
    let constraints = parse_constraints(value)?;

    // Extract time constraints
    let (min_observation, requested_duration) = parse_time_constraints(value)?;

    // Extract scheduled period (if exists)
    let scheduled_period = match value.get("scheduled_period") {
        Some(v) => parse_period_from_value(v)?,
        None => None,
    };

    // Get visibility periods from possible_periods_map
    let visibility_periods = if let Some(map) = possible_periods_map {
        map.get(&block_id).cloned().unwrap_or_default()
    } else {
        Vec::new()
    };

    Ok(SchedulingBlock {
        id: SchedulingBlockId(block_id),
        original_block_id,
        target,
        constraints,
        priority,
        min_observation,
        requested_duration,
        visibility_periods,
        scheduled_period,
    })
}

/// Parse target coordinates from scheduling block JSON
fn parse_target(value: &Value) -> Result<ICRS> {
    let target_obj = value.get("target").context("Missing 'target' field")?;

    let position = target_obj
        .get("position_")
        .context("Missing 'position_' field")?;

    let coord = position.get("coord").context("Missing 'coord' field")?;

    let celestial = coord
        .get("celestial")
        .context("Missing 'celestial' field")?;

    let ra_deg = celestial
        .get("raInDeg")
        .and_then(|v| v.as_f64())
        .context("Missing or invalid 'raInDeg'")?;

    let dec_deg = celestial
        .get("decInDeg")
        .and_then(|v| v.as_f64())
        .context("Missing or invalid 'decInDeg'")?;

    Ok(ICRS::new(Degrees::new(ra_deg), Degrees::new(dec_deg)))
}

/// Parse constraints from scheduling block JSON
fn parse_constraints(value: &Value) -> Result<Constraints> {
    let config = value
        .get("schedulingBlockConfiguration_")
        .context("Missing 'schedulingBlockConfiguration_' field")?;

    let constraints_obj = config
        .get("constraints_")
        .context("Missing 'constraints_' field")?;

    // Parse elevation constraint
    let elevation_constraint = constraints_obj
        .get("elevationConstraint_")
        .context("Missing 'elevationConstraint_' field")?;

    let min_alt = elevation_constraint
        .get("minElevationAngleInDeg")
        .and_then(|v| v.as_f64())
        .context("Missing or invalid 'minElevationAngleInDeg'")? as f64;

    let max_alt = elevation_constraint
        .get("maxElevationAngleInDeg")
        .and_then(|v| v.as_f64())
        .context("Missing or invalid 'maxElevationAngleInDeg'")? as f64;

    // Parse azimuth constraint
    let azimuth_constraint = constraints_obj
        .get("azimuthConstraint_")
        .context("Missing 'azimuthConstraint_' field")?;

    let min_az = azimuth_constraint
        .get("minAzimuthAngleInDeg")
        .and_then(|v| v.as_f64())
        .context("Missing or invalid 'minAzimuthAngleInDeg'")? as f64;

    let max_az = azimuth_constraint
        .get("maxAzimuthAngleInDeg")
        .and_then(|v| v.as_f64())
        .context("Missing or invalid 'maxAzimuthAngleInDeg'")? as f64;

    // Parse time constraint for fixed time (if exists)
    let time_constraint = constraints_obj
        .get("timeConstraint_")
        .context("Missing 'timeConstraint_' field")?;

    let fixed_start = time_constraint
        .get("fixedStartTime")
        .and_then(|v| v.as_array());

    let fixed_stop = time_constraint
        .get("fixedStopTime")
        .and_then(|v| v.as_array());

    let fixed_time = match (fixed_start, fixed_stop) {
        (Some(start_arr), Some(stop_arr)) if !start_arr.is_empty() && !stop_arr.is_empty() => {
            let start_mjd = parse_time_entry(&start_arr[0], "startTime")?;
            let stop_mjd = parse_time_entry(&stop_arr[0], "stopTime")?;
            Period::new(start_mjd, stop_mjd)
        }
        _ => None,
    };

    Ok(Constraints {
        min_alt: Degrees::new(min_alt),
        max_alt: Degrees::new(max_alt),
        min_az: Degrees::new(min_az),
        max_az: Degrees::new(max_az),
        fixed_time,
    })
}

/// Extract time constraint parameters from scheduling block JSON
fn parse_time_constraints(value: &Value) -> Result<(Seconds, Seconds)> {
    let config = value
        .get("schedulingBlockConfiguration_")
        .context("Missing 'schedulingBlockConfiguration_' field")?;

    let constraints_obj = config
        .get("constraints_")
        .context("Missing 'constraints_' field")?;

    let time_constraint = constraints_obj
        .get("timeConstraint_")
        .context("Missing 'timeConstraint_' field")?;

    let min_observation_sec = time_constraint
        .get("minObservationTimeInSec")
        .and_then(|v| v.as_f64())
        .context("Missing or invalid 'minObservationTimeInSec'")?
        as f64;

    let requested_duration_sec = time_constraint
        .get("requestedDurationSec")
        .and_then(|v| v.as_f64())
        .context("Missing or invalid 'requestedDurationSec'")?
        as f64;

    Ok((
        Seconds::new(min_observation_sec),
        Seconds::new(requested_duration_sec),
    ))
}

/// Parse a single fixed time entry helper supporting both full period objects
/// and standalone timestamp objects.
fn parse_time_entry(value: &Value, key_hint: &str) -> Result<ModifiedJulianDate> {
    if let Some(obj) = value.get(key_hint) {
        return parse_time_value_object(obj, key_hint);
    }

    if let Some(obj) = value.get("startTime") {
        return parse_time_value_object(obj, "startTime");
    }

    if let Some(obj) = value.get("stopTime") {
        return parse_time_value_object(obj, "stopTime");
    }

    parse_time_value_object(value, key_hint)
}

/// Extract the numeric MJD value from a time object.
fn parse_time_value_object(value: &Value, context_label: &str) -> Result<ModifiedJulianDate> {
    let mjd = value
        .get("value")
        .and_then(|v| v.as_f64())
        .or_else(|| value.as_f64())
        .with_context(|| format!("Missing or invalid '{}' in time entry", context_label))?;

    Ok(ModifiedJulianDate::new(mjd))
}

/// Parse a period from a JSON value
fn parse_period_from_value(value: &Value) -> Result<Option<Period>> {
    if value.is_null() {
        return Ok(None);
    }

    let start_obj = value
        .get("startTime")
        .context("Missing 'startTime' in period")?;

    let stop_obj = value
        .get("stopTime")
        .context("Missing 'stopTime' in period")?;

    let start_mjd = start_obj
        .get("value")
        .and_then(|v| v.as_f64())
        .context("Missing or invalid 'value' in startTime")?;

    let stop_mjd = stop_obj
        .get("value")
        .and_then(|v| v.as_f64())
        .context("Missing or invalid 'value' in stopTime")?;

    let period = Period::new(
        ModifiedJulianDate::new(start_mjd),
        ModifiedJulianDate::new(stop_mjd),
    );

    Ok(period)
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
