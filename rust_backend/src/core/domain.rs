use siderust::astro::ModifiedJulianDate;
use siderust::units::{
    time::*,
    angular::Degrees,
};
use siderust::coordinates::spherical::direction::ICRS;

/// Represents a single visibility period (start, stop)
#[derive(Debug, Clone, PartialEq)]
pub struct Period {
    pub start: ModifiedJulianDate,
    pub stop: ModifiedJulianDate,
}

impl Period {
    pub fn new(start: ModifiedJulianDate, stop: ModifiedJulianDate) -> Self {
        Self { start, stop }
    }

    /// Returns the duration of this period in hours
    pub fn duration_hours(&self) -> f64 {
        // ModifiedJulianDate subtraction gives us days, convert to hours
        let duration_days = self.stop.value() - self.start.value();
        duration_days * 24.0
    }

    /// Returns the duration as a Days quantity
    pub fn duration(&self) -> Days {
        Days::new(self.stop.value() - self.start.value())
    }
}

/// Core scheduling block data structure
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

    pub fn is_scheduled(&self) -> bool {
        self.scheduled_period.is_some()
    }

    pub fn elevation_range(&self) -> Option<Degrees> {
        match (self.max_elevation_angle, self.min_elevation_angle) {
            (Some(max), Some(min)) => Some(max - min),
            _ => None,
        }
    }

    pub fn total_visibility_hours(&self) -> Hours {
        let total_days = self.visibility_periods
            .iter()
            .fold(Days::new(0.0), |acc, p| acc + p.duration());
        total_days.to::<Hour>()
    }

    pub fn num_visibility_periods(&self) -> usize {
        self.visibility_periods.len()
    }

    /// Assign priority bin based on priority value
    /// TODO: move this to front end
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
