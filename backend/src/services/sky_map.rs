#![allow(clippy::needless_range_loop)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::useless_vec)]

use crate::api::{LightweightBlock, SkyMapData};
use tokio::runtime::Runtime;

// Import the global repository accessor
use crate::db::get_repository;

/// Compute sky map data with priority bins and metadata.
/// This function takes the raw blocks and computes everything needed for visualization.
pub fn compute_sky_map_data(blocks: Vec<LightweightBlock>) -> Result<SkyMapData, String> {
    if blocks.is_empty() {
        return Ok(SkyMapData {
            blocks: vec![],
            priority_bins: vec![],
            priority_min: 0.0,
            priority_max: 10.0,
            ra_min: qtty::Degrees::new(0.0),
            ra_max: qtty::Degrees::new(360.0),
            dec_min: qtty::Degrees::new(-90.0),
            dec_max: qtty::Degrees::new(90.0),
            total_count: 0,
            scheduled_count: 0,
            scheduled_time_min: None,
            scheduled_time_max: None,
        });
    }

    // Compute statistics
    let mut priority_min = f64::MAX;
    let mut priority_max = f64::MIN;
    let mut ra_min = f64::MAX;
    let mut ra_max = f64::MIN;
    let mut dec_min = f64::MAX;
    let mut dec_max = f64::MIN;
    let mut scheduled_count = 0;
    let mut scheduled_time_min: Option<f64> = None;
    let mut scheduled_time_max: Option<f64> = None;

    for block in &blocks {
        priority_min = priority_min.min(block.priority);
        priority_max = priority_max.max(block.priority);
        ra_min = ra_min.min(block.target_ra_deg.value());
        ra_max = ra_max.max(block.target_ra_deg.value());
        dec_min = dec_min.min(block.target_dec_deg.value());
        dec_max = dec_max.max(block.target_dec_deg.value());

        if let Some(period) = &block.scheduled_period {
            scheduled_count += 1;
            let start_val = period.start.value();
            scheduled_time_min = Some(scheduled_time_min.map_or(start_val, |v| v.min(start_val)));
            scheduled_time_max = Some(scheduled_time_max.map_or(start_val, |v| v.max(start_val)));
        }
    }

    // Ensure priority range is valid
    if priority_min == priority_max {
        priority_max = priority_min + 1.0;
    }

    // Compute 4 priority bins proportional to min/max values
    let bin_count = 4;
    let bin_width = (priority_max - priority_min) / bin_count as f64;

    // Define colors for the 4 bins (from low to high priority)
    let bin_colors = vec![
        "#2ca02c".to_string(), // Green - Low priority
        "#1f77b4".to_string(), // Blue - Medium-low priority
        "#ff7f0e".to_string(), // Orange - Medium-high priority
        "#d62728".to_string(), // Red - High priority
    ];

    let mut priority_bins = Vec::with_capacity(bin_count);
    for i in 0..bin_count {
        let bin_min = priority_min + (i as f64 * bin_width);
        let bin_max = if i == bin_count - 1 {
            priority_max
        } else {
            priority_min + ((i + 1) as f64 * bin_width)
        };

        let label = format!("Bin {} [{:.1}-{:.1}]", i + 1, bin_min, bin_max);

        priority_bins.push(crate::api::PriorityBinInfo {
            label,
            min_priority: bin_min,
            max_priority: bin_max,
            color: bin_colors[i].clone(),
        });
    }

    // Assign computed bins to blocks
    let total_count = blocks.len();
    let mut blocks_with_bins = blocks;
    for block in &mut blocks_with_bins {
        // Find which bin this block belongs to
        let priority = block.priority;
        let bin_index = if priority >= priority_max {
            bin_count - 1
        } else {
            ((priority - priority_min) / bin_width).floor() as usize
        };
        block.priority_bin = format!(
            "Bin {} [{:.1}-{:.1}]",
            bin_index + 1,
            priority_bins[bin_index].min_priority,
            priority_bins[bin_index].max_priority
        );
    }

    Ok(SkyMapData {
        blocks: blocks_with_bins,
        priority_bins,
        priority_min,
        priority_max,
        ra_min: qtty::Degrees::new(ra_min),
        ra_max: qtty::Degrees::new(ra_max),
        dec_min: qtty::Degrees::new(dec_min),
        dec_max: qtty::Degrees::new(dec_max),
        total_count,
        scheduled_count,
        scheduled_time_min,
        scheduled_time_max,
    })
}

