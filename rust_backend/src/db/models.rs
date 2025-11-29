/// Core domain model for the scheduling system.
/// This is an in-memory representation of the "schedule concept"
/// from the database schema: schedules, dark periods, scheduling blocks,
/// and their assignments (schedule_scheduling_blocks).

use chrono::{DateTime, Utc};
use siderust::{
    astro::ModifiedJulianDate,
    units::time::{Seconds, Days},
    units::angular::Degrees,
    coordinates::spherical::direction::ICRS,
};

/// Strongly-typed identifiers (you can map these to BIGINT in the DB).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ScheduleId(pub i64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TargetId(pub i64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ConstraintsId(pub i64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SchedulingBlockId(pub i64);

/// Simple representation of a time window in Modified Julian Date.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Period {
    pub start: ModifiedJulianDate,
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

    /// Length of the interval in seconds (like your computed column).
    pub fn duration(&self) -> Days {
        Days::new(self.stop.value() - self.start.value())
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

#[derive(Debug, Clone)]
pub struct Constraints {
    pub min_alt: Degrees,
    pub max_alt: Degrees,
    pub min_az: Degrees,
    pub max_az: Degrees,
    pub fixed_time: Option<Period>,
}

/// Atomic observing request (mirrors scheduling_blocks).
#[derive(Debug, Clone)]
pub struct SchedulingBlock {
    pub id: SchedulingBlockId,
    pub target: ICRS,
    pub constraints: Constraints,
    pub priority: f32,             // NUMERIC(4,1) as f32
    pub min_observation: Seconds,
    pub requested_duration: Seconds,
    pub visibility_periods: Vec<Period>,
    pub scheduled_period: Option<Period>,
}


/// Core "Schedule" concept:
/// - Metadata (name, checksum, etc.)
/// - Dark periods
/// - Assigned scheduling blocks with optional execution windows
#[derive(Debug, Clone)]
pub struct Schedule {
    pub id: Option<ScheduleId>,
    pub name: String,
    pub checksum: String,
    pub dark_periods: Vec<Period>,
    pub blocks: Vec<SchedulingBlock>,
}

/// Lightweight metadata about a schedule (for listings).
#[derive(Debug, Clone)]
pub struct ScheduleMetadata {
    pub schedule_id: Option<i64>,
    pub schedule_name: String,
    pub upload_timestamp: DateTime<Utc>,
    pub checksum: String,
}

/// Extended schedule information including stats.
#[derive(Debug, Clone)]
pub struct ScheduleInfo {
    pub metadata: ScheduleMetadata,
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
    pub unscheduled_blocks: usize,
}
