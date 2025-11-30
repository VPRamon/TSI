//! Visibility histogram computation service.
//!
//! This module provides efficient computation of visibility histograms for scheduling blocks.
//! It processes visibility periods stored as JSON and bins them into time intervals,
//! counting unique blocks visible in each bin.
//!
//! ## Performance considerations
//! - Streams data from database to minimize memory usage
//! - Uses integer arithmetic for bin indexing
//! - Counts unique blocks per bin (not duplicate periods from same block)
//! - Handles edge cases: periods spanning bin boundaries, zero-length ranges

use std::collections::HashSet;

use crate::db::models::{BlockHistogramData, VisibilityBin};

/// MJD epoch (1858-11-17 00:00:00 UTC) as Unix timestamp
const MJD_EPOCH_UNIX: i64 = -3506716800;

/// Convert Modified Julian Date to Unix timestamp (seconds since 1970-01-01)
#[inline]
fn mjd_to_unix(mjd: f64) -> i64 {
    MJD_EPOCH_UNIX + (mjd * 86400.0) as i64
}

/// A parsed visibility period with Unix timestamps for efficient comparison
#[derive(Debug, Clone, Copy)]
struct VisibilityPeriod {
    start_unix: i64,
    end_unix: i64,
    block_id: i64,
}

/// Compute visibility histogram from database rows.
///
/// This function:
/// 1. Parses visibility periods JSON from each block
/// 2. Bins periods into time intervals
/// 3. Counts unique blocks visible in each bin
///
/// ## Arguments
/// * `blocks` - Iterator of database rows with visibility JSON
/// * `start_unix` - Start of histogram range (Unix timestamp)
/// * `end_unix` - End of histogram range (Unix timestamp)
/// * `bin_duration_seconds` - Duration of each bin in seconds
/// * `priority_min` - Optional minimum priority filter (inclusive)
/// * `priority_max` - Optional maximum priority filter (inclusive)
///
/// ## Returns
/// Vector of bins with start/end timestamps and visible block counts
///
/// ## Edge cases
/// - Periods touching bin boundaries: counted if overlap exists (start < bin_end && end > bin_start)
/// - Empty visibility: returns zero counts
/// - Invalid JSON: logs warning and skips block
/// - Same block visible multiple times in bin: counted once
pub fn compute_visibility_histogram_rust(
    blocks: impl Iterator<Item = BlockHistogramData>,
    start_unix: i64,
    end_unix: i64,
    bin_duration_seconds: i64,
    priority_min: Option<i32>,
    priority_max: Option<i32>,
) -> Result<Vec<VisibilityBin>, String> {
    // Validate inputs
    if start_unix >= end_unix {
        return Err("start_unix must be less than end_unix".to_string());
    }
    if bin_duration_seconds <= 0 {
        return Err("bin_duration_seconds must be positive".to_string());
    }

    // Calculate number of bins
    let time_range = end_unix - start_unix;
    let num_bins = ((time_range + bin_duration_seconds - 1) / bin_duration_seconds) as usize;

    // Initialize bins
    let mut bins: Vec<VisibilityBin> = (0..num_bins)
        .map(|i| {
            let bin_start = start_unix + (i as i64) * bin_duration_seconds;
            let bin_end = std::cmp::min(bin_start + bin_duration_seconds, end_unix);
            VisibilityBin::new(bin_start, bin_end, 0)
        })
        .collect();

    // Parse visibility periods from all blocks
    let mut all_periods: Vec<VisibilityPeriod> = Vec::new();

    for block in blocks {
        // Apply priority filter
        if let Some(min_p) = priority_min {
            if block.priority < min_p {
                continue;
            }
        }
        if let Some(max_p) = priority_max {
            if block.priority > max_p {
                continue;
            }
        }

        // Parse visibility periods JSON
        if let Some(json_str) = &block.visibility_periods_json {
            match parse_visibility_periods_json(json_str, block.scheduling_block_id) {
                Ok(periods) => all_periods.extend(periods),
                Err(e) => {
                    log::warn!(
                        "Failed to parse visibility for block {}: {}",
                        block.scheduling_block_id,
                        e
                    );
                }
            }
        }
    }

    // Count unique blocks per bin using HashSet for deduplication
    for bin in bins.iter_mut() {
        let mut visible_blocks: HashSet<i64> = HashSet::new();

        for period in &all_periods {
            // Check if period overlaps with bin
            if period.start_unix < bin.bin_end_unix && period.end_unix > bin.bin_start_unix {
                visible_blocks.insert(period.block_id);
            }
        }

        bin.visible_count = visible_blocks.len() as u32;
    }

    Ok(bins)
}

