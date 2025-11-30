use crate::db::models::{LightweightBlock, PriorityBinInfo, SkyMapData};
use crate::db::operations;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

/// Compute sky map data with priority bins and metadata.
/// This function takes the raw blocks and computes everything needed for visualization.
pub fn compute_sky_map_data(blocks: Vec<LightweightBlock>) -> Result<SkyMapData, String> {
    if blocks.is_empty() {
        return Ok(SkyMapData {
            blocks: vec![],
            priority_bins: vec![],
            priority_min: 0.0,
            priority_max: 10.0,
            ra_min: 0.0,
            ra_max: 360.0,
            dec_min: -90.0,
            dec_max: 90.0,
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
        ra_min = ra_min.min(block.target_ra_deg);
        ra_max = ra_max.max(block.target_ra_deg);
        dec_min = dec_min.min(block.target_dec_deg);
        dec_max = dec_max.max(block.target_dec_deg);

        if let Some(period) = &block.scheduled_period {
            scheduled_count += 1;
            let start_mjd = period.start.value();
            scheduled_time_min = Some(scheduled_time_min.map_or(start_mjd, |v| v.min(start_mjd)));
            scheduled_time_max = Some(scheduled_time_max.map_or(start_mjd, |v| v.max(start_mjd)));
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

        priority_bins.push(PriorityBinInfo {
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
        ra_min,
        ra_max,
        dec_min,
        dec_max,
        total_count,
        scheduled_count,
        scheduled_time_min,
        scheduled_time_max,
    })
}

/// Get complete sky map data with computed bins and metadata.
/// This function orchestrates fetching blocks from the database and computing the sky map data.
pub async fn get_sky_map_data(schedule_id: i64) -> Result<SkyMapData, String> {
    let blocks = operations::fetch_lightweight_blocks(schedule_id).await?;
    compute_sky_map_data(blocks)
}

/// Get complete sky map data with computed bins and metadata.
/// This is the main function for the sky map feature, computing priority bins
/// and all necessary metadata on the Rust side.
#[pyfunction]
pub fn py_get_sky_map_data(schedule_id: i64) -> PyResult<SkyMapData> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(get_sky_map_data(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}
