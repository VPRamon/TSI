// ! Backend visibility computation service.
//!
//! Computes per-block visibility periods from observer location, schedule period,
//! target RA/Dec, altitude/azimuth constraints, optional fixed_time window, and
//! minimum duration filter.
//!
//! This is the fallback path used when `possible_periods` is absent from the
//! incoming JSON payload.

use qtty::{Degrees, Meters, Seconds};
use siderust::calculus::altitude::{AltitudePeriodsProvider, AltitudeQuery};
use siderust::coordinates::centers::Geodetic;
use siderust::coordinates::frames::ECEF;
use siderust::coordinates::spherical::direction;
use siderust::time::{Interval, ModifiedJulianDate};

use crate::api::{Constraints, GeographicLocation, Period};
use crate::models::ModifiedJulianDate as AppMJD;

/// Input parameters for a single block's visibility computation.
pub struct VisibilityInput<'a> {
    pub location: &'a GeographicLocation,
    pub schedule_period: &'a Period,
    pub target_ra: Degrees,
    pub target_dec: Degrees,
    pub constraints: &'a Constraints,
    pub min_duration: Seconds,
}

/// Compute visibility periods for a single block.
///
/// Returns intervals within `schedule_period` (intersected with `fixed_time` if
/// present) where the target satisfies the altitude constraints and the duration
/// is at least `min_duration`.
///
/// Azimuth constraints other than the full circle (0–360°) are currently not
/// filtered at the sub-period level; only altitude is enforced by siderust.
/// When `max_az - min_az < 360`, the returned periods are a superset (altitude
/// only); callers that need strict azimuth filtering should post-process.
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
    let raw_periods = icrs.altitude_periods(&query);

    // Convert siderust Period<MJD> → api::Period, applying duration filter.
    let min_days = input.min_duration.value() / 86_400.0;
    raw_periods
        .into_iter()
        .filter(|p| (p.end.value() - p.start.value()) >= min_days)
        .map(|p| Period {
            start: AppMJD::new(p.start.value()),
            stop: AppMJD::new(p.end.value()),
        })
        .collect()
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
        };
        let input_tight = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &tight,
            min_duration: Seconds::new(0.0),
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
        };
        let clipped = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &fixed,
            min_duration: Seconds::new(0.0),
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
        };
        let input_filtered = VisibilityInput {
            location: &roque_location(),
            schedule_period: &one_week_period(),
            target_ra: Degrees::new(95.988),
            target_dec: Degrees::new(-52.696),
            constraints: &full_sky_constraints(),
            min_duration: Seconds::new(3600.0),
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
}
