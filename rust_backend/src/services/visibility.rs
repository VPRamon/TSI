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

/// Represents a single time bin in a visibility histogram.
/// Used internally in Rust for efficient computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibilityBin {
    /// Start of the bin as Unix timestamp (seconds since epoch)
    pub bin_start_unix: i64,
    /// End of the bin as Unix timestamp (seconds since epoch)
    pub bin_end_unix: i64,
    /// Number of unique scheduling blocks visible in this bin
    pub visible_count: u32,
}

impl VisibilityBin {
    /// Create a new visibility bin
    pub fn new(bin_start_unix: i64, bin_end_unix: i64, visible_count: u32) -> Self {
        Self {
            bin_start_unix,
            bin_end_unix,
            visible_count,
        }
    }

    /// Check if a time period (in Unix timestamps) overlaps with this bin
    pub fn overlaps_period(&self, period_start_unix: i64, period_end_unix: i64) -> bool {
        period_start_unix < self.bin_end_unix && period_end_unix > self.bin_start_unix
    }
}

/// A row from the database containing minimal data needed for histogram computation
#[derive(Debug, Clone)]
pub struct BlockHistogramData {
    /// Scheduling block ID
    pub scheduling_block_id: i64,
    /// Priority of the block
    pub priority: i32,
    /// Visibility periods for this block
    pub visibility_periods: Option<Vec<crate::api::Period>>,
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

        // Use typed visibility periods directly
        if let Some(periods) = &block.visibility_periods {
            // Convert Period to VisibilityPeriod with Unix timestamps
            for period in periods {
                let start_unix = mjd_to_unix(period.start.value());
                let end_unix = mjd_to_unix(period.stop.value());
                all_periods.push(VisibilityPeriod {
                    block_id: block.scheduling_block_id,
                    start_unix,
                    end_unix,
                });
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
        use crate::api::ModifiedJulianDate;
        use crate::api::Period;

        let block = BlockHistogramData {
            scheduling_block_id: 1,
            priority: 5,
            visibility_periods: Some(vec![Period {
                start: ModifiedJulianDate::new(40587.0),
                stop: ModifiedJulianDate::new(40587.5),
            }]),
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
        use crate::api::ModifiedJulianDate;
        use crate::api::Period;

        let blocks = vec![
            BlockHistogramData {
                scheduling_block_id: 1,
                priority: 3,
                visibility_periods: Some(vec![Period {
                    start: ModifiedJulianDate::new(40587.0),
                    stop: ModifiedJulianDate::new(40587.5),
                }]),
            },
            BlockHistogramData {
                scheduling_block_id: 2,
                priority: 7,
                visibility_periods: Some(vec![Period {
                    start: ModifiedJulianDate::new(40587.0),
                    stop: ModifiedJulianDate::new(40587.5),
                }]),
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
        use crate::api::ModifiedJulianDate;
        use crate::api::Period;

        // Same block with multiple overlapping periods in same bin
        let block = BlockHistogramData {
            scheduling_block_id: 1,
            priority: 5,
            visibility_periods: Some(vec![
                Period {
                    start: ModifiedJulianDate::new(40587.0),
                    stop: ModifiedJulianDate::new(40587.1),
                },
                Period {
                    start: ModifiedJulianDate::new(40587.05),
                    stop: ModifiedJulianDate::new(40587.15),
                },
            ]),
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
