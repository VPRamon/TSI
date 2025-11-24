use anyhow::{Context, Result};
use polars::prelude::*;
use std::path::Path;

use crate::core::domain::SchedulingBlock;
use crate::parsing::visibility::VisibilityParser;

/// Parse CSV file into a Polars DataFrame
pub fn parse_schedule_csv(csv_path: &Path) -> Result<DataFrame> {
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(csv_path.into()))?
        .finish()
        .context("Failed to parse CSV into DataFrame")?;
    
    Ok(df)
}

/// Parse CSV and convert to SchedulingBlock structures
pub fn parse_schedule_csv_to_blocks(csv_path: &Path) -> Result<Vec<SchedulingBlock>> {
    let df = parse_schedule_csv(csv_path)?;
    dataframe_to_blocks(&df)
}

/// Convert a Polars DataFrame to SchedulingBlock structures
pub fn dataframe_to_blocks(df: &DataFrame) -> Result<Vec<SchedulingBlock>> {
    let mut blocks = Vec::new();
    let height = df.height();
    
    // Extract columns
    let ids = df.column("schedulingBlockId")?.str()?;
    let priorities = df.column("priority")?.f64()?;
    let requested_durations = df.column("requestedDurationSec")?.f64()?;
    let min_obs_times = df.column("minObservationTimeInSec").ok().and_then(|c| c.f64().ok());
    
    let ra = df.column("raInDeg").ok().and_then(|c| c.f64().ok());
    let dec = df.column("decInDeg").ok().and_then(|c| c.f64().ok());
    
    let min_az = df.column("minAzimuthAngleInDeg").ok().and_then(|c| c.f64().ok());
    let max_az = df.column("maxAzimuthAngleInDeg").ok().and_then(|c| c.f64().ok());
    let min_el = df.column("minElevationAngleInDeg").ok().and_then(|c| c.f64().ok());
    let max_el = df.column("maxElevationAngleInDeg").ok().and_then(|c| c.f64().ok());
    
    let scheduled_starts = df.column("scheduled_period.start").ok().and_then(|c| c.f64().ok());
    let scheduled_stops = df.column("scheduled_period.stop").ok().and_then(|c| c.f64().ok());
    
    let visibility_col = df.column("visibility").ok().and_then(|c| c.str().ok());
    
    for i in 0..height {
        let id = ids.get(i)
            .with_context(|| format!("Missing schedulingBlockId at row {}", i))?
            .to_string();
        
        let priority = priorities.get(i)
            .with_context(|| format!("Missing priority at row {}", i))?;
        
        let requested_duration = requested_durations.get(i)
            .with_context(|| format!("Missing requestedDurationSec at row {}", i))?;
        
        // Parse visibility periods if available
        let visibility_periods = if let Some(vis_col) = visibility_col {
            if let Some(vis_str) = vis_col.get(i) {
                VisibilityParser::parse(vis_str).unwrap_or_default()
            } else {
                vec![]
            }
        } else {
            vec![]
        };
        
        let block = SchedulingBlock {
            scheduling_block_id: id,
            priority,
            requested_duration_sec: requested_duration,
            min_observation_time_sec: min_obs_times.and_then(|col| col.get(i)),
            fixed_start_time: None, // Not typically in CSV
            fixed_stop_time: None,
            ra_in_deg: ra.and_then(|col| col.get(i)),
            dec_in_deg: dec.and_then(|col| col.get(i)),
            min_azimuth_angle_in_deg: min_az.and_then(|col| col.get(i)),
            max_azimuth_angle_in_deg: max_az.and_then(|col| col.get(i)),
            min_elevation_angle_in_deg: min_el.and_then(|col| col.get(i)),
            max_elevation_angle_in_deg: max_el.and_then(|col| col.get(i)),
            scheduled_start: scheduled_starts.and_then(|col| col.get(i)),
            scheduled_stop: scheduled_stops.and_then(|col| col.get(i)),
            visibility_periods,
        };
        
        blocks.push(block);
    }
    
    Ok(blocks)
}

