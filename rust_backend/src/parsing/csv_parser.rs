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

    // Get existing column names
    let column_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Cast columns to expected types if they were inferred incorrectly
    let mut lazy_df = df.lazy();

    // schedulingBlockId should be String (may be inferred as i64)
    if column_names.contains(&"schedulingBlockId".to_string()) {
        lazy_df = lazy_df.with_column(col("schedulingBlockId").cast(DataType::String));
    }

    // Numeric columns that should be Float64 (may be inferred as i64 if no decimal point)
    let float_columns = [
        "priority",
        "requestedDurationSec",
        "minObservationTimeInSec",
        "raInDeg",
        "decInDeg",
        "minAzimuthAngleInDeg",
        "maxAzimuthAngleInDeg",
        "minElevationAngleInDeg",
        "maxElevationAngleInDeg",
        "scheduled_period.start",
        "scheduled_period.stop",
    ];

    for col_name in float_columns {
        if column_names.contains(&col_name.to_string()) {
            lazy_df = lazy_df.with_column(
                when(col(col_name).is_not_null())
                    .then(col(col_name).cast(DataType::Float64))
                    .otherwise(lit(NULL).cast(DataType::Float64))
                    .alias(col_name),
            );
        }
    }

    let df = lazy_df
        .collect()
        .context("Failed to cast columns to expected types")?;

    Ok(df)
}

/// Parse CSV and convert to SchedulingBlock structures
pub fn parse_schedule_csv_to_blocks(csv_path: &Path) -> Result<Vec<SchedulingBlock>> {
    let df = parse_schedule_csv(csv_path)?;
    dataframe_to_blocks(&df)
}

