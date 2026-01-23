//! Visibility period computation service.
//!
//! This module computes visibility periods for scheduling blocks using the astro crate's
//! constraint evaluation system. Visibility periods represent when a target is observable
//! from a given location, considering:
//!
//! - Target altitude above horizon
//! - Target azimuth range
//! - Nighttime (astronomical twilight)
//! - Moon altitude constraints (if specified)

use crate::api::{GeographicLocation, Period, Schedule, SchedulingBlock, ModifiedJulianDate};
use anyhow::Result;
use astro::constraints::{AltitudeConstraint, AzimuthConstraint, ConstraintLeaf, ConstraintTree, NighttimeConstraint};
use qtty::{Day, Quantity};
use siderust::astro::JulianDate;
use siderust::coordinates::spherical::direction::ICRS;
use siderust::coordinates::spherical::position::Geographic;
use siderust::targets::Target;
use vrolai::constraints::{Constraint as VrolaiConstraint, ConstraintExpr};
use vrolai::solution_space::Interval;

/// Compute visibility periods for all blocks in a schedule.
///
/// This function iterates over all blocks, computes their visibility periods
/// based on target coordinates, observer location, and constraints, then
/// updates each block's `visibility_periods` field.
///
/// # Arguments
/// * `schedule` - Mutable reference to the schedule to update
///
/// # Returns
/// * `Ok(())` on success with schedule blocks updated in place
/// * `Err` if visibility computation fails
pub fn compute_schedule_visibility(schedule: &mut Schedule) -> Result<()> {
    let observer = convert_to_geographic(&schedule.geographic_location)?;
    let schedule_interval = period_to_interval(&schedule.schedule_period);

    for block in &mut schedule.blocks {
        let visibility = compute_block_visibility(block, &observer, schedule_interval)?;
        block.visibility_periods = visibility;
    }

    Ok(())
}

/// Compute visibility periods for a single scheduling block.
///
/// Builds a constraint tree from the block's constraints and target coordinates,
/// then evaluates it over the schedule period to find visibility windows.
///
/// # Arguments
/// * `block` - The scheduling block to compute visibility for
/// * `observer` - Observer's geographic location
/// * `schedule_interval` - The time interval to search for visibility
///
/// # Returns
/// Vector of visibility periods in MJD
pub fn compute_block_visibility(
    block: &SchedulingBlock,
    observer: &Geographic,
    schedule_interval: Interval<Day>,
) -> Result<Vec<Period>> {
    // Create target from block coordinates
    let target = Target::new_static(
        ICRS::new(block.target_ra, block.target_dec),
        JulianDate::J2000,
    );

    // Build constraint tree from block constraints
    let constraint_tree = build_constraint_tree(block, &target, observer)?;

    // Compute visibility intervals using the constraint tree
    let intervals = VrolaiConstraint::<Day>::compute_intervals(&constraint_tree, schedule_interval);

    // Convert intervals to API periods
    let periods = intervals
        .into_iter()
        .map(|interval| Period {
            start: ModifiedJulianDate::new(interval.start().value()),
            stop: ModifiedJulianDate::new(interval.end().value()),
        })
        .collect();

    Ok(periods)
}

/// Build a constraint tree for a block.
///
/// The constraint tree combines:
/// 1. Nighttime constraint (astronomical twilight)
/// 2. Altitude constraint from block
/// 3. Azimuth constraint from block (if not full range)
/// 4. Fixed time window (if specified)
fn build_constraint_tree(
    block: &SchedulingBlock,
    target: &Target<ICRS>,
    observer: &Geographic,
) -> Result<ConstraintTree> {
    let mut leaves: Vec<ConstraintTree> = Vec::new();

    // Always add nighttime constraint
    let nighttime = NighttimeConstraint::new(observer.clone());
    leaves.push(ConstraintExpr::Leaf(ConstraintLeaf::Nighttime(nighttime)));

    // Add altitude constraint
    let altitude = AltitudeConstraint::new(
        block.constraints.min_alt,
        block.constraints.max_alt,
        target.clone(),
        observer.clone(),
    );
    leaves.push(ConstraintExpr::Leaf(ConstraintLeaf::Altitude(altitude)));

    // Add azimuth constraint if not full range
    if block.constraints.min_az.value() > 0.0 || block.constraints.max_az.value() < 360.0 {
        let azimuth = AzimuthConstraint::new(
            block.constraints.min_az,
            block.constraints.max_az,
            target.clone(),
            observer.clone(),
        );
        leaves.push(ConstraintExpr::Leaf(ConstraintLeaf::Azimuth(azimuth)));
    }

    // Add fixed time window if specified
    if let Some(ref fixed_time) = block.constraints.fixed_time {
        leaves.push(ConstraintExpr::Leaf(ConstraintLeaf::Interval {
            start: fixed_time.start.value(),
            end: fixed_time.stop.value(),
        }));
    }

    // Combine all constraints with AND (intersection)
    if leaves.len() == 1 {
        Ok(leaves.pop().unwrap())
    } else {
        Ok(ConstraintExpr::intersection(leaves))
    }
}