/// Convert SchedulingBlock structures to a Polars DataFrame
pub fn blocks_to_dataframe(blocks: &[SchedulingBlock]) -> Result<DataFrame> {
    let n = blocks.len();
    
    // Prepare column vectors
    let mut ids = Vec::with_capacity(n);
    let mut priorities = Vec::with_capacity(n);
    let mut requested_durations = Vec::with_capacity(n);
    let mut min_obs_times = Vec::with_capacity(n);
    
    let mut ras = Vec::with_capacity(n);
    let mut decs = Vec::with_capacity(n);
    
    let mut min_azs = Vec::with_capacity(n);
    let mut max_azs = Vec::with_capacity(n);
    let mut min_els = Vec::with_capacity(n);
    let mut max_els = Vec::with_capacity(n);
    
    let mut scheduled_starts = Vec::with_capacity(n);
    let mut scheduled_stops = Vec::with_capacity(n);
    let mut scheduled_flags = Vec::with_capacity(n);
    
    let mut num_vis_periods = Vec::with_capacity(n);
    let mut total_vis_hours = Vec::with_capacity(n);
    let mut requested_hours = Vec::with_capacity(n);
    let mut elevation_ranges = Vec::with_capacity(n);
    let mut priority_bins = Vec::with_capacity(n);
    
    for block in blocks {
        ids.push(block.scheduling_block_id.clone());
        priorities.push(block.priority);
        requested_durations.push(block.requested_duration_sec);
        min_obs_times.push(block.min_observation_time_sec);
        
        ras.push(block.ra_in_deg);
        decs.push(block.dec_in_deg);
        
        min_azs.push(block.min_azimuth_angle_in_deg);
        max_azs.push(block.max_azimuth_angle_in_deg);
        min_els.push(block.min_elevation_angle_in_deg);
        max_els.push(block.max_elevation_angle_in_deg);
        
        scheduled_starts.push(block.scheduled_start);
        scheduled_stops.push(block.scheduled_stop);
        scheduled_flags.push(block.is_scheduled());
        
        num_vis_periods.push(block.num_visibility_periods() as u32);
        total_vis_hours.push(block.total_visibility_hours());
        requested_hours.push(block.requested_hours());
        elevation_ranges.push(block.elevation_range_deg());
        priority_bins.push(block.priority_bin().to_string());
    }
    
    // Create DataFrame
    let df = df!(
        "schedulingBlockId" => ids,
        "priority" => priorities,
        "requestedDurationSec" => requested_durations,
        "minObservationTimeInSec" => min_obs_times,
        "raInDeg" => ras,
        "decInDeg" => decs,
        "minAzimuthAngleInDeg" => min_azs,
        "maxAzimuthAngleInDeg" => max_azs,
        "minElevationAngleInDeg" => min_els,
        "maxElevationAngleInDeg" => max_els,
        "scheduled_period.start" => scheduled_starts,
        "scheduled_period.stop" => scheduled_stops,
        "scheduled_flag" => scheduled_flags,
        "num_visibility_periods" => num_vis_periods,
        "total_visibility_hours" => total_vis_hours,
        "requested_hours" => requested_hours,
        "elevation_range_deg" => elevation_ranges,
        "priority_bin" => priority_bins,
    )?;
    
    Ok(df)
}

#[cfg(all(test, not(feature = "extension-module")))]
mod tests {
    use super::*;
    use crate::core::domain::SchedulingBlock;

    #[test]
    fn test_blocks_to_dataframe_roundtrip() {
        let blocks = vec![
            SchedulingBlock {
                scheduling_block_id: "1000004990".to_string(),
                priority: 8.5,
                requested_duration_sec: 1200.0,
                min_observation_time_sec: Some(1200.0),
                fixed_start_time: None,
                fixed_stop_time: None,
                ra_in_deg: Some(158.03),
                dec_in_deg: Some(-68.03),
                min_azimuth_angle_in_deg: Some(0.0),
                max_azimuth_angle_in_deg: Some(360.0),
                min_elevation_angle_in_deg: Some(60.0),
                max_elevation_angle_in_deg: Some(90.0),
                scheduled_start: Some(61894.194),
                scheduled_stop: Some(61894.208),
                visibility_periods: vec![],
            },
        ];
        
        let df = blocks_to_dataframe(&blocks).unwrap();
        
        assert_eq!(df.height(), 1);
        let col_names = df.get_column_names();
        assert!(col_names.iter().any(|s| s.as_str() == "schedulingBlockId"));
        assert!(col_names.iter().any(|s| s.as_str() == "priority"));
        assert!(col_names.iter().any(|s| s.as_str() == "scheduled_flag"));
        
        // Check values
        let ids = df.column("schedulingBlockId").unwrap().str().unwrap();
        assert_eq!(ids.get(0), Some("1000004990"));
        
        let priorities = df.column("priority").unwrap().f64().unwrap();
        assert_eq!(priorities.get(0), Some(8.5));
        
        let scheduled = df.column("scheduled_flag").unwrap().bool().unwrap();
        assert_eq!(scheduled.get(0), Some(true));
    }
}