/// Get complete sky map data with computed bins and metadata using ETL analytics.
///
/// This function retrieves blocks from the analytics repository
/// which contains pre-computed, denormalized data for optimal performance.
pub async fn get_sky_map_data(schedule_id: crate::api::ScheduleId) -> Result<SkyMapData, String> {
    // Get the initialized repository
    let repo = get_repository().map_err(|e| format!("Failed to get repository: {}", e))?;

    let blocks = repo
        .fetch_analytics_blocks_for_sky_map(schedule_id)
        .await
        .map_err(|e| format!("Failed to fetch analytics blocks: {}", e))?;

    if blocks.is_empty() {
        return Err(format!(
            "No analytics data available for schedule_id={}. Run populate_schedule_analytics() first.",
            schedule_id
        ));
    }
    compute_sky_map_data(blocks)
}

/// Get complete sky map data with computed bins and metadata.
pub fn py_get_sky_map_data(schedule_id: crate::api::ScheduleId) -> Result<SkyMapData, String> {
    let runtime = Runtime::new().map_err(|e| {
        format!("Failed to create async runtime: {}", e)
    })?;

    runtime
        .block_on(get_sky_map_data(schedule_id))
}

/// Alias for compatibility - uses analytics path.
pub fn py_get_sky_map_data_analytics(schedule_id: crate::api::ScheduleId) -> Result<SkyMapData, String> {
    py_get_sky_map_data(schedule_id)
}

#[cfg(test)]
mod tests {
    use super::compute_sky_map_data;
    use crate::api::LightweightBlock;

    fn create_test_block(
        id: &str,
        priority: f64,
        ra: f64,
        dec: f64,
        scheduled: bool,
    ) -> LightweightBlock {
        LightweightBlock {
            original_block_id: id.to_string(),
            priority,
            priority_bin: String::new(),
            requested_duration_seconds: qtty::Seconds::new(3600.0),
            target_ra_deg: qtty::Degrees::new(ra),
            target_dec_deg: qtty::Degrees::new(dec),
            scheduled_period: if scheduled {
                Some(crate::api::Period {
                    start: crate::api::ModifiedJulianDate::new(1000.0),
                    stop: crate::api::ModifiedJulianDate::new(2000.0),
                })
            } else {
                None
            },
        }
    }

