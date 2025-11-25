//! Domain models for telescope scheduling blocks and time periods.
//!
//! This module provides the core data structures that represent observation schedules,
//! including time periods, visibility windows, and scheduling constraints.

use siderust::astro::ModifiedJulianDate;
use siderust::units::{
    time::*,
    angular::Degrees,
};
use siderust::coordinates::spherical::direction::ICRS;

/// Represents a single time period with start and stop times.
///
/// A `Period` defines a contiguous time interval using Modified Julian Dates (MJD).
/// This is used to represent visibility windows, scheduled observation times,
/// and fixed time constraints.
///
/// # Examples
///
/// ```
/// use tsi_rust::core::domain::Period;
/// use siderust::astro::ModifiedJulianDate;
///
/// let start = ModifiedJulianDate::new(59000.0);
/// let stop = ModifiedJulianDate::new(59000.5);
/// let period = Period::new(start, stop);
///
/// assert_eq!(period.duration_hours(), 12.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Period {
    pub start: ModifiedJulianDate,
    pub stop: ModifiedJulianDate,
}

impl Period {
    /// Creates a new time period.
    ///
    /// # Arguments
    ///
    /// * `start` - The start time of the period in MJD
    /// * `stop` - The end time of the period in MJD
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::core::domain::Period;
    /// use siderust::astro::ModifiedJulianDate;
    ///
    /// let period = Period::new(
    ///     ModifiedJulianDate::new(59000.0),
    ///     ModifiedJulianDate::new(59001.0)
    /// );
    /// ```
    pub fn new(start: ModifiedJulianDate, stop: ModifiedJulianDate) -> Self {
        Self { start, stop }
    }

    /// Returns the duration of this period in hours.
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::core::domain::Period;
    /// use siderust::astro::ModifiedJulianDate;
    ///
    /// let period = Period::new(
    ///     ModifiedJulianDate::new(100.0),
    ///     ModifiedJulianDate::new(100.5)
    /// );
    /// assert_eq!(period.duration_hours(), 12.0);
    /// ```
    pub fn duration_hours(&self) -> f64 {
        // ModifiedJulianDate subtraction gives us days, convert to hours
        let duration_days = self.stop.value() - self.start.value();
        duration_days * 24.0
    }

    /// Returns the duration as a strongly-typed `Days` quantity.
    ///
    /// This provides a type-safe representation of the duration that can be
    /// converted to other time units using the `siderust::units` API.
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::core::domain::Period;
    /// use siderust::astro::ModifiedJulianDate;
    /// use siderust::units::time::Days;
    ///
    /// let period = Period::new(
    ///     ModifiedJulianDate::new(0.0),
    ///     ModifiedJulianDate::new(1.5)
    /// );
    /// assert_eq!(period.duration(), Days::new(1.5));
    /// ```
    pub fn duration(&self) -> Days {
        Days::new(self.stop.value() - self.start.value())
    }
}

/// Represents a telescope observation scheduling block with all constraints and metadata.
///
/// A `SchedulingBlock` encapsulates all information about a single astronomical observation,
/// including its priority, requested duration, pointing coordinates, angular constraints,
/// visibility windows, and actual scheduled time (if assigned).
///
/// # Fields
///
/// * `scheduling_block_id` - Unique identifier for this observation
/// * `priority` - Scheduling priority (higher values = higher priority)
/// * `requested_duration` - Total observation time requested
/// * `min_observation_time` - Minimum acceptable observation duration
/// * `coordinates` - Target sky coordinates in ICRS frame (optional)
/// * `fixed_time` - Fixed time window constraint (optional)
/// * `min_azimuth_angle` - Minimum azimuth constraint in degrees (optional)
/// * `max_azimuth_angle` - Maximum azimuth constraint in degrees (optional)
/// * `min_elevation_angle` - Minimum elevation constraint in degrees (optional)
/// * `max_elevation_angle` - Maximum elevation constraint in degrees (optional)
/// * `scheduled_period` - Actual scheduled time period (None if unscheduled)
/// * `visibility_periods` - List of time windows when target is observable
///
/// # Examples
///
/// ```
/// use tsi_rust::core::domain::{SchedulingBlock, Period};
/// use siderust::astro::ModifiedJulianDate;
/// use siderust::units::time::Seconds;
/// use siderust::units::angular::Degrees;
///
/// let block = SchedulingBlock {
///     scheduling_block_id: "SB001".to_string(),
///     priority: 10.0,
///     requested_duration: Seconds::new(3600.0),
///     min_observation_time: Seconds::new(1800.0),
///     coordinates: None,
///     fixed_time: None,
///     min_azimuth_angle: None,
///     max_azimuth_angle: None,
///     min_elevation_angle: Some(Degrees::new(30.0)),
///     max_elevation_angle: Some(Degrees::new(80.0)),
///     scheduled_period: None,
///     visibility_periods: vec![],
/// };
///
/// assert!(!block.is_scheduled());
/// ```
#[derive(Debug, Clone)]
pub struct SchedulingBlock {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub requested_duration: Seconds,
    pub min_observation_time: Seconds,
    pub coordinates: Option<ICRS>,

