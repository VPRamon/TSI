// ! Backend visibility computation service.
//!
//! Computes per-block visibility periods from observer location, schedule period,
//! target RA/Dec, altitude/azimuth constraints, optional fixed_time window,
//! optional astronomical-night windows, and minimum duration filter.
//!
//! This is the fallback path used when `possible_periods` is absent from the
//! incoming JSON payload.

use qtty::{Degrees, Seconds};
use siderust::calculus::altitude::{AltitudePeriodsProvider, AltitudeQuery};
use siderust::calculus::azimuth::{AzimuthProvider, AzimuthQuery};
use siderust::coordinates::spherical::direction;
use siderust::time::intersect_periods;

use crate::api::{Constraints, GeographicLocation, ModifiedJulianDate, Period};

/// Input parameters for a single block's visibility computation.
pub struct VisibilityInput<'a> {
    pub location: &'a GeographicLocation,
    pub schedule_period: &'a Period,
    pub target_ra: Degrees,
    pub target_dec: Degrees,
    pub constraints: &'a Constraints,
    pub min_duration: Seconds,
    pub astronomical_nights: Option<&'a [Period]>,
}

/// Compute visibility periods for a single block.
///
/// Returns intervals within `schedule_period` (intersected with `fixed_time` and
/// `astronomical_nights` if present) where the target satisfies the altitude
/// constraints and the duration is at least `min_duration`.
///
pub fn compute_block_visibility(input: &VisibilityInput<'_>) -> Vec<Period> {
    let site = *input.location;

    // Determine effective search window: schedule_period clipped to fixed_time.
    let window: Period = match &input.constraints.fixed_time {
        Some(fixed) => {
            let start = input.schedule_period.start.value().max(fixed.start.value());
            let end = input.schedule_period.end.value().min(fixed.end.value());
            if start >= end {
                return Vec::new();
            }
            Period::new(ModifiedJulianDate::new(start), ModifiedJulianDate::new(end))
        }
        None => *input.schedule_period,
    };

    // Build the altitude query with the block's altitude constraints.
    let query = AltitudeQuery {
        observer: site,
        window,
        min_altitude: input.constraints.min_alt,
        max_altitude: input.constraints.max_alt,
    };

    // Compute altitude-valid periods for this RA/Dec direction.
    let icrs = direction::ICRS::new(input.target_ra, input.target_dec);
    let altitude_intervals = icrs.altitude_periods(&query);

    let min_days = input.min_duration.value() / 86_400.0;

    let position_intervals =
        if is_full_azimuth_range(input.constraints.min_az, input.constraints.max_az) {
            altitude_intervals
        } else {
            let azimuth_query = AzimuthQuery {
                observer: site,
                window,
                min_azimuth: input.constraints.min_az,
                max_azimuth: input.constraints.max_az,
            };
            let azimuth_intervals = icrs.azimuth_periods(&azimuth_query);
            intersect_periods(&altitude_intervals, &azimuth_intervals)
        };

    let constrained_intervals = match input.astronomical_nights {
        Some(nights) => intersect_periods(&position_intervals, nights),
        None => position_intervals,
    };

    constrained_intervals
        .into_iter()
        .filter(|iv| (iv.end.value() - iv.start.value()) >= min_days)
        .collect()
}

