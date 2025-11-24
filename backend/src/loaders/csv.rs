/// CSV loader using Polars for efficient data loading
use anyhow::{Context, Result};
use polars::prelude::*;
use std::path::Path;

use crate::models::schedule::{PriorityBin, SchedulingBlock, VisibilityPeriod};

/// Parse a visibility string from CSV format
/// Format: "[(start1, stop1), (start2, stop2), ...]"
fn parse_visibility_string(vis_str: &str) -> Result<Vec<VisibilityPeriod>> {
    if vis_str.is_empty() || vis_str == "[]" {
        return Ok(Vec::new());
    }

    let mut periods = Vec::new();
    
    // Remove outer brackets and split by "), ("
    let trimmed = vis_str.trim_start_matches('[').trim_end_matches(']');
    
    for pair_str in trimmed.split("), (") {
        let pair_clean = pair_str
            .trim_start_matches('(')
            .trim_end_matches(')')
            .trim();
        
        let parts: Vec<&str> = pair_clean.split(',').map(|s| s.trim()).collect();
        
        if parts.len() == 2 {
            let start: f64 = parts[0].parse().context("Failed to parse visibility start")?;
            let stop: f64 = parts[1].parse().context("Failed to parse visibility stop")?;
            periods.push(VisibilityPeriod { start, stop });
        }
    }
    
    Ok(periods)
}

/// Parse priority bin string to enum
fn parse_priority_bin(bin_str: &str) -> PriorityBin {
    match bin_str {
        "Low (0-5)" => PriorityBin::Low,
        "Medium (5-8)" => PriorityBin::Medium,
        "Medium (8-10)" => PriorityBin::MediumHigh,
        "High (10+)" => PriorityBin::High,
        _ => PriorityBin::NoPriority,
    }
}

