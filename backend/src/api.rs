//! Public API surface for the Rust backend.
//!
//! This file consolidates the DTO types for the HTTP API.
//! All types derive Serialize/Deserialize for JSON serialization.

pub use crate::routes::compare::CompareBlock;
pub use crate::routes::compare::CompareData;
pub use crate::routes::compare::CompareStats;
pub use crate::routes::compare::SchedulingChange;
pub use crate::routes::distribution::DistributionBlock;
pub use crate::routes::distribution::DistributionData;
pub use crate::routes::distribution::DistributionStats;
pub use crate::routes::insights::AnalyticsMetrics;
pub use crate::routes::insights::ConflictRecord;
pub use crate::routes::insights::CorrelationEntry;
pub use crate::routes::insights::InsightsBlock;
pub use crate::routes::insights::InsightsData;
pub use crate::routes::insights::TopObservation;
pub use crate::routes::landing::ScheduleInfo;
pub use crate::routes::skymap::LightweightBlock;
pub use crate::routes::skymap::PriorityBinInfo;
pub use crate::routes::skymap::SkyMapData;
pub use crate::routes::timeline::ScheduleTimelineBlock;
pub use crate::routes::timeline::ScheduleTimelineData;
pub use crate::routes::trends::EmpiricalRatePoint;
pub use crate::routes::trends::HeatmapBin;
pub use crate::routes::trends::SmoothedPoint;
pub use crate::routes::trends::TrendsBlock;
pub use crate::routes::trends::TrendsData;
pub use crate::routes::trends::TrendsMetrics;
pub use crate::routes::validation::ValidationIssue;
pub use crate::routes::validation::ValidationReport;
pub use crate::routes::visibility::VisibilityBlockSummary;
pub use crate::routes::visibility::VisibilityMapData;

use serde::{Deserialize, Serialize};

/// Schedule identifier (database primary key).
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ScheduleId(pub i64);

/// Target identifier.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetId(pub i64);

/// Constraints identifier.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstraintsId(pub i64);

/// Scheduling block identifier.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SchedulingBlockId(pub i64);

impl ScheduleId {
    pub fn new(value: i64) -> Self {
        ScheduleId(value)
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}

impl TargetId {
    pub fn new(value: i64) -> Self {
        TargetId(value)
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}

impl ConstraintsId {
    pub fn new(value: i64) -> Self {
        ConstraintsId(value)
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}

impl SchedulingBlockId {
    pub fn new(value: i64) -> Self {
        SchedulingBlockId(value)
    }

    pub fn value(&self) -> i64 {
        self.0
    }
}

impl std::fmt::Display for ScheduleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for TargetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for ConstraintsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for SchedulingBlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ScheduleId> for i64 {
    fn from(id: ScheduleId) -> Self {
        id.0
    }
}

pub use crate::models::ModifiedJulianDate;

/// Geographic location (latitude, longitude, elevation).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeographicLocation {
    /// Latitude in decimal degrees (-90 to 90)
    pub latitude: f64,
    /// Longitude in decimal degrees (-180 to 180)
    pub longitude: f64,
    /// Elevation in meters above sea level (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation_m: Option<f64>,
}

impl GeographicLocation {
    pub fn new(latitude: f64, longitude: f64, elevation_m: Option<f64>) -> Result<Self, String> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err("Latitude must be between -90 and 90 degrees".to_string());
        }
        if !(-180.0..=180.0).contains(&longitude) {
            return Err("Longitude must be between -180 and 180 degrees".to_string());
        }
        Ok(Self {
            latitude,
            longitude,
            elevation_m,
        })
    }
}

/// Time period in Modified Julian Date (MJD) format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Period {
    /// Start time in MJD
    pub start: ModifiedJulianDate,
    /// End time in MJD
    pub stop: ModifiedJulianDate,
}

impl Period {
    pub fn new(start: ModifiedJulianDate, stop: ModifiedJulianDate) -> Option<Self> {
        if start.value() < stop.value() {
            Some(Self { start, stop })
        } else {
            None
        }
    }

