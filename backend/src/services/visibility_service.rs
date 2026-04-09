// ! Backend visibility computation service.
//!
//! Computes per-block visibility periods from observer location, schedule period,
//! target RA/Dec, altitude/azimuth constraints, optional fixed_time window,
//! optional astronomical-night windows, and minimum duration filter.
//!
//! This is the fallback path used when `possible_periods` is absent from the
//! incoming JSON payload.

use qtty::{Degrees, Meters, Seconds};
use siderust::calculus::altitude::{AltitudePeriodsProvider, AltitudeQuery};
use siderust::calculus::azimuth::{AzimuthProvider, AzimuthQuery};
use siderust::coordinates::centers::Geodetic;
use siderust::coordinates::frames::ECEF;
use siderust::coordinates::spherical::direction;
use siderust::time::{intersect_periods, Interval, ModifiedJulianDate};

use crate::api::{Constraints, GeographicLocation, Period};
use crate::models::ModifiedJulianDate as AppMJD;

type MjdInterval = Interval<ModifiedJulianDate>;

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
    let site = Geodetic::<ECEF>::new(
        Degrees::new(input.location.longitude),
        Degrees::new(input.location.latitude),
        Meters::new(input.location.elevation_m.unwrap_or(0.0)),
    );

    // Determine effective search window: schedule_period clipped to fixed_time.
    let window = match &input.constraints.fixed_time {
        Some(fixed) => {
            let start = input.schedule_period.start.value().max(fixed.start.value());
            let stop = input.schedule_period.stop.value().min(fixed.stop.value());
            if start >= stop {
                return Vec::new();
            }
            Interval::<ModifiedJulianDate>::new(
                ModifiedJulianDate::new(start),
                ModifiedJulianDate::new(stop),
            )
        }
        None => Interval::<ModifiedJulianDate>::new(
            ModifiedJulianDate::new(input.schedule_period.start.value()),
            ModifiedJulianDate::new(input.schedule_period.stop.value()),
        ),
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
        Some(nights) => {
            let night_intervals = periods_to_intervals(nights);
            intersect_periods(&position_intervals, &night_intervals)
        }
        None => position_intervals,
    };

    constrained_intervals
        .into_iter()
        .filter(|iv| (iv.end.value() - iv.start.value()) >= min_days)
        .map(|iv| Period {
            start: AppMJD::new(iv.start.value()),
            stop: AppMJD::new(iv.end.value()),
        })
        .collect()
}

fn periods_to_intervals(periods: &[Period]) -> Vec<MjdInterval> {
    periods
        .iter()
        .map(|p| {
            MjdInterval::new(
                ModifiedJulianDate::new(p.start.value()),
                ModifiedJulianDate::new(p.stop.value()),
            )
        })
        .collect()
}

fn is_full_azimuth_range(min_az: Degrees, max_az: Degrees) -> bool {
    max_az.value() >= min_az.value() && (max_az.value() - min_az.value()) >= 360.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Constraints, GeographicLocation, ModifiedJulianDate, Period};

    fn roque_location() -> GeographicLocation {
        GeographicLocation {
            latitude: 28.7624,
            longitude: -17.8892,
            elevation_m: Some(2396.0),
        }
    }

    fn one_week_period() -> Period {
        Period {
            start: ModifiedJulianDate::new(60694.0), // 2026-01-15
            stop: ModifiedJulianDate::new(60701.0),  // 2026-01-22
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
                p.start.value() < p.stop.value(),
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
            .map(|p| p.stop.value() - p.start.value())
            .sum();
        let tight_total: f64 = compute_block_visibility(&input_tight)
            .iter()
            .map(|p| p.stop.value() - p.start.value())
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
            .map(|p| p.stop.value() - p.start.value())
            .sum();
        let constrained_total: f64 = compute_block_visibility(&constrained)
            .iter()
            .map(|p| p.stop.value() - p.start.value())
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
                stop: ModifiedJulianDate::new(60697.0), // 3 days only
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
            assert!(p.stop.value() <= 60697.0);
        }

        let full_total: f64 = full_periods
            .iter()
            .map(|p| p.stop.value() - p.start.value())
            .sum();
        let clipped_total: f64 = clipped_periods
            .iter()
            .map(|p| p.stop.value() - p.start.value())
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
                stop: ModifiedJulianDate::new(60801.0),
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
                p.stop.value() - p.start.value() >= min_days,
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
                stop: ModifiedJulianDate::new(60698.0),
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
            assert!(p.stop.value() <= 60698.0);
        }

        // All results must meet duration requirement
        let min_days = 600.0 / 86_400.0;
        for p in &periods {
            assert!(p.stop.value() - p.start.value() >= min_days);
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
                stop: ModifiedJulianDate::new(60694.8),
            }),
        };
        let nights = vec![Period {
            start: ModifiedJulianDate::new(60694.4),
            stop: ModifiedJulianDate::new(60694.6),
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
            assert!(p.stop.value() <= 60694.6);
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