/// Load scheduling blocks from a CSV file
pub fn load_csv<P: AsRef<Path>>(path: P) -> Result<Vec<SchedulingBlock>> {
    let df = CsvReader::from_path(path.as_ref())?
        .finish()
        .context("Failed to read CSV file")?;

    // Validate required columns
    let required_cols = vec![
        "schedulingBlockId",
        "priority",
        "minObservationTimeInSec",
        "requestedDurationSec",
        "decInDeg",
        "raInDeg",
        "minAzimuthAngleInDeg",
        "maxAzimuthAngleInDeg",
        "minElevationAngleInDeg",
        "maxElevationAngleInDeg",
        "visibility",
        "num_visibility_periods",
        "total_visibility_hours",
        "priority_bin",
        "scheduled_flag",
        "requested_hours",
        "elevation_range_deg",
    ];

    for col in &required_cols {
        if !df.get_column_names().contains(col) {
            anyhow::bail!("Missing required column: {}", col);
        }
    }

    let num_rows = df.height();
    let mut blocks = Vec::with_capacity(num_rows);

    // Extract columns as ChunkedArrays
    // Store the column for IDs (might be i64 or utf8)
    let ids_col = df.column("schedulingBlockId")?.clone();
    
    // Helper closure to get f64 from a column that might be i64 or f64
    let get_f64_col = |col_name: &str| -> Result<ChunkedArray<Float64Type>> {
        let col = df.column(col_name)?;
        if col.dtype() == &DataType::Float64 {
            Ok(col.f64()?.clone())
        } else if col.dtype() == &DataType::Int64 {
            let casted = col.cast(&DataType::Float64)?;
            Ok(casted.f64()?.clone())
        } else {
            anyhow::bail!("Column {} has unexpected type: {:?}", col_name, col.dtype())
        }
    };
    
    let priorities = get_f64_col("priority")?;
    let min_obs_times = get_f64_col("minObservationTimeInSec")?;
    let req_durations = get_f64_col("requestedDurationSec")?;
    let fixed_starts_opt = df.column("fixedStartTime").ok();
    let fixed_stops_opt = df.column("fixedStopTime").ok();
    let decs = get_f64_col("decInDeg")?;
    let ras = get_f64_col("raInDeg")?;
    let min_azs = get_f64_col("minAzimuthAngleInDeg")?;
    let max_azs = get_f64_col("maxAzimuthAngleInDeg")?;
    let min_elevs = get_f64_col("minElevationAngleInDeg")?;
    let max_elevs = get_f64_col("maxElevationAngleInDeg")?;
    let sched_starts_opt = df.column("scheduled_period.start").ok();
    let sched_stops_opt = df.column("scheduled_period.stop").ok();
    let visibility_strs = df.column("visibility")?.utf8()?;
    // num_visibility_periods might be i64, so we get it and convert
    let num_vis_col = df.column("num_visibility_periods")?;
    let num_vis_periods = if num_vis_col.dtype() == &DataType::Int64 {
        num_vis_col.i64()?.clone()
    } else {
        num_vis_col.cast(&DataType::Int64)?.i64()?.clone()
    };
    let total_vis_hours = get_f64_col("total_visibility_hours")?;
    let priority_bins = df.column("priority_bin")?.utf8()?;
    let scheduled_flags = df.column("scheduled_flag")?.bool()?;
    let req_hours = get_f64_col("requested_hours")?;
    let elev_ranges = get_f64_col("elevation_range_deg")?;

    for i in 0..num_rows {
        // Handle both numeric and string IDs
        let id = match ids_col.dtype() {
            DataType::Int64 => {
                let id_val = ids_col.i64()?.get(i).context("Missing ID")?;
                id_val.to_string()
            }
            DataType::Utf8 => {
                ids_col.utf8()?.get(i).context("Missing ID")?.to_string()
            }
            _ => {
                // Try casting to string
                let str_col = ids_col.cast(&DataType::Utf8)?;
                str_col.utf8()?.get(i).context("Missing ID")?.to_string()
            }
        };
        
        let priority = priorities.get(i).context("Missing priority")?;
        let min_obs_time = min_obs_times.get(i).context("Missing min observation time")?;
        let req_duration = req_durations.get(i).context("Missing requested duration")?;
        
        // Handle optional numeric columns with type flexibility
        let fixed_start = fixed_starts_opt.as_ref().and_then(|col| {
            if col.dtype() == &DataType::Float64 {
                col.f64().ok().and_then(|ca| ca.get(i))
            } else if col.dtype() == &DataType::Int64 {
                col.i64().ok().and_then(|ca| ca.get(i)).map(|v| v as f64)
            } else {
                None
            }
        });
        
        let fixed_stop = fixed_stops_opt.as_ref().and_then(|col| {
            if col.dtype() == &DataType::Float64 {
                col.f64().ok().and_then(|ca| ca.get(i))
            } else if col.dtype() == &DataType::Int64 {
                col.i64().ok().and_then(|ca| ca.get(i)).map(|v| v as f64)
            } else {
                None
            }
        });
        
        let dec = decs.get(i).context("Missing dec")?;
        let ra = ras.get(i).context("Missing ra")?;
        let min_az = min_azs.get(i).context("Missing min azimuth")?;
        let max_az = max_azs.get(i).context("Missing max azimuth")?;
        let min_elev = min_elevs.get(i).context("Missing min elevation")?;
        let max_elev = max_elevs.get(i).context("Missing max elevation")?;
        
        let sched_start = sched_starts_opt.as_ref().and_then(|col| {
            if col.dtype() == &DataType::Float64 {
                col.f64().ok().and_then(|ca| ca.get(i))
            } else if col.dtype() == &DataType::Int64 {
                col.i64().ok().and_then(|ca| ca.get(i)).map(|v| v as f64)
            } else {
                None
            }
        });
        
        let sched_stop = sched_stops_opt.as_ref().and_then(|col| {
            if col.dtype() == &DataType::Float64 {
                col.f64().ok().and_then(|ca| ca.get(i))
            } else if col.dtype() == &DataType::Int64 {
                col.i64().ok().and_then(|ca| ca.get(i)).map(|v| v as f64)
            } else {
                None
            }
        });
        
        let vis_str = visibility_strs.get(i).context("Missing visibility")?;
        let visibility = parse_visibility_string(vis_str)?;
        
        let num_vis = num_vis_periods.get(i).context("Missing num visibility periods")? as usize;
        let total_vis = total_vis_hours.get(i).context("Missing total visibility hours")?;
        let priority_bin_str = priority_bins.get(i).context("Missing priority bin")?;
        let priority_bin = parse_priority_bin(priority_bin_str);
        let scheduled = scheduled_flags.get(i).context("Missing scheduled flag")?;
        let req_hrs = req_hours.get(i).context("Missing requested hours")?;
        let elev_range = elev_ranges.get(i).context("Missing elevation range")?;

        blocks.push(SchedulingBlock {
            scheduling_block_id: id,
            priority,
            min_observation_time_in_sec: min_obs_time,
            requested_duration_sec: req_duration,
            fixed_start_time: fixed_start,
            fixed_stop_time: fixed_stop,
            target_name: None,
            target_id: None,
            dec_in_deg: dec,
            ra_in_deg: ra,
            min_azimuth_angle_in_deg: min_az,
            max_azimuth_angle_in_deg: max_az,
            min_elevation_angle_in_deg: min_elev,
            max_elevation_angle_in_deg: max_elev,
            scheduled_period_start: sched_start,
            scheduled_period_stop: sched_stop,
            visibility,
            num_visibility_periods: num_vis,
            total_visibility_hours: total_vis,
            priority_bin,
            scheduled_flag: scheduled,
            requested_hours: req_hrs,
            elevation_range_deg: elev_range,
        });
    }

    Ok(blocks)
}