fn is_full_azimuth_range(min_az: Degrees, max_az: Degrees) -> bool {
    max_az.value() >= min_az.value() && (max_az.value() - min_az.value()) >= 360.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Constraints, GeographicLocation, ModifiedJulianDate, Period};
    use qtty::{Degrees, Meters};
    use siderust::coordinates::centers::Geodetic;
    use siderust::coordinates::frames::ECEF;

    fn roque_location() -> GeographicLocation {
        Geodetic::<ECEF>::new(
            Degrees::new(-17.8892),
            Degrees::new(28.7624),
            Meters::new(2396.0),
        )
    }

    fn one_week_period() -> Period {
        Period {
            start: ModifiedJulianDate::new(60694.0), // 2026-01-15
            end: ModifiedJulianDate::new(60701.0),   // 2026-01-22
        }
    }

    fn full_sky_constraints() -> Constraints {
        Constraints {
            min_alt: Degrees::new(0.0),
            max_alt: Degrees::new(90.0),
            min_az: Degrees::new(0.0),
            max_az: Degrees::new(360.0),
            fixed_time: None,
        }
    }

    #[test]
    fn test_altitude_only_visibility() {
        // Canopus: RA=95.988°, Dec=-52.696° — visible from Roque
        let input = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &full_sky_constraints(),
            min_duration: Seconds::new(0.0),
            astronomical_nights: None,
        };

        let periods = compute_block_visibility(&input);
        assert!(!periods.is_empty(), "Canopus should be visible from Roque");

        for p in &periods {
            assert!(
                p.start.value() < p.end.value(),
                "Each period must have start < stop"
            );
        }
    }

    #[test]
    fn test_high_altitude_constraint_reduces_periods() {
        // Same target but with high min_alt — should have fewer/shorter periods
        let tight = Constraints {
            min_alt: Degrees::new(60.0),
            max_alt: Degrees::new(90.0),
            min_az: Degrees::new(0.0),
            max_az: Degrees::new(360.0),
            fixed_time: None,
        };

        let input_loose = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &full_sky_constraints(),
            min_duration: Seconds::new(0.0),
            astronomical_nights: None,
        };
        let input_tight = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &tight,
            min_duration: Seconds::new(0.0),
            astronomical_nights: None,
        };

        let loose_total: f64 = compute_block_visibility(&input_loose)
            .iter()
            .map(|p| p.end.value() - p.start.value())
            .sum();
        let tight_total: f64 = compute_block_visibility(&input_tight)
            .iter()
            .map(|p| p.end.value() - p.start.value())
            .sum();

        assert!(
            tight_total <= loose_total,
            "Tight altitude constraint should give ≤ total time vs loose"
        );
    }

    #[test]
    fn test_azimuth_constraint_reduces_periods() {
        let full_az = Constraints {
            min_alt: Degrees::new(-90.0),
            max_alt: Degrees::new(90.0),
            min_az: Degrees::new(0.0),
            max_az: Degrees::new(360.0),
            fixed_time: None,
        };
        let half_az = Constraints {
            min_alt: Degrees::new(-90.0),
            max_alt: Degrees::new(90.0),
            min_az: Degrees::new(270.0),
            max_az: Degrees::new(90.0),
            fixed_time: None,
        };

        let full = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &full_az,
            min_duration: Seconds::new(0.0),
            astronomical_nights: None,
        };
        let constrained = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &half_az,
            min_duration: Seconds::new(0.0),
            astronomical_nights: None,
        };

        let full_total: f64 = compute_block_visibility(&full)
            .iter()
            .map(|p| p.end.value() - p.start.value())
            .sum();
        let constrained_total: f64 = compute_block_visibility(&constrained)
            .iter()
            .map(|p| p.end.value() - p.start.value())
            .sum();

        assert!(
            constrained_total < full_total,
            "Azimuth-constrained total should be less than full azimuth"
        );
    }

    #[test]
    fn test_fixed_time_clips_window() {
        // fixed_time covers only half the week
        let fixed = Constraints {
            min_alt: Degrees::new(0.0),
            max_alt: Degrees::new(90.0),
            min_az: Degrees::new(0.0),
            max_az: Degrees::new(360.0),
            fixed_time: Some(Period {
                start: ModifiedJulianDate::new(60694.0),
                end: ModifiedJulianDate::new(60697.0), // 3 days only
            }),
        };

        let full = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &full_sky_constraints(),
            min_duration: Seconds::new(0.0),
            astronomical_nights: None,
        };
        let clipped = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &fixed,
            min_duration: Seconds::new(0.0),
            astronomical_nights: None,
        };

        let full_periods = compute_block_visibility(&full);
        let clipped_periods = compute_block_visibility(&clipped);

        // All clipped periods must lie within the fixed window
        for p in &clipped_periods {
            assert!(p.start.value() >= 60694.0);
            assert!(p.end.value() <= 60697.0);
        }

        let full_total: f64 = full_periods
            .iter()
            .map(|p| p.end.value() - p.start.value())
            .sum();
        let clipped_total: f64 = clipped_periods
            .iter()
            .map(|p| p.end.value() - p.start.value())
            .sum();
        assert!(
            clipped_total <= full_total,
            "Clipped total must be ≤ full total"
        );
    }

    #[test]
    fn test_fixed_time_outside_schedule_returns_empty() {
        let outside = Constraints {
            min_alt: Degrees::new(0.0),
            max_alt: Degrees::new(90.0),
            min_az: Degrees::new(0.0),
            max_az: Degrees::new(360.0),
            fixed_time: Some(Period {
                start: ModifiedJulianDate::new(60800.0), // outside week window
                end: ModifiedJulianDate::new(60801.0),
            }),
        };

        let input = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &outside,
            min_duration: Seconds::new(0.0),
            astronomical_nights: None,
        };

        let periods = compute_block_visibility(&input);
        assert!(
            periods.is_empty(),
            "No periods expected when fixed_time is outside schedule window"
        );
    }

    #[test]
    fn test_duration_filter() {
        // Filter out periods shorter than 1 hour (3600 seconds)
        let input_no_filter = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &full_sky_constraints(),
            min_duration: Seconds::new(0.0),
            astronomical_nights: None,
        };
        let input_filtered = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &full_sky_constraints(),
            min_duration: Seconds::new(3600.0),
            astronomical_nights: None,
        };

        let unfiltered = compute_block_visibility(&input_no_filter);
        let filtered = compute_block_visibility(&input_filtered);

        // Filtered should have fewer or equal periods
        assert!(filtered.len() <= unfiltered.len());

        // All filtered periods must meet the duration requirement
        let min_days = 3600.0 / 86_400.0;
        for p in &filtered {
            assert!(
                p.end.value() - p.start.value() >= min_days,
                "Period duration must be ≥ min_duration"
            );
        }
    }

    #[test]
    fn test_combined_altitude_and_fixed_time() {
        let constraints = Constraints {
            min_alt: Degrees::new(30.0),
            max_alt: Degrees::new(90.0),
            min_az: Degrees::new(0.0),
            max_az: Degrees::new(360.0),
            fixed_time: Some(Period {
                start: ModifiedJulianDate::new(60695.0),
                end: ModifiedJulianDate::new(60698.0),
            }),
        };

        let input = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &constraints,
            min_duration: Seconds::new(600.0), // 10 minutes
            astronomical_nights: None,
        };

        let periods = compute_block_visibility(&input);

        // All results must lie within fixed_time window
        for p in &periods {
            assert!(p.start.value() >= 60695.0);
            assert!(p.end.value() <= 60698.0);
        }

        // All results must meet duration requirement
        let min_days = 600.0 / 86_400.0;
        for p in &periods {
            assert!(p.end.value() - p.start.value() >= min_days);
        }
    }

    #[test]
    fn test_astronomical_night_clips_visibility() {
        let constraints = Constraints {
            min_alt: Degrees::new(-90.0),
            max_alt: Degrees::new(90.0),
            min_az: Degrees::new(0.0),
            max_az: Degrees::new(360.0),
            fixed_time: Some(Period {
                start: ModifiedJulianDate::new(60694.2),
                end: ModifiedJulianDate::new(60694.8),
            }),
        };
        let nights = vec![Period {
            start: ModifiedJulianDate::new(60694.4),
            end: ModifiedJulianDate::new(60694.6),
        }];

        let input = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &constraints,
            min_duration: Seconds::new(0.0),
            astronomical_nights: Some(&nights),
        };

        let periods = compute_block_visibility(&input);

        assert!(!periods.is_empty());
        for p in &periods {
            assert!(p.start.value() >= 60694.4);
            assert!(p.end.value() <= 60694.6);
        }
    }

    #[test]
    fn test_empty_astronomical_nights_returns_empty_visibility() {
        let constraints = Constraints {
            min_alt: Degrees::new(-90.0),
            max_alt: Degrees::new(90.0),
            min_az: Degrees::new(0.0),
            max_az: Degrees::new(360.0),
            fixed_time: None,
        };
        let nights = Vec::new();

        let input = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &constraints,
            min_duration: Seconds::new(0.0),
            astronomical_nights: Some(&nights),
        };

        let periods = compute_block_visibility(&input);
        assert!(periods.is_empty());
    }
}