    #[test]
    fn test_compute_sky_map_data_empty() {
        let result = compute_sky_map_data(vec![]);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.blocks.len(), 0);
        assert_eq!(data.priority_bins.len(), 0);
        assert_eq!(data.priority_min, 0.0);
        assert_eq!(data.priority_max, 10.0);
        assert_eq!(data.ra_min.value(), 0.0);
        assert_eq!(data.ra_max.value(), 360.0);
        assert_eq!(data.dec_min.value(), -90.0);
        assert_eq!(data.dec_max.value(), 90.0);
        assert_eq!(data.total_count, 0);
        assert_eq!(data.scheduled_count, 0);
        assert!(data.scheduled_time_min.is_none());
        assert!(data.scheduled_time_max.is_none());
    }

    #[test]
    fn test_compute_sky_map_data_single_block() {
        let blocks = vec![create_test_block("b1", 5.0, 180.0, 45.0, false)];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.blocks.len(), 1);
        assert_eq!(data.priority_bins.len(), 4);
        assert_eq!(data.priority_min, 5.0);
        assert_eq!(data.priority_max, 6.0); // Adjusted when min == max
        assert_eq!(data.total_count, 1);
        assert_eq!(data.scheduled_count, 0);
    }

    #[test]
    fn test_compute_sky_map_data_priority_range() {
        let blocks = vec![
            create_test_block("b1", 2.0, 0.0, -30.0, false),
            create_test_block("b2", 8.0, 120.0, 60.0, true),
            create_test_block("b3", 5.0, 240.0, 0.0, false),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.priority_min, 2.0);
        assert_eq!(data.priority_max, 8.0);
        assert_eq!(data.priority_bins.len(), 4);

        // Check bin properties
        let bin_width = (8.0 - 2.0) / 4.0;
        assert_eq!(data.priority_bins[0].min_priority, 2.0);
        assert_eq!(data.priority_bins[0].max_priority, 2.0 + bin_width);
        assert_eq!(data.priority_bins[3].max_priority, 8.0);
    }

    #[test]
    fn test_compute_sky_map_data_bin_assignment() {
        let blocks = vec![
            create_test_block("low", 1.0, 0.0, 0.0, false),
            create_test_block("high", 9.0, 90.0, 30.0, false),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        // Check that blocks have priority_bin assigned
        assert!(data.blocks[0].priority_bin.starts_with("Bin"));
        assert!(data.blocks[1].priority_bin.starts_with("Bin"));

        // Low priority should be in bin 1, high priority in bin 4
        assert!(data.blocks[0].priority_bin.contains("Bin 1"));
        assert!(data.blocks[1].priority_bin.contains("Bin 4"));
    }

    #[test]
    fn test_compute_sky_map_data_scheduled_tracking() {
        let blocks = vec![
            create_test_block("b1", 5.0, 0.0, 0.0, true),
            create_test_block("b2", 7.0, 90.0, 30.0, false),
            create_test_block("b3", 9.0, 180.0, -30.0, true),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.scheduled_count, 2);
        assert!(data.scheduled_time_min.is_some());
        assert!(data.scheduled_time_max.is_some());
        assert_eq!(data.scheduled_time_min.unwrap(), 1000.0);
        assert_eq!(data.scheduled_time_max.unwrap(), 1000.0);
    }

    #[test]
    fn test_compute_sky_map_data_ra_dec_ranges() {
        let blocks = vec![
            create_test_block("b1", 5.0, 30.0, -80.0, false),
            create_test_block("b2", 5.0, 270.0, 70.0, false),
            create_test_block("b3", 5.0, 150.0, 10.0, false),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.ra_min.value(), 30.0);
        assert_eq!(data.ra_max.value(), 270.0);
        assert_eq!(data.dec_min.value(), -80.0);
        assert_eq!(data.dec_max.value(), 70.0);
    }

    #[test]
    fn test_compute_sky_map_data_bin_colors() {
        let blocks = vec![create_test_block("b1", 5.0, 0.0, 0.0, false)];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        // Check that all 4 bins have colors assigned
        assert_eq!(data.priority_bins.len(), 4);
        assert_eq!(data.priority_bins[0].color, "#2ca02c"); // Green
        assert_eq!(data.priority_bins[1].color, "#1f77b4"); // Blue
        assert_eq!(data.priority_bins[2].color, "#ff7f0e"); // Orange
        assert_eq!(data.priority_bins[3].color, "#d62728"); // Red
    }

    #[test]
    fn test_compute_sky_map_data_edge_priority_goes_to_last_bin() {
        // When priority equals priority_max, it should go to the last bin
        let blocks = vec![
            create_test_block("b1", 0.0, 0.0, 0.0, false),
            create_test_block("b2", 10.0, 90.0, 30.0, false),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        // Block with max priority should be in the last bin
        let max_priority_block = data.blocks.iter().find(|b| b.priority == 10.0).unwrap();
        assert!(max_priority_block.priority_bin.contains("Bin 4"));
    }

    #[test]
    fn test_compute_sky_map_data_boundary_values() {
        let blocks = vec![
            create_test_block("b1", 0.0, 0.0, -90.0, false),
            create_test_block("b2", 10.0, 360.0, 90.0, false),
        ];
        let result = compute_sky_map_data(blocks);

        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.ra_min.value(), 0.0);
        assert_eq!(data.ra_max.value(), 360.0);
        assert_eq!(data.dec_min.value(), -90.0);
        assert_eq!(data.dec_max.value(), 90.0);
    }
}