/// Load scheduling blocks from CSV bytes (for file uploads)
pub fn load_csv_from_bytes(data: &[u8]) -> Result<Vec<SchedulingBlock>> {
    // Use Cursor to create a reader from bytes
    let cursor = std::io::Cursor::new(data);
    let df = CsvReader::new(cursor)
        .finish()
        .context("Failed to read CSV from bytes")?;

    // Validate required columns
    let required_cols = vec![
        "schedulingBlockId",
        "priority",
        "minObservationTimeInSec",
        "requestedDurationSec",
        "decInDeg",
        "raInDeg",
        "minAzimuthAngleInDeg",
        "maxAzimuthAngleInDeg",
        "minElevationAngleInDeg",
        "maxElevationAngleInDeg",
        "visibility",
        "num_visibility_periods",
        "total_visibility_hours",
        "priority_bin",
        "scheduled_flag",
        "requested_hours",
        "elevation_range_deg",
    ];

    for col in &required_cols {
        if !df.get_column_names().contains(col) {
            anyhow::bail!("Missing required column: {}", col);
        }
    }

    let num_rows = df.height();
    let mut blocks = Vec::with_capacity(num_rows);

    // Extract columns as ChunkedArrays
    // Store the column for IDs (might be i64 or utf8)
    let ids_col = df.column("schedulingBlockId")?.clone();
    
    // Helper closure to get f64 from a column that might be i64 or f64
    let get_f64_col = |col_name: &str| -> Result<ChunkedArray<Float64Type>> {
        let col = df.column(col_name)?;
        if col.dtype() == &DataType::Float64 {
            Ok(col.f64()?.clone())
        } else if col.dtype() == &DataType::Int64 {
            let casted = col.cast(&DataType::Float64)?;
            Ok(casted.f64()?.clone())
        } else {
            anyhow::bail!("Column {} has unexpected type: {:?}", col_name, col.dtype())
        }
    };
    
    let priorities = get_f64_col("priority")?;
    let min_obs_times = get_f64_col("minObservationTimeInSec")?;
    let req_durations = get_f64_col("requestedDurationSec")?;
    let fixed_starts_opt = df.column("fixedStartTime").ok();
    let fixed_stops_opt = df.column("fixedStopTime").ok();
    let decs = get_f64_col("decInDeg")?;
    let ras = get_f64_col("raInDeg")?;
    let min_azs = get_f64_col("minAzimuthAngleInDeg")?;
    let max_azs = get_f64_col("maxAzimuthAngleInDeg")?;
    let min_elevs = get_f64_col("minElevationAngleInDeg")?;
    let max_elevs = get_f64_col("maxElevationAngleInDeg")?;
    let sched_starts_opt = df.column("scheduled_period.start").ok();
    let sched_stops_opt = df.column("scheduled_period.stop").ok();
    let visibility_strs = df.column("visibility")?.utf8()?;
    // num_visibility_periods might be i64, so we get it and convert
    let num_vis_col = df.column("num_visibility_periods")?;
    let num_vis_periods = if num_vis_col.dtype() == &DataType::Int64 {
        num_vis_col.i64()?.clone()
    } else {
        num_vis_col.cast(&DataType::Int64)?.i64()?.clone()
    };
    let total_vis_hours = get_f64_col("total_visibility_hours")?;
    let priority_bins = df.column("priority_bin")?.utf8()?;
    let scheduled_flags = df.column("scheduled_flag")?.bool()?;
    let req_hours = get_f64_col("requested_hours")?;
    let elev_ranges = get_f64_col("elevation_range_deg")?;

    for i in 0..num_rows {
        // Handle both numeric and string IDs
        let id = match ids_col.dtype() {
            DataType::Int64 => {
                let id_val = ids_col.i64()?.get(i).context("Missing ID")?;
                id_val.to_string()
            }
            DataType::Utf8 => {
                ids_col.utf8()?.get(i).context("Missing ID")?.to_string()
            }
            _ => {
                // Try casting to string
                let str_col = ids_col.cast(&DataType::Utf8)?;
                str_col.utf8()?.get(i).context("Missing ID")?.to_string()
            }
        };
        
        let priority = priorities.get(i).context("Missing priority")?;
        let min_obs_time = min_obs_times.get(i).context("Missing min observation time")?;
        let req_duration = req_durations.get(i).context("Missing requested duration")?;
        
        // Handle optional numeric columns with type flexibility
        let fixed_start = fixed_starts_opt.as_ref().and_then(|col| {
            if col.dtype() == &DataType::Float64 {
                col.f64().ok().and_then(|ca| ca.get(i))
            } else if col.dtype() == &DataType::Int64 {
                col.i64().ok().and_then(|ca| ca.get(i)).map(|v| v as f64)
            } else {
                None
            }
        });
        
        let fixed_stop = fixed_stops_opt.as_ref().and_then(|col| {
            if col.dtype() == &DataType::Float64 {
                col.f64().ok().and_then(|ca| ca.get(i))
            } else if col.dtype() == &DataType::Int64 {
                col.i64().ok().and_then(|ca| ca.get(i)).map(|v| v as f64)
            } else {
                None
            }
        });
        
        let dec = decs.get(i).context("Missing dec")?;
        let ra = ras.get(i).context("Missing ra")?;
        let min_az = min_azs.get(i).context("Missing min azimuth")?;
        let max_az = max_azs.get(i).context("Missing max azimuth")?;
        let min_elev = min_elevs.get(i).context("Missing min elevation")?;
        let max_elev = max_elevs.get(i).context("Missing max elevation")?;
        
        let sched_start = sched_starts_opt.as_ref().and_then(|col| {
            if col.dtype() == &DataType::Float64 {
                col.f64().ok().and_then(|ca| ca.get(i))
            } else if col.dtype() == &DataType::Int64 {
                col.i64().ok().and_then(|ca| ca.get(i)).map(|v| v as f64)
            } else {
                None
            }
        });
        
        let sched_stop = sched_stops_opt.as_ref().and_then(|col| {
            if col.dtype() == &DataType::Float64 {
                col.f64().ok().and_then(|ca| ca.get(i))
            } else if col.dtype() == &DataType::Int64 {
                col.i64().ok().and_then(|ca| ca.get(i)).map(|v| v as f64)
            } else {
                None
            }
        });
        
        let vis_str = visibility_strs.get(i).context("Missing visibility")?;
        let visibility = parse_visibility_string(vis_str)?;
        
        let num_vis = num_vis_periods.get(i).context("Missing num visibility periods")? as usize;
        let total_vis = total_vis_hours.get(i).context("Missing total visibility hours")?;
        let priority_bin_str = priority_bins.get(i).context("Missing priority bin")?;
        let priority_bin = parse_priority_bin(priority_bin_str);
        let scheduled = scheduled_flags.get(i).context("Missing scheduled flag")?;
        let req_hrs = req_hours.get(i).context("Missing requested hours")?;
        let elev_range = elev_ranges.get(i).context("Missing elevation range")?;

        blocks.push(SchedulingBlock {
            scheduling_block_id: id,
            priority,
            min_observation_time_in_sec: min_obs_time,
            requested_duration_sec: req_duration,
            fixed_start_time: fixed_start,
            fixed_stop_time: fixed_stop,
            target_name: None,
            target_id: None,
            dec_in_deg: dec,
            ra_in_deg: ra,
            min_azimuth_angle_in_deg: min_az,
            max_azimuth_angle_in_deg: max_az,
            min_elevation_angle_in_deg: min_elev,
            max_elevation_angle_in_deg: max_elev,
            scheduled_period_start: sched_start,
            scheduled_period_stop: sched_stop,
            visibility,
            num_visibility_periods: num_vis,
            total_visibility_hours: total_vis,
            priority_bin,
            scheduled_flag: scheduled,
            requested_hours: req_hrs,
            elevation_range_deg: elev_range,
        });
    }

    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_visibility_string() {
        let vis_str = "[(61892.19975694455, 61892.21081296308), (61893.19702662015, 61893.21006319439)]";
        let periods = parse_visibility_string(vis_str).unwrap();
        
        assert_eq!(periods.len(), 2);
        assert!((periods[0].start - 61892.19975694455).abs() < 1e-9);
        assert!((periods[0].stop - 61892.21081296308).abs() < 1e-9);
    }

    #[test]
    fn test_parse_empty_visibility() {
        let periods = parse_visibility_string("[]").unwrap();
        assert_eq!(periods.len(), 0);
    }

    #[test]
    fn test_parse_priority_bin() {
        assert!(matches!(parse_priority_bin("Low (0-5)"), PriorityBin::Low));
        assert!(matches!(parse_priority_bin("Medium (8-10)"), PriorityBin::MediumHigh));
        assert!(matches!(parse_priority_bin("No priority"), PriorityBin::NoPriority));
    }
}