    pub fn from_mjd(start: f64, stop: f64) -> Self {
        Self {
            start: ModifiedJulianDate::new(start),
            stop: ModifiedJulianDate::new(stop),
        }
    }

    pub fn start_mjd(&self) -> f64 {
        self.start.value()
    }

    pub fn stop_mjd(&self) -> f64 {
        self.stop.value()
    }

    pub fn contains_mjd(&self, mjd: f64) -> bool {
        let min_mjd = self.start.value().min(self.stop.value());
        let max_mjd = self.start.value().max(self.stop.value());
        mjd >= min_mjd && mjd <= max_mjd
    }

    /// Length of the interval in days.
    pub fn duration(&self) -> qtty::Days {
        qtty::Days::new(self.stop.value() - self.start.value())
    }

    /// Check if a given MJD instant lies inside this interval (inclusive start, exclusive end).
    pub fn contains(&self, t_mjd: ModifiedJulianDate) -> bool {
        self.start.value() <= t_mjd.value() && t_mjd.value() < self.stop.value()
    }

    /// Check if this interval overlaps with another.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start.value() < other.stop.value() && other.start.value() < self.stop.value()
    }
}

/// Observing constraints for a scheduling block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    /// Minimum altitude in degrees
    pub min_alt: qtty::Degrees,
    /// Maximum altitude in degrees
    pub max_alt: qtty::Degrees,
    /// Minimum azimuth in degrees
    pub min_az: qtty::Degrees,
    /// Maximum azimuth in degrees
    pub max_az: qtty::Degrees,
    /// Fixed observation time window in MJD
    pub fixed_time: Option<Period>,
}

impl Constraints {
    pub fn new(
        min_alt: qtty::Degrees,
        max_alt: qtty::Degrees,
        min_az: qtty::Degrees,
        max_az: qtty::Degrees,
        fixed_time: Option<Period>,
    ) -> Self {
        Self {
            min_alt,
            max_alt,
            min_az,
            max_az,
            fixed_time,
        }
    }
}

/// Individual scheduling block (observation request).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingBlock {
    /// Database ID for the block (optional on input, server-assigned)
    #[serde(default)]
    pub id: Option<SchedulingBlockId>,
    /// Original ID from JSON (shown to user, required on input for new data)
    #[serde(default)]
    pub original_block_id: String,
    /// Right Ascension in degrees (ICRS)
    pub target_ra: qtty::Degrees,
    /// Declination in degrees (ICRS)
    pub target_dec: qtty::Degrees,
    /// Observing constraints (flattened for database storage)
    pub constraints: Constraints,
    /// Original astro constraint tree (for accurate visibility computation)
    /// This is populated when parsing astro format and used for constraint evaluation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint_tree: Option<astro::constraints::ConstraintTree>,
    /// Observation priority
    pub priority: f64,
    /// Minimum observation duration in seconds
    pub min_observation: qtty::Seconds,
    /// Requested observation duration in seconds
    pub requested_duration: qtty::Seconds,
    /// Visibility periods in MJD
    #[serde(default)]
    pub visibility_periods: Vec<Period>,
    /// Scheduled time window in MJD (if scheduled)
    pub scheduled_period: Option<Period>,
}

impl SchedulingBlock {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        original_block_id: String,
        target_ra: qtty::Degrees,
        target_dec: qtty::Degrees,
        constraints: Constraints,
        priority: f64,
        min_observation: qtty::Seconds,
        requested_duration: qtty::Seconds,
        id: Option<SchedulingBlockId>,
        visibility_periods: Option<Vec<Period>>,
        scheduled_period: Option<Period>,
    ) -> Self {
        Self {
            id,
            original_block_id,
            target_ra,
            target_dec,
            constraints,
            constraint_tree: None,
            priority,
            min_observation,
            requested_duration,
            visibility_periods: visibility_periods.unwrap_or_default(),
            scheduled_period,
        }
    }

    /// Create a scheduling block with a constraint tree from astro format
    #[allow(clippy::too_many_arguments)]
    pub fn with_constraint_tree(
        original_block_id: String,
        target_ra: qtty::Degrees,
        target_dec: qtty::Degrees,
        constraints: Constraints,
        constraint_tree: Option<astro::constraints::ConstraintTree>,
        priority: f64,
        min_observation: qtty::Seconds,
        requested_duration: qtty::Seconds,
    ) -> Self {
        Self {
            id: None,
            original_block_id,
            target_ra,
            target_dec,
            constraints,
            constraint_tree,
            priority,
            min_observation,
            requested_duration,
            visibility_periods: Vec::new(),
            scheduled_period: None,
        }
    }
}