    // Constraints
    pub fixed_time: Option<Period>,
    pub min_azimuth_angle: Option<Degrees>,
    pub max_azimuth_angle: Option<Degrees>,
    pub min_elevation_angle: Option<Degrees>,
    pub max_elevation_angle: Option<Degrees>,

    pub scheduled_period: Option<Period>,

    pub visibility_periods: Vec<Period>,
}

impl SchedulingBlock {
    /// Returns `true` if this block has been assigned a scheduled time period.
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::core::domain::{SchedulingBlock, Period};
    /// use siderust::astro::ModifiedJulianDate;
    /// use siderust::units::time::Seconds;
    ///
    /// let mut block = SchedulingBlock {
    ///     scheduling_block_id: "SB001".to_string(),
    ///     priority: 10.0,
    ///     requested_duration: Seconds::new(3600.0),
    ///     min_observation_time: Seconds::new(1800.0),
    ///     coordinates: None,
    ///     fixed_time: None,
    ///     min_azimuth_angle: None,
    ///     max_azimuth_angle: None,
    ///     min_elevation_angle: None,
    ///     max_elevation_angle: None,
    ///     scheduled_period: None,
    ///     visibility_periods: vec![],
    /// };
    ///
    /// assert!(!block.is_scheduled());
    ///
    /// block.scheduled_period = Some(Period::new(
    ///     ModifiedJulianDate::new(59000.0),
    ///     ModifiedJulianDate::new(59000.5)
    /// ));
    /// assert!(block.is_scheduled());
    /// ```
    pub fn is_scheduled(&self) -> bool {
        self.scheduled_period.is_some()
    }

    /// Returns the elevation angle range if both min and max are specified.
    ///
    /// Computes the difference between maximum and minimum elevation constraints.
    /// Returns `None` if either constraint is not set.
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::core::domain::SchedulingBlock;
    /// use siderust::units::{time::Seconds, angular::Degrees};
    ///
    /// let mut block = SchedulingBlock {
    ///     scheduling_block_id: "SB001".to_string(),
    ///     priority: 10.0,
    ///     requested_duration: Seconds::new(3600.0),
    ///     min_observation_time: Seconds::new(1800.0),
    ///     coordinates: None,
    ///     fixed_time: None,
    ///     min_azimuth_angle: None,
    ///     max_azimuth_angle: None,
    ///     min_elevation_angle: Some(Degrees::new(20.0)),
    ///     max_elevation_angle: Some(Degrees::new(70.0)),
    ///     scheduled_period: None,
    ///     visibility_periods: vec![],
    /// };
    ///
    /// assert_eq!(block.elevation_range().unwrap().value(), 50.0);
    /// ```
    pub fn elevation_range(&self) -> Option<Degrees> {
        match (self.max_elevation_angle, self.min_elevation_angle) {
            (Some(max), Some(min)) => Some(max - min),
            _ => None,
        }
    }

    /// Computes the total visibility duration across all visibility periods.
    ///
    /// Sums up the duration of all periods in the `visibility_periods` vector
    /// and returns the result as a `Hours` quantity.
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::core::domain::{SchedulingBlock, Period};
    /// use siderust::astro::ModifiedJulianDate;
    /// use siderust::units::time::{Seconds, Hours};
    ///
    /// let block = SchedulingBlock {
    ///     scheduling_block_id: "SB001".to_string(),
    ///     priority: 10.0,
    ///     requested_duration: Seconds::new(3600.0),
    ///     min_observation_time: Seconds::new(1800.0),
    ///     coordinates: None,
    ///     fixed_time: None,
    ///     min_azimuth_angle: None,
    ///     max_azimuth_angle: None,
    ///     min_elevation_angle: None,
    ///     max_elevation_angle: None,
    ///     scheduled_period: None,
    ///     visibility_periods: vec![
    ///         Period::new(ModifiedJulianDate::new(0.0), ModifiedJulianDate::new(0.5)),
    ///         Period::new(ModifiedJulianDate::new(1.0), ModifiedJulianDate::new(1.25)),
    ///     ],
    /// };
    ///
    /// assert_eq!(block.total_visibility_hours(), Hours::new(18.0));
    /// ```
    pub fn total_visibility_hours(&self) -> Hours {
        let total_days = self.visibility_periods
            .iter()
            .fold(Days::new(0.0), |acc, p| acc + p.duration());
        total_days.to::<Hour>()
    }

    /// Returns the number of visibility periods for this observation.
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::core::domain::{SchedulingBlock, Period};
    /// use siderust::astro::ModifiedJulianDate;
    /// use siderust::units::time::Seconds;
    ///
    /// let block = SchedulingBlock {
    ///     scheduling_block_id: "SB001".to_string(),
    ///     priority: 10.0,
    ///     requested_duration: Seconds::new(3600.0),
    ///     min_observation_time: Seconds::new(1800.0),
    ///     coordinates: None,
    ///     fixed_time: None,
    ///     min_azimuth_angle: None,
    ///     max_azimuth_angle: None,
    ///     min_elevation_angle: None,
    ///     max_elevation_angle: None,
    ///     scheduled_period: None,
    ///     visibility_periods: vec![
    ///         Period::new(ModifiedJulianDate::new(0.0), ModifiedJulianDate::new(1.0)),
    ///         Period::new(ModifiedJulianDate::new(2.0), ModifiedJulianDate::new(3.0)),
    ///     ],
    /// };
    ///
    /// assert_eq!(block.num_visibility_periods(), 2);
    /// ```
    pub fn num_visibility_periods(&self) -> usize {
        self.visibility_periods.len()
    }

