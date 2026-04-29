//! Environment preschedule cache computation and application.
//!
//! This module provides functions to compute and apply cached preschedule data
//! (astronomical nights and block visibility) for environments that share
//! identical structure.

use crate::api::{Period, Schedule};
use crate::services::astronomical_night::{compute_astronomical_nights, compute_dark_periods};
use crate::services::visibility::{compute_block_visibility, VisibilityInput};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cached preschedule data for an environment.
///
/// Contains astronomical nights and per-block visibility summaries that can be
/// reused across schedules with identical structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvPreschedulePayload {
    /// Astronomical night periods for the environment location and schedule period
    pub astronomical_nights: Vec<Period>,

    /// Dark periods: intersection of astronomical nights and Moon-below-horizon periods.
    /// Uses `#[serde(default)]` for backwards compatibility with cached preschedules
    /// that were serialized before this field was added.
    #[serde(default)]
    pub dark_periods: Vec<Period>,

    /// Per-block visibility summaries keyed by original_block_id
    pub block_visibility: HashMap<String, BlockVisibilitySummary>,
}

/// Visibility summary for a single block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockVisibilitySummary {
    /// Block identifier
    pub block_id: String,
    /// Visibility periods for this block
    pub visibility_periods: Vec<Period>,
    /// Total visible seconds across all periods
    pub total_visible_seconds: f64,
    /// Number of visibility periods
    pub num_periods: usize,
}

/// Computes preschedule cache data for an environment from a representative schedule.
///
/// Calculates astronomical nights and block visibility periods that can be
/// cached and reused for all schedules in the same environment.
///
/// # Arguments
/// * `schedule` - Representative schedule with structure matching the environment
///
/// # Returns
/// Preschedule payload containing astronomical nights and block visibility summaries
pub fn compute_env_preschedule(schedule: &Schedule) -> EnvPreschedulePayload {
    // Compute astronomical nights for the location and period
    let astronomical_nights =
        compute_astronomical_nights(&schedule.geographic_location, &schedule.schedule_period);

    // Compute visibility for each block
    let mut block_visibility = HashMap::new();
    for block in &schedule.blocks {
        let visibility_input = VisibilityInput {
            location: &schedule.geographic_location,
            schedule_period: &schedule.schedule_period,
            target_ra: block.target_ra,
            target_dec: block.target_dec,
            constraints: &block.constraints,
            min_duration: block.min_observation,
            astronomical_nights: Some(&astronomical_nights),
        };

        let visibility_periods: Vec<Period> = compute_block_visibility(&visibility_input);

        // Convert to visibility summary
        // Duration is in days (MJD), convert to seconds
        let total_visible_seconds = visibility_periods
            .iter()
            .map(|p| (p.end.value() - p.start.value()) * 86400.0)
            .sum();

        let summary = BlockVisibilitySummary {
            block_id: block.original_block_id.clone(),
            visibility_periods: visibility_periods.clone(),
            total_visible_seconds,
            num_periods: visibility_periods.len(),
        };

        block_visibility.insert(block.original_block_id.clone(), summary);
    }

    let dark_periods =
        compute_dark_periods(&schedule.geographic_location, &schedule.schedule_period, &astronomical_nights);

    EnvPreschedulePayload {
        astronomical_nights,
        dark_periods,
        block_visibility,
    }
}