// =====================================================================
// Visibility histogram (formerly services::visibility)
//
// Bins per-block visibility periods into a time histogram for the UI.
// Operates on raw DB rows to avoid extra JSONB deserialization.
// =====================================================================

mod histogram {
    use std::collections::HashSet;

    use crate::db::models::{BlockHistogramData, VisibilityBin};

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
        priority_min: Option<f64>,
        priority_max: Option<f64>,
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
                    all_periods.push(VisibilityPeriod {
                        block_id: block.scheduling_block_id,
                        start_unix: ((period.start.value() - 40587.0) * 86400.0) as i64,
                        end_unix: ((period.end.value() - 40587.0) * 86400.0) as i64,
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

            bin.visible_count = visible_blocks.len() as i64;
        }

        Ok(bins)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_mjd_unix_conversion() {
            // MJD 40587 = 1970-01-01 00:00:00 UTC (Unix epoch)
            let mjd_to_unix = |mjd: f64| -> i64 { ((mjd - 40587.0) * 86400.0) as i64 };
            assert_eq!(mjd_to_unix(40587.0), 0);

            // Round trip: unix → mjd → unix should be close
            let mjd = 59000.5;
            let unix = mjd_to_unix(mjd);
            let back = unix as f64 / 86400.0 + 40587.0;
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
            use crate::api::Period;
            use crate::models::ModifiedJulianDate;

            let block = BlockHistogramData {
                scheduling_block_id: 1,
                priority: 5.0,
                visibility_periods: Some(vec![Period {
                    start: ModifiedJulianDate::new(40587.0),
                    end: ModifiedJulianDate::new(40587.5),
                }]),
            };

            // Unix epoch (MJD 40587) to +1 day
            let bins = compute_visibility_histogram_rust(
                vec![block].into_iter(),
                0,
                86400,
                3600,
                None,
                None,
            )
            .unwrap();

            assert_eq!(bins.len(), 24);
            // First 12 hours should have visibility
            let visible_bins = bins.iter().filter(|b| b.visible_count > 0).count();
            assert!(visible_bins > 0);
        }

        #[test]
        fn test_compute_histogram_priority_filter() {
            use crate::api::Period;
            use crate::models::ModifiedJulianDate;

            let blocks = vec![
                BlockHistogramData {
                    scheduling_block_id: 1,
                    priority: 3.0,
                    visibility_periods: Some(vec![Period {
                        start: ModifiedJulianDate::new(40587.0),
                        end: ModifiedJulianDate::new(40587.5),
                    }]),
                },
                BlockHistogramData {
                    scheduling_block_id: 2,
                    priority: 7.0,
                    visibility_periods: Some(vec![Period {
                        start: ModifiedJulianDate::new(40587.0),
                        end: ModifiedJulianDate::new(40587.5),
                    }]),
                },
            ];

            // Filter for priority >= 5
            let bins = compute_visibility_histogram_rust(
                blocks.into_iter(),
                0,
                86400,
                3600,
                Some(5.0),
                None,
            )
            .unwrap();

            // Only block 2 (priority 7) should be counted
            let max_count = bins.iter().map(|b| b.visible_count).max().unwrap();
            assert_eq!(max_count, 1);
        }

        #[test]
        fn test_compute_histogram_overlapping_periods() {
            use crate::api::Period;
            use crate::models::ModifiedJulianDate;

            // Same block with multiple overlapping periods in same bin
            let block = BlockHistogramData {
                scheduling_block_id: 1,
                priority: 5.0,
                visibility_periods: Some(vec![
                    Period {
                        start: ModifiedJulianDate::new(40587.0),
                        end: ModifiedJulianDate::new(40587.1),
                    },
                    Period {
                        start: ModifiedJulianDate::new(40587.05),
                        end: ModifiedJulianDate::new(40587.15),
                    },
                ]),
            };

            let bins = compute_visibility_histogram_rust(
                vec![block].into_iter(),
                0,
                86400,
                3600,
                None,
                None,
            )
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
                compute_visibility_histogram_rust(blocks.into_iter(), 0, 100, 0, None, None)
                    .is_err()
            );
        }
    }
}
pub use histogram::compute_visibility_histogram_rust;