    /// Categorizes the observation priority into a human-readable bin.
    ///
    /// Maps numeric priority values to descriptive categories for display purposes.
    /// The bins are:
    /// - "Invalid (<0)" for negative priorities
    /// - "Low (0-8)" for priorities 0 to <8
    /// - "Medium (8-10)" for priorities 8 to <10
    /// - "High (10-12)" for priorities 10 to <12
    /// - "Very High (12-15)" for priorities 12 to <15
    /// - "Critical (>15)" for priorities ≥15
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::core::domain::SchedulingBlock;
    /// use siderust::units::time::Seconds;
    ///
    /// let block = SchedulingBlock {
    ///     scheduling_block_id: "SB001".to_string(),
    ///     priority: 11.5,
    ///     requested_duration: Seconds::new(3600.0),
    ///     min_observation_time: Seconds::new(1800.0),
    ///     coordinates: None,
    ///     fixed_time: None,
    ///     min_azimuth_angle: None,
    ///     max_azimuth_angle: None,
    ///     min_elevation_angle: None,
    ///     max_elevation_angle: None,
    ///     scheduled_period: None,
    ///     visibility_periods: vec![],
    /// };
    ///
    /// assert_eq!(block.priority_bin(), "High (10-12)");
    /// ```
    ///
    /// # Note
    ///
    /// This is a presentation-layer helper that may be moved to frontend code in the future.
    pub fn priority_bin(&self) -> &'static str {
        if self.priority < 0.0 {
            "Invalid (<0)"
        } else if self.priority < 8.0 {
            "Low (0-8)"
        } else if self.priority < 10.0 {
            "Medium (8-10)"
        } else if self.priority < 12.0 {
            "High (10-12)"
        } else if self.priority < 15.0 {
            "Very High (12-15)"
        } else {
            "Critical (>15)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siderust::units::angular::Degrees;
    use siderust::units::time::{Days, Hours, Seconds};

    #[test]
    fn period_duration_helpers() {
        let start = ModifiedJulianDate::new(100.0);
        let stop = ModifiedJulianDate::new(100.5);
        let period = Period::new(start, stop);

        assert_eq!(period.duration_hours(), 12.0);
        assert_eq!(period.duration(), Days::new(0.5));
    }

    #[test]
    fn scheduling_block_derived_values() {
        let visibility = vec![
            Period::new(ModifiedJulianDate::new(0.0), ModifiedJulianDate::new(0.25)),
            Period::new(ModifiedJulianDate::new(1.0), ModifiedJulianDate::new(1.5)),
        ];

        let block = SchedulingBlock {
            scheduling_block_id: "sb-1".to_string(),
            priority: 9.5,
            requested_duration: Seconds::new(600.0),
            min_observation_time: Seconds::new(300.0),
            coordinates: None,
            fixed_time: None,
            min_azimuth_angle: None,
            max_azimuth_angle: None,
            min_elevation_angle: Some(Degrees::new(30.0)),
            max_elevation_angle: Some(Degrees::new(80.0)),
            scheduled_period: Some(Period::new(
                ModifiedJulianDate::new(0.0),
                ModifiedJulianDate::new(0.5),
            )),
            visibility_periods: visibility,
        };

        assert!(block.is_scheduled());
        assert_eq!(block.num_visibility_periods(), 2);
        assert_eq!(block.elevation_range().unwrap().value(), 50.0);
        assert_eq!(block.total_visibility_hours(), Hours::new(18.0));
        assert_eq!(block.priority_bin(), "Medium (8-10)");
    }

    #[test]
    fn priority_bins_cover_boundaries() {
        let thresholds = vec![
            (-1.0, "Invalid (<0)"),
            (0.0, "Low (0-8)"),
            (8.0, "Medium (8-10)"),
            (10.0, "High (10-12)"),
            (12.0, "Very High (12-15)"),
            (15.0, "Critical (>15)"),
        ];

        for (priority, expected_bin) in thresholds {
            let block = SchedulingBlock {
                scheduling_block_id: "bin-test".to_string(),
                priority,
                requested_duration: Seconds::new(1.0),
                min_observation_time: Seconds::new(1.0),
                coordinates: None,
                fixed_time: None,
                min_azimuth_angle: None,
                max_azimuth_angle: None,
                min_elevation_angle: None,
                max_elevation_angle: None,
                scheduled_period: None,
                visibility_periods: vec![],
            };

            assert_eq!(block.priority_bin(), expected_bin);
        }
    }
}