/// Parse visibility periods JSON array into VisibilityPeriod structs.
///
/// Expected JSON format: [{"start": mjd_float, "stop": mjd_float}, ...]
///
/// ## Arguments
/// * `json_str` - JSON string to parse
/// * `block_id` - Scheduling block ID for tracking
///
/// ## Returns
/// Vector of parsed periods with Unix timestamps
fn parse_visibility_periods_json(
    json_str: &str,
    block_id: i64,
) -> Result<Vec<VisibilityPeriod>, String> {
    let periods_array: Vec<serde_json::Value> =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {}", e))?;

    let mut periods = Vec::with_capacity(periods_array.len());

    for period_obj in periods_array {
        let start_mjd = period_obj["start"]
            .as_f64()
            .ok_or_else(|| "Missing or invalid 'start' field".to_string())?;

        let stop_mjd = period_obj["stop"]
            .as_f64()
            .ok_or_else(|| "Missing or invalid 'stop' field".to_string())?;

        // Convert MJD to Unix timestamps
        let start_unix = mjd_to_unix(start_mjd);
        let end_unix = mjd_to_unix(stop_mjd);

        // Validate period
        if start_unix < end_unix {
            periods.push(VisibilityPeriod {
                start_unix,
                end_unix,
                block_id,
            });
        } else {
            log::warn!(
                "Invalid period for block {}: start {} >= end {}",
                block_id,
                start_mjd,
                stop_mjd
            );
        }
    }

    Ok(periods)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to convert Unix timestamp to MJD for round-trip tests
    #[inline]
    fn unix_to_mjd(unix: i64) -> f64 {
        ((unix - MJD_EPOCH_UNIX) as f64) / 86400.0
    }

    #[test]
    fn test_mjd_unix_conversion() {
        // MJD 0 = 1858-11-17 00:00:00 UTC
        assert_eq!(mjd_to_unix(0.0), MJD_EPOCH_UNIX);

        // MJD 40587 = 1970-01-01 00:00:00 UTC (Unix epoch)
        assert_eq!(mjd_to_unix(40587.0), 0);

        // Round trip
        let mjd = 59000.5;
        let unix = mjd_to_unix(mjd);
        let back = unix_to_mjd(unix);
        assert!((back - mjd).abs() < 0.0001);
    }

    #[test]
    fn test_parse_visibility_periods_json() {
        let json = r#"[
            {"start": 59000.0, "stop": 59000.5},
            {"start": 59001.0, "stop": 59001.25}
        ]"#;

        let periods = parse_visibility_periods_json(json, 123).unwrap();
        assert_eq!(periods.len(), 2);
        assert_eq!(periods[0].block_id, 123);
        assert_eq!(periods[1].block_id, 123);
    }

    #[test]
    fn test_parse_visibility_invalid_json() {
        let json = r#"not valid json"#;
        assert!(parse_visibility_periods_json(json, 123).is_err());
    }

    #[test]
    fn test_compute_histogram_empty() {
        let blocks: Vec<BlockHistogramData> = vec![];
        let bins =
            compute_visibility_histogram_rust(blocks.into_iter(), 0, 86400, 3600, None, None)
                .unwrap();

        assert_eq!(bins.len(), 24); // 24 hours
        assert!(bins.iter().all(|b| b.visible_count == 0));
    }

    #[test]
    fn test_compute_histogram_single_block() {
        let block = BlockHistogramData {
            scheduling_block_id: 1,
            priority: 5,
            visibility_periods_json: Some(r#"[{"start": 40587.0, "stop": 40587.5}]"#.to_string()),
        };

        // Unix epoch (MJD 40587) to +1 day
        let bins =
            compute_visibility_histogram_rust(vec![block].into_iter(), 0, 86400, 3600, None, None)
                .unwrap();

        assert_eq!(bins.len(), 24);
        // First 12 hours should have visibility
        let visible_bins = bins.iter().filter(|b| b.visible_count > 0).count();
        assert!(visible_bins > 0);
    }

    #[test]
    fn test_compute_histogram_priority_filter() {
        let blocks = vec![
            BlockHistogramData {
                scheduling_block_id: 1,
                priority: 3,
                visibility_periods_json: Some(
                    r#"[{"start": 40587.0, "stop": 40587.5}]"#.to_string(),
                ),
            },
            BlockHistogramData {
                scheduling_block_id: 2,
                priority: 7,
                visibility_periods_json: Some(
                    r#"[{"start": 40587.0, "stop": 40587.5}]"#.to_string(),
                ),
            },
        ];

        // Filter for priority >= 5
        let bins =
            compute_visibility_histogram_rust(blocks.into_iter(), 0, 86400, 3600, Some(5), None)
                .unwrap();

        // Only block 2 (priority 7) should be counted
        let max_count = bins.iter().map(|b| b.visible_count).max().unwrap();
        assert_eq!(max_count, 1);
    }

    #[test]
    fn test_compute_histogram_overlapping_periods() {
        // Same block with multiple overlapping periods in same bin
        let block = BlockHistogramData {
            scheduling_block_id: 1,
            priority: 5,
            visibility_periods_json: Some(
                r#"[
                    {"start": 40587.0, "stop": 40587.1},
                    {"start": 40587.05, "stop": 40587.15}
                ]"#
                .to_string(),
            ),
        };

        let bins =
            compute_visibility_histogram_rust(vec![block].into_iter(), 0, 86400, 3600, None, None)
                .unwrap();

        // Even with overlapping periods, block should be counted once per bin
        let visible_bins: Vec<_> = bins.iter().filter(|b| b.visible_count > 0).collect();
        assert!(visible_bins.iter().all(|b| b.visible_count <= 1));
    }

    #[test]
    fn test_compute_histogram_validation() {
        let blocks: Vec<BlockHistogramData> = vec![];

        // Invalid: start >= end
        assert!(compute_visibility_histogram_rust(
            blocks.clone().into_iter(),
            100,
            50,
            3600,
            None,
            None
        )
        .is_err());

        // Invalid: zero bin duration
        assert!(
            compute_visibility_histogram_rust(blocks.into_iter(), 0, 100, 0, None, None).is_err()
        );
    }
}