/// Applies cached preschedule data to a schedule.
///
/// Updates the schedule's blocks with visibility periods from the environment cache.
/// Assumes the schedule structure matches the environment (caller must validate).
///
/// # Arguments
/// * `schedule` - Mutable schedule to update with cached visibility
/// * `preschedule` - Cached preschedule data from environment
pub fn apply_to_schedule(schedule: &mut Schedule, preschedule: &EnvPreschedulePayload) {
    for block in &mut schedule.blocks {
        if let Some(visibility) = preschedule.block_visibility.get(&block.original_block_id) {
            block.visibility_periods = visibility.visibility_periods.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Constraints, ModifiedJulianDate, Schedule, SchedulingBlock};
    use qtty::{Degrees, Meters, Seconds};
    use siderust::coordinates::centers::Geodetic;
    use siderust::coordinates::frames::ECEF;

    fn make_test_schedule() -> Schedule {
        Schedule {
            id: None,
            name: "Test Schedule".to_string(),
            checksum: String::new(),
            schedule_period: Period {
                start: ModifiedJulianDate::new(60000.0),
                end: ModifiedJulianDate::new(60001.0),
            },
            dark_periods: vec![],
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.89),
                Degrees::new(28.76),
                Meters::new(2200.0),
            ),
            astronomical_nights: vec![],
            blocks: vec![
                SchedulingBlock {
                    id: None,
                    original_block_id: "block1".to_string(),
                    block_name: "Test Block 1".to_string(),
                    target_ra: Degrees::new(45.0),
                    target_dec: Degrees::new(-30.0),
                    constraints: Constraints::new(
                        Degrees::new(20.0),
                        Degrees::new(80.0),
                        Degrees::new(0.0),
                        Degrees::new(360.0),
                        None,
                    ),
                    priority: 10.0,
                    min_observation: Seconds::new(300.0),
                    requested_duration: Seconds::new(600.0),
                    visibility_periods: vec![],
                    scheduled_period: None,
                },
                SchedulingBlock {
                    id: None,
                    original_block_id: "block2".to_string(),
                    block_name: "Test Block 2".to_string(),
                    target_ra: Degrees::new(120.0),
                    target_dec: Degrees::new(15.0),
                    constraints: Constraints::new(
                        Degrees::new(30.0),
                        Degrees::new(70.0),
                        Degrees::new(0.0),
                        Degrees::new(360.0),
                        None,
                    ),
                    priority: 5.0,
                    min_observation: Seconds::new(450.0),
                    requested_duration: Seconds::new(900.0),
                    visibility_periods: vec![],
                    scheduled_period: None,
                },
            ],
        }
    }

    #[test]
    fn test_compute_env_preschedule() {
        let schedule = make_test_schedule();
        let preschedule = compute_env_preschedule(&schedule);

        // Should have computed astronomical nights
        assert!(!preschedule.astronomical_nights.is_empty());

        // Should have visibility for all blocks
        assert_eq!(preschedule.block_visibility.len(), 2);
        assert!(preschedule.block_visibility.contains_key("block1"));
        assert!(preschedule.block_visibility.contains_key("block2"));

        // Each block should have visibility data
        for (block_id, visibility) in &preschedule.block_visibility {
            assert_eq!(visibility.block_id, *block_id);
            assert!(visibility.total_visible_seconds >= 0.0);
            assert_eq!(visibility.num_periods, visibility.visibility_periods.len());
        }
    }

    #[test]
    fn test_apply_to_schedule() {
        let schedule = make_test_schedule();
        let preschedule = compute_env_preschedule(&schedule);

        // Create a new schedule with same structure but no visibility
        let mut new_schedule = schedule.clone();
        for block in &mut new_schedule.blocks {
            block.visibility_periods.clear();
        }

        // Apply cached preschedule
        apply_to_schedule(&mut new_schedule, &preschedule);

        // Verify visibility was applied
        for block in &new_schedule.blocks {
            let expected_visibility = preschedule
                .block_visibility
                .get(&block.original_block_id)
                .unwrap();
            assert_eq!(
                block.visibility_periods.len(),
                expected_visibility.num_periods
            );
        }
    }

    #[test]
    fn test_apply_handles_missing_blocks() {
        let schedule = make_test_schedule();
        let preschedule = compute_env_preschedule(&schedule);

        // Create schedule with different block IDs
        let mut new_schedule = schedule.clone();
        new_schedule.blocks[0].original_block_id = "nonexistent".to_string();

        // Apply should not crash on missing block ID
        apply_to_schedule(&mut new_schedule, &preschedule);

        // First block should have empty visibility (no match)
        assert!(new_schedule.blocks[0].visibility_periods.is_empty());

        // Second block should have visibility (matched)
        assert!(!new_schedule.blocks[1].visibility_periods.is_empty());
    }
}