/// Convert API `GeographicLocation` to siderust `Geographic`.
///
/// Note: siderust Geographic uses elevation above sea level in km (not distance from Earth center).
/// Constructor takes (lon, lat, altitude).
fn convert_to_geographic(loc: &GeographicLocation) -> Result<Geographic> {
    // Convert elevation from meters to km
    let elevation_km = loc.elevation_m.unwrap_or(0.0) / 1000.0;

    Ok(Geographic::new(
        qtty::Degrees::new(loc.longitude),  // lon first
        qtty::Degrees::new(loc.latitude),   // lat second
        qtty::Kilometers::new(elevation_km),
    ))
}

/// Convert API `Period` to virolai `Interval<Day>`.
fn period_to_interval(period: &Period) -> Interval<Day> {
    Interval::new(
        Quantity::<Day>::new(period.start.value()),
        Quantity::<Day>::new(period.stop.value()),
    )
}

/// Compute the union of all visibility periods from schedule blocks.
///
/// This produces a single list of merged periods that can be stored as
/// `schedules.possible_periods_json` in the database.
pub fn compute_possible_periods_union(blocks: &[SchedulingBlock]) -> Vec<Period> {
    let mut all_periods: Vec<Period> = blocks
        .iter()
        .flat_map(|b| b.visibility_periods.clone())
        .collect();

    // Sort by start time
    all_periods.sort_by(|a, b| {
        a.start.value().partial_cmp(&b.start.value()).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Merge overlapping periods
    let mut merged: Vec<Period> = Vec::new();
    for period in all_periods {
        if let Some(last) = merged.last_mut() {
            if period.start.value() <= last.stop.value() {
                // Extend the last period if overlapping
                if period.stop.value() > last.stop.value() {
                    last.stop = period.stop;
                }
            } else {
                merged.push(period);
            }
        } else {
            merged.push(period);
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Constraints;

    fn test_location() -> GeographicLocation {
        GeographicLocation::new(28.7624, -17.8892, Some(2396.0)).unwrap()
    }

    fn test_schedule_period() -> Period {
        Period {
            start: ModifiedJulianDate::new(60676.0),
            stop: ModifiedJulianDate::new(60677.0),
        }
    }

    #[test]
    fn test_convert_to_geographic() {
        let loc = test_location();
        let geo = convert_to_geographic(&loc).unwrap();

        assert!((geo.lat().value() - 28.7624).abs() < 0.001);
        // Longitude is normalized to 0-360 by Geographic constructor
        // -17.8892 becomes 342.1108
        let expected_lon = 360.0 - 17.8892; // 342.1108
        assert!((geo.lon().value() - expected_lon).abs() < 0.001);
        // Distance should be elevation (2.396 km)
        assert!((geo.distance().value() - 2.396).abs() < 0.01);
    }

    #[test]
    fn test_period_to_interval() {
        let period = test_schedule_period();
        let interval = period_to_interval(&period);

        assert!((interval.start().value() - 60676.0).abs() < 0.001);
        assert!((interval.end().value() - 60677.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_possible_periods_union() {
        let blocks = vec![
            SchedulingBlock::new(
                "1".to_string(),
                qtty::Degrees::new(10.0),
                qtty::Degrees::new(41.0),
                Constraints::new(
                    qtty::Degrees::new(30.0),
                    qtty::Degrees::new(90.0),
                    qtty::Degrees::new(0.0),
                    qtty::Degrees::new(360.0),
                    None,
                ),
                5.0,
                qtty::Seconds::new(1200.0),
                qtty::Seconds::new(1200.0),
                None,
                Some(vec![
                    Period::from_mjd(60676.0, 60676.5),
                    Period::from_mjd(60677.0, 60677.5),
                ]),
                None,
            ),
            SchedulingBlock::new(
                "2".to_string(),
                qtty::Degrees::new(20.0),
                qtty::Degrees::new(50.0),
                Constraints::new(
                    qtty::Degrees::new(30.0),
                    qtty::Degrees::new(90.0),
                    qtty::Degrees::new(0.0),
                    qtty::Degrees::new(360.0),
                    None,
                ),
                7.0,
                qtty::Seconds::new(1800.0),
                qtty::Seconds::new(1800.0),
                None,
                Some(vec![
                    Period::from_mjd(60676.3, 60676.8), // Overlaps with first block's first period
                ]),
                None,
            ),
        ];

        let union = compute_possible_periods_union(&blocks);

        // Should merge overlapping periods
        assert_eq!(union.len(), 2);
        // First merged period: 60676.0 - 60676.8
        assert!((union[0].start.value() - 60676.0).abs() < 0.001);
        assert!((union[0].stop.value() - 60676.8).abs() < 0.001);
    }
}