/// Top-level schedule with metadata and blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    /// Database ID
    pub id: Option<i64>,
    /// Schedule name
    #[serde(default)]
    pub name: String,
    /// SHA256 checksum of schedule data
    #[serde(default)]
    pub checksum: String,
    /// Overall time window for the schedule in MJD
    pub schedule_period: Period,
    /// Dark periods (observing windows)
    #[serde(default)]
    pub dark_periods: Vec<Period>,
    /// Geographic location of the observatory (required)
    pub geographic_location: GeographicLocation,
    /// Computed astronomical night periods (Sun altitude < -18°)
    #[serde(default)]
    pub astronomical_nights: Vec<Period>,
    /// List of scheduling blocks
    pub blocks: Vec<SchedulingBlock>,
}

impl Schedule {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Option<i64>,
        name: String,
        checksum: String,
        schedule_period: Period,
        dark_periods: Vec<Period>,
        geographic_location: GeographicLocation,
        astronomical_nights: Vec<Period>,
        blocks: Vec<SchedulingBlock>,
    ) -> Self {
        Self {
            id,
            name,
            checksum,
            schedule_period,
            dark_periods,
            geographic_location,
            astronomical_nights,
            blocks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConstraintsId, ScheduleId, SchedulingBlockId, TargetId};

    #[test]
    fn test_schedule_id_new() {
        let id = ScheduleId::new(42);
        assert_eq!(id.value(), 42);
    }

    #[test]
    fn test_schedule_id_equality() {
        let id1 = ScheduleId::new(100);
        let id2 = ScheduleId::new(100);
        let id3 = ScheduleId::new(101);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_schedule_id_ordering() {
        let id1 = ScheduleId::new(1);
        let id2 = ScheduleId::new(2);

        assert!(id1 < id2);
        assert!(id2 > id1);
    }

    #[test]
    fn test_schedule_id_clone() {
        let id1 = ScheduleId::new(123);
        let id2 = id1;
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_schedule_id_from_i64() {
        let id = ScheduleId(999);
        assert_eq!(id.0, 999);
    }

    #[test]
    fn test_target_id_new() {
        let id = TargetId::new(55);
        assert_eq!(id.value(), 55);
    }

    #[test]
    fn test_target_id_equality() {
        let id1 = TargetId::new(200);
        let id2 = TargetId::new(200);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_constraints_id_new() {
        let id = ConstraintsId::new(77);
        assert_eq!(id.value(), 77);
    }

    #[test]
    fn test_constraints_id_equality() {
        let id1 = ConstraintsId::new(300);
        let id2 = ConstraintsId::new(300);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_scheduling_block_id_new() {
        let id = SchedulingBlockId::new(88);
        assert_eq!(id.value(), 88);
    }

    #[test]
    fn test_scheduling_block_id_equality() {
        let id1 = SchedulingBlockId::new(400);
        let id2 = SchedulingBlockId::new(400);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_all_ids_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ScheduleId::new(1));
        set.insert(ScheduleId::new(2));
        set.insert(ScheduleId::new(1)); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_schedule_id_negative() {
        let id = ScheduleId::new(-1);
        assert_eq!(id.value(), -1);
    }

    #[test]
    fn test_schedule_id_zero() {
        let id = ScheduleId::new(0);
        assert_eq!(id.value(), 0);
    }
}
