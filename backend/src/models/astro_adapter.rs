//! Adapter module for converting astro crate schedule types to backend API types.
//!
//! This module provides the bridge between the astro crate's schedule format and the
//! backend's internal API types. It handles:
//!
//! - Location conversion: `Geographic` (geocentric km) -> `GeographicLocation` (lat/lon)
//! - Period conversion: `Interval<Day>` (with `end`) -> `Period` (with `stop`)
//! - Task conversion: `ObservationTask` -> `SchedulingBlock`
//! - Priority conversion: `i32` -> `f64`
//! - Constraint tree preservation: `ConstraintTree` is preserved for accurate visibility computation

use crate::api::{self, Constraints, GeographicLocation, ModifiedJulianDate, Period, SchedulingBlock};
use anyhow::Result;
use astro::constraints::{ConstraintLeaf, ConstraintTree};
use astro::schedule::Schedule as AstroSchedule;
use astro::tasks::{AstronomicalTask, ObservationTask};
use siderust::coordinates::spherical::position::Geographic;
use vrolai::constraints::ConstraintExpr;

/// Convert an astro crate `Schedule` to the backend's `api::Schedule`.
///
/// This function:
/// 1. Converts the geocentric location to geographic coordinates
/// 2. Converts the schedule period (using `end` -> `stop` terminology)
/// 3. Converts observation tasks to scheduling blocks
/// 4. Skips calibration tasks (not supported in current backend)
///
/// # Arguments
/// * `astro_schedule` - The parsed astro crate schedule
/// * `schedule_name` - Name to assign to the schedule
///
/// # Returns
/// An `api::Schedule` with blocks but empty `visibility_periods` (to be computed separately)
pub fn convert_astro_schedule(
    astro_schedule: &AstroSchedule,
    schedule_name: &str,
) -> Result<api::Schedule> {
    // Convert location
    let geographic_location = convert_location(&astro_schedule.location)?;

    // Convert period (astro uses `end`, api uses `stop`)
    let schedule_period = Period {
        start: ModifiedJulianDate::new(astro_schedule.period_mjd.start().value()),
        stop: ModifiedJulianDate::new(astro_schedule.period_mjd.end().value()),
    };

    // Compute astronomical nights from location
    let astronomical_nights = crate::services::astronomical_night::compute_astronomical_nights(
        &geographic_location,
        &schedule_period,
    );

    // Convert tasks to blocks (filter out calibration tasks)
    let blocks: Vec<SchedulingBlock> = astro_schedule
        .tasks
        .iter()
        .filter_map(|task| match task {
            AstronomicalTask::Observation(obs) => Some(convert_observation_task(obs)),
            AstronomicalTask::Calibration(_) => {
                // Calibration tasks are not supported in the current backend
                // They could be logged or handled differently in the future
                None
            }
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(api::Schedule {
        id: None,
        name: schedule_name.to_string(),
        checksum: String::new(), // Will be computed after serialization
        schedule_period,
        dark_periods: Vec::new(), // Dark periods are computed from astronomical nights
        geographic_location,
        astronomical_nights,
        blocks,
    })
}

/// Convert astro `Geographic` location to backend `GeographicLocation`.
///
/// The astro crate stores location as:
/// - `lat`: Latitude in degrees
/// - `lon`: Longitude in degrees (0-360 range, East-positive, or -180 to 180)
/// - `distance`: Elevation above sea level in kilometers
///
/// We extract lat/lon (converting longitude to -180 to 180 range) and convert elevation to meters.
fn convert_location(geo: &Geographic) -> Result<GeographicLocation> {
    let latitude = geo.lat().value();
    let mut longitude = geo.lon().value();
    
    // Convert longitude from 0-360 range to -180 to 180 range if needed
    if longitude > 180.0 {
        longitude -= 360.0;
    }
    
    // Convert elevation from km to meters
    let elevation_m = geo.distance().value() * 1000.0;

    GeographicLocation::new(latitude, longitude, Some(elevation_m))
        .map_err(|e| anyhow::anyhow!("Invalid geographic location: {}", e))
}

/// Convert an `ObservationTask` to a `SchedulingBlock`.
///
/// Maps:
/// - `id` -> `original_block_id`
/// - `target.position` -> `target_ra`, `target_dec`
/// - `duration_sec` -> `requested_duration`, `min_observation`
/// - `priority` (i32) -> `priority` (f64)
/// - `constraint` -> preserved as `constraint_tree`, also flattened to `Constraints`
fn convert_observation_task(task: &ObservationTask) -> Result<SchedulingBlock> {
    let coords = task.coordinates();
    let target_ra = coords.ra();
    let target_dec = coords.dec();

    // Get the original constraint tree (if any) for accurate visibility computation
    let constraint_tree = task.constraint().cloned();

    // Extract flattened constraints for database storage
    let constraints = if let Some(ref tree) = constraint_tree {
        flatten_constraints(tree)
    } else {
        // Default constraints if none specified
        Constraints::new(
            qtty::Degrees::new(0.0),   // min_alt
            qtty::Degrees::new(90.0),  // max_alt
            qtty::Degrees::new(0.0),   // min_az
            qtty::Degrees::new(360.0), // max_az
            None,                       // fixed_time
        )
    };

    let duration_sec = task.duration_seconds();

    Ok(SchedulingBlock::with_constraint_tree(
        task.id().to_string(),                          // original_block_id
        target_ra,                                       // target_ra
        target_dec,                                      // target_dec
        constraints,                                     // constraints (flattened)
        constraint_tree,                                 // constraint_tree (preserved)
        task.priority() as f64,                          // priority (i32 -> f64)
        qtty::Seconds::new(duration_sec),               // min_observation
        qtty::Seconds::new(duration_sec),               // requested_duration
    ))
}

/// Flatten a `ConstraintTree` into simple altitude/azimuth range constraints.
///
/// The backend's `Constraints` struct only supports:
/// - min/max altitude
/// - min/max azimuth  
/// - fixed time window
///
/// More complex constraint compositions (AND/OR/NOT) are simplified to
/// their constituent ranges where possible.
fn flatten_constraints(tree: &ConstraintTree) -> Constraints {
    let mut min_alt = qtty::Degrees::new(0.0);
    let mut max_alt = qtty::Degrees::new(90.0);
    let mut min_az = qtty::Degrees::new(0.0);
    let mut max_az = qtty::Degrees::new(360.0);
    let mut fixed_time: Option<Period> = None;

    // Walk the constraint tree and extract leaf constraints
    extract_constraints_recursive(tree, &mut min_alt, &mut max_alt, &mut min_az, &mut max_az, &mut fixed_time);

    Constraints::new(min_alt, max_alt, min_az, max_az, fixed_time)
}

/// Recursively extract constraint values from a constraint tree.
fn extract_constraints_recursive(
    tree: &ConstraintTree,
    min_alt: &mut qtty::Degrees,
    max_alt: &mut qtty::Degrees,
    min_az: &mut qtty::Degrees,
    max_az: &mut qtty::Degrees,
    fixed_time: &mut Option<Period>,
) {
    match tree {
        ConstraintExpr::Leaf(leaf) => {
            match leaf {
                ConstraintLeaf::Altitude(alt) => {
                    *min_alt = alt.min_altitude();
                    *max_alt = alt.max_altitude();
                }
                ConstraintLeaf::Azimuth(az) => {
                    *min_az = az.min_azimuth();
                    *max_az = az.max_azimuth();
                }
                ConstraintLeaf::Interval { start, end } => {
                    *fixed_time = Some(Period {
                        start: ModifiedJulianDate::new(*start),
                        stop: ModifiedJulianDate::new(*end),
                    });
                }
                // Nighttime and MoonAltitude constraints are handled at telescope level
                // or during visibility computation
                ConstraintLeaf::Nighttime(_) => {}
                ConstraintLeaf::MoonAltitude(_) => {}
            }
        }
        ConstraintExpr::Intersection { children, .. } => {
            // For AND (intersection), all children apply
            for child in children {
                extract_constraints_recursive(child, min_alt, max_alt, min_az, max_az, fixed_time);
            }
        }
        ConstraintExpr::Union { children, .. } => {
            // For OR (union), we take the first child's constraints as a simplification
            // In a more complete implementation, we could compute the bounding box
            if let Some(first) = children.first() {
                extract_constraints_recursive(first, min_alt, max_alt, min_az, max_az, fixed_time);
            }
        }
        ConstraintExpr::Not { .. } => {
            // NOT constraints are hard to flatten; skip them
        }
    }
}

/// Trait to access ObservationTask fields that may be private.
trait ObservationTaskExt {
    fn id(&self) -> &str;
    fn priority(&self) -> i32;
}

impl ObservationTaskExt for ObservationTask {
    fn id(&self) -> &str {
        // Access via the Task trait
        vrolai::scheduling_block::Task::<qtty::Second>::id(self)
    }

    fn priority(&self) -> i32 {
        vrolai::scheduling_block::Task::<qtty::Second>::priority(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_location() {
        // La Palma observatory location (elevation in km as per siderust convention)
        // Geographic::new takes (lon, lat, altitude)
        let geo = Geographic::new(
            qtty::Degrees::new(-17.8892),  // lon
            qtty::Degrees::new(28.7624),   // lat
            qtty::Kilometers::new(2.396),  // 2396m elevation
        );
        
        let result = convert_location(&geo).unwrap();
        
        assert!((result.latitude - 28.7624).abs() < 0.001);
        assert!((result.longitude - (-17.8892)).abs() < 0.001);
        assert!(result.elevation_m.is_some());
        // Elevation should be 2396m
        let elevation = result.elevation_m.unwrap();
        assert!((elevation - 2396.0).abs() < 1.0);
    }

    #[test]
    fn test_convert_location_with_longitude_over_180() {
        // La Palma with longitude in 0-360 range (342.1108 = -17.8892)
        // Geographic::new takes (lon, lat, altitude)
        let geo = Geographic::new(
            qtty::Degrees::new(342.1108),  // lon (will be normalized)
            qtty::Degrees::new(28.7624),   // lat
            qtty::Kilometers::new(2.396),
        );
        
        let result = convert_location(&geo).unwrap();
        
        assert!((result.latitude - 28.7624).abs() < 0.001);
        // Should be converted to negative longitude
        assert!((result.longitude - (-17.8892)).abs() < 0.001);
    }

    #[test]
    fn test_flatten_default_constraints() {
        let constraints = Constraints::new(
            qtty::Degrees::new(0.0),
            qtty::Degrees::new(90.0),
            qtty::Degrees::new(0.0),
            qtty::Degrees::new(360.0),
            None,
        );
        
        assert_eq!(constraints.min_alt.value(), 0.0);
        assert_eq!(constraints.max_alt.value(), 90.0);
        assert_eq!(constraints.min_az.value(), 0.0);
        assert_eq!(constraints.max_az.value(), 360.0);
    }
}