/// Convert a Polars DataFrame to SchedulingBlock structures
pub fn dataframe_to_blocks(df: &DataFrame) -> Result<Vec<SchedulingBlock>> {
    use crate::core::domain::Period;
    use siderust::astro::ModifiedJulianDate;
    use siderust::coordinates::spherical::direction::ICRS;
    use siderust::units::{angular::Degrees, time::Seconds};

    let mut blocks = Vec::new();
    let height = df.height();

    // Extract columns
    let ids = df.column("schedulingBlockId")?.str()?;
    let target_ids = df.column("targetId").ok().and_then(|c| c.str().ok());
    let target_names = df.column("targetName").ok().and_then(|c| c.str().ok());
    let priorities = df.column("priority")?.f64()?;
    let requested_durations = df.column("requestedDurationSec")?.f64()?;
    let min_obs_times = df
        .column("minObservationTimeInSec")
        .ok()
        .and_then(|c| c.f64().ok());

    let ra = df.column("raInDeg").ok().and_then(|c| c.f64().ok());
    let dec = df.column("decInDeg").ok().and_then(|c| c.f64().ok());

    let min_az = df
        .column("minAzimuthAngleInDeg")
        .ok()
        .and_then(|c| c.f64().ok());
    let max_az = df
        .column("maxAzimuthAngleInDeg")
        .ok()
        .and_then(|c| c.f64().ok());
    let min_el = df
        .column("minElevationAngleInDeg")
        .ok()
        .and_then(|c| c.f64().ok());
    let max_el = df
        .column("maxElevationAngleInDeg")
        .ok()
        .and_then(|c| c.f64().ok());

    let scheduled_starts = df
        .column("scheduled_period.start")
        .ok()
        .and_then(|c| c.f64().ok());
    let scheduled_stops = df
        .column("scheduled_period.stop")
        .ok()
        .and_then(|c| c.f64().ok());

    let visibility_col = df.column("visibility").ok().and_then(|c| c.str().ok());

    for i in 0..height {
        let id = ids
            .get(i)
            .with_context(|| format!("Missing schedulingBlockId at row {}", i))?
            .to_string();

        let priority = priorities
            .get(i)
            .with_context(|| format!("Missing priority at row {}", i))?;

        let requested_duration = requested_durations
            .get(i)
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
            target_id: target_ids.and_then(|col| col.get(i)).map(|s| s.to_string()),
            target_name: target_names.and_then(|col| col.get(i)).map(|s| s.to_string()),
            priority,
            requested_duration: Seconds::new(requested_duration),
            min_observation_time: Seconds::new(
                min_obs_times.and_then(|col| col.get(i)).unwrap_or(0.0),
            ),
            fixed_time: None, // Not typically in CSV
            coordinates: match (
                ra.and_then(|col| col.get(i)),
                dec.and_then(|col| col.get(i)),
            ) {
                (Some(ra_val), Some(dec_val)) => {
                    Some(ICRS::new(Degrees::new(ra_val), Degrees::new(dec_val)))
                }
                _ => None,
            },
            min_azimuth_angle: min_az.and_then(|col| col.get(i)).map(Degrees::new),
            max_azimuth_angle: max_az.and_then(|col| col.get(i)).map(Degrees::new),
            min_elevation_angle: min_el.and_then(|col| col.get(i)).map(Degrees::new),
            max_elevation_angle: max_el.and_then(|col| col.get(i)).map(Degrees::new),
            scheduled_period: match (
                scheduled_starts.and_then(|col| col.get(i)),
                scheduled_stops.and_then(|col| col.get(i)),
            ) {
                (Some(start), Some(stop)) => Some(Period::new(
                    ModifiedJulianDate::new(start),
                    ModifiedJulianDate::new(stop),
                )),
                _ => None,
            },
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
    let mut target_ids = Vec::with_capacity(n);
    let mut target_names = Vec::with_capacity(n);
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

    let mut fixed_start_times = Vec::with_capacity(n);
    let mut fixed_stop_times = Vec::with_capacity(n);

    let mut visibility_strings = Vec::with_capacity(n);
    let mut num_vis_periods = Vec::with_capacity(n);
    let mut total_vis_hours = Vec::with_capacity(n);
    let mut requested_hours = Vec::with_capacity(n);
    let mut elevation_ranges = Vec::with_capacity(n);
    let mut priority_bins = Vec::with_capacity(n);

    for block in blocks {
        ids.push(block.scheduling_block_id.clone());
        target_ids.push(block.target_id.clone());
        target_names.push(block.target_name.clone());
        priorities.push(block.priority);
        requested_durations.push(block.requested_duration.value());
        min_obs_times.push(Some(block.min_observation_time.value()));

        // Extract RA/Dec from coordinates
        ras.push(block.coordinates.as_ref().map(|c| c.ra().value()));
        decs.push(block.coordinates.as_ref().map(|c| c.dec().value()));

        min_azs.push(block.min_azimuth_angle.map(|a| a.value()));
        max_azs.push(block.max_azimuth_angle.map(|a| a.value()));
        min_els.push(block.min_elevation_angle.map(|a| a.value()));
        max_els.push(block.max_elevation_angle.map(|a| a.value()));

        scheduled_starts.push(block.scheduled_period.as_ref().map(|p| p.start.value()));
        scheduled_stops.push(block.scheduled_period.as_ref().map(|p| p.stop.value()));
        scheduled_flags.push(block.is_scheduled());

        fixed_start_times.push(block.fixed_time.as_ref().map(|p| p.start.value()));
        fixed_stop_times.push(block.fixed_time.as_ref().map(|p| p.stop.value()));

        // Format visibility periods as string: "[(start, stop), ...]"
        let vis_str = format_visibility_periods(&block.visibility_periods);
        visibility_strings.push(vis_str);

        num_vis_periods.push(block.num_visibility_periods() as u32);
        total_vis_hours.push(block.total_visibility_hours().value());
        requested_hours.push(block.requested_duration.value() / 3600.0); // Convert seconds to hours
        elevation_ranges.push(block.elevation_range().map(|r| r.value()));
        priority_bins.push(block.priority_bin().to_string());
    }

    // Create DataFrame with columns in Python's expected order
    let df = df!(
        "schedulingBlockId" => ids,
        "targetId" => target_ids,
        "targetName" => target_names,
        "priority" => priorities,
        "minObservationTimeInSec" => min_obs_times,
        "requestedDurationSec" => requested_durations,
        "fixedStartTime" => fixed_start_times,
        "fixedStopTime" => fixed_stop_times,
        "decInDeg" => decs,
        "raInDeg" => ras,
        "minAzimuthAngleInDeg" => min_azs,
        "maxAzimuthAngleInDeg" => max_azs,
        "minElevationAngleInDeg" => min_els,
        "maxElevationAngleInDeg" => max_els,
        "scheduled_period.start" => scheduled_starts,
        "scheduled_period.stop" => scheduled_stops,
        "visibility" => visibility_strings,
        "num_visibility_periods" => num_vis_periods,
        "total_visibility_hours" => total_vis_hours,
        "priority_bin" => priority_bins,
        "scheduled_flag" => scheduled_flags,
        "requested_hours" => requested_hours,
        "elevation_range_deg" => elevation_ranges,
    )?;

    Ok(df)
}

/// Format visibility periods as a string representation matching Python's format.
/// Returns "[]" for empty, or "[(start1, stop1), (start2, stop2), ...]" for populated.
fn format_visibility_periods(periods: &[crate::core::domain::Period]) -> String {
    if periods.is_empty() {
        return "[]".to_string();
    }

    let tuples: Vec<String> = periods
        .iter()
        .map(|p| format!("({}, {})", p.start.value(), p.stop.value()))
        .collect();

    format!("[{}]", tuples.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::Period;
    use siderust::astro::ModifiedJulianDate;
    use siderust::coordinates::spherical::direction::ICRS;
    use siderust::units::{Degrees, Seconds};

    #[test]
    fn test_blocks_to_dataframe_roundtrip() {
        let blocks = vec![SchedulingBlock {
            scheduling_block_id: "1000004990".to_string(),
            target_id: None,
            target_name: None,
            priority: 8.5,
            requested_duration: Seconds::new(1200.0),
            min_observation_time: Seconds::new(1200.0),
            fixed_time: None,
            coordinates: Some(ICRS::new(Degrees::new(158.03), Degrees::new(-68.03))),
            min_azimuth_angle: Some(Degrees::new(0.0)),
            max_azimuth_angle: Some(Degrees::new(360.0)),
            min_elevation_angle: Some(Degrees::new(60.0)),
            max_elevation_angle: Some(Degrees::new(90.0)),
            scheduled_period: Some(Period::new(
                ModifiedJulianDate::new(61894.194),
                ModifiedJulianDate::new(61894.208),
            )),
            visibility_periods: vec![],
        }];

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
