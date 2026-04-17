//! Fragmentation analysis DTOs.
//!
//! Provides data types for the `/v1/schedules/{id}/fragmentation` endpoint that
//! measures schedule fragmentation against the telescope-operable baseline
//! (`dark_periods`, falling back to `astronomical_nights`).

use serde::{Deserialize, Serialize};

use crate::api::{ModifiedJulianDate, Period};
use qtty::time::Hours;

/// Classification kind for a segment of the schedule window.
///
/// Precedence (highest first, used when partitioning):
/// 1. `non_operable` — outside dark/astronomical-night baseline
/// 2. `scheduled` — covered by a scheduled observation
/// 3. `no_target_visible` — no block has raw visibility in this segment
/// 4. `visible_but_no_task_fits` — raw-visible but no block's min-observation fits
/// 5. `feasible_but_unused` — at least one block could fit here, yet none scheduled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentationSegmentKind {
    NonOperable,
    Scheduled,
    NoTargetVisible,
    VisibleButNoTaskFits,
    FeasibleButUnused,
}

/// A single classified slice of the schedule window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationSegment {
    pub start_mjd: ModifiedJulianDate,
    pub stop_mjd: ModifiedJulianDate,
    pub duration_hours: Hours,
    pub kind: FragmentationSegmentKind,
}

/// An idle operable gap (used for gap statistics and the largest-gaps table).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationGap {
    pub start_mjd: ModifiedJulianDate,
    pub stop_mjd: ModifiedJulianDate,
    pub duration_hours: Hours,
    pub cause: FragmentationSegmentKind,
}

/// Reason breakdown entry — total idle/unused hours attributed to a cause.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonBreakdownEntry {
    pub kind: FragmentationSegmentKind,
    pub total_hours: Hours,
    pub fraction_of_operable: f64,
}

/// Coarse reason a task ended up unscheduled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnscheduledReason {
    NoVisibility,
    NoContiguousFit,
    RequestedExceedsTotalVisibility,
    OtherValidationIssue,
    FeasibleButUnscheduled,
}

/// Summary for a single unscheduled-reason bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnscheduledReasonSummary {
    pub reason: UnscheduledReason,
    pub block_count: usize,
    /// Up to 10 example identifiers for UI display. Uses `original_block_id`.
    pub example_block_ids: Vec<String>,
    /// Optional human-friendly names (parallel to `example_block_ids`).
    pub example_block_names: Vec<String>,
}

/// Interpretable metrics — no composite "fragmentation score" is exposed in v1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationMetrics {
    pub schedule_hours: Hours,
    pub requested_hours: Hours,
    pub operable_hours: Hours,
    pub scheduled_hours: Hours,
    pub idle_operable_hours: Hours,
    pub raw_visibility_coverage_hours: Hours,
    pub fit_visibility_coverage_hours: Hours,
    pub gap_count: usize,
    pub gap_mean_hours: Hours,
    pub gap_median_hours: Hours,
    pub largest_gap_hours: Hours,
    /// Fraction of operable time that is scheduled (`scheduled_hours / operable_hours`).
    pub scheduled_fraction_of_operable: f64,
    /// Fraction of operable time that is idle (`idle_operable_hours / operable_hours`).
    pub idle_fraction_of_operable: f64,
    /// Fraction of operable time with at least raw target visibility.
    pub raw_visibility_fraction_of_operable: f64,
    /// Fraction of operable time where a full min-observation would fit.
    pub fit_visibility_fraction_of_operable: f64,
}

/// Top-level response for the fragmentation endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationData {
    pub schedule_id: crate::api::ScheduleId,
    pub schedule_window: Period,
    /// Operable baseline used (`dark_periods` preferred, else `astronomical_nights`).
    pub operable_periods: Vec<Period>,
    /// Which field the baseline came from.
    pub operable_source: String,
    pub segments: Vec<FragmentationSegment>,
    pub largest_gaps: Vec<FragmentationGap>,
    pub reason_breakdown: Vec<ReasonBreakdownEntry>,
    pub unscheduled_reasons: Vec<UnscheduledReasonSummary>,
    pub metrics: FragmentationMetrics,
}

/// Route function name constant.
pub const GET_FRAGMENTATION_DATA: &str = "get_fragmentation_data";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_kind_serialization() {
        let v = serde_json::to_string(&FragmentationSegmentKind::NoTargetVisible).unwrap();
        assert_eq!(v, "\"no_target_visible\"");
    }

    #[test]
    fn test_unscheduled_reason_serialization() {
        let v = serde_json::to_string(&UnscheduledReason::NoContiguousFit).unwrap();
        assert_eq!(v, "\"no_contiguous_fit\"");
    }

    #[test]
    fn test_fragmentation_data_debug() {
        let data = FragmentationData {
            schedule_id: crate::api::ScheduleId::new(1),
            schedule_window: Period::new(
                ModifiedJulianDate::new(59000.0),
                ModifiedJulianDate::new(59001.0),
            ),
            operable_periods: vec![],
            operable_source: "dark_periods".to_string(),
            segments: vec![],
            largest_gaps: vec![],
            reason_breakdown: vec![],
            unscheduled_reasons: vec![],
            metrics: FragmentationMetrics {
                schedule_hours: Hours::new(24.0),
                requested_hours: Hours::new(3.0),
                operable_hours: Hours::new(0.0),
                scheduled_hours: Hours::new(0.0),
                idle_operable_hours: Hours::new(0.0),
                raw_visibility_coverage_hours: Hours::new(0.0),
                fit_visibility_coverage_hours: Hours::new(0.0),
                gap_count: 0,
                gap_mean_hours: Hours::new(0.0),
                gap_median_hours: Hours::new(0.0),
                largest_gap_hours: Hours::new(0.0),
                scheduled_fraction_of_operable: 0.0,
                idle_fraction_of_operable: 0.0,
                raw_visibility_fraction_of_operable: 0.0,
                fit_visibility_fraction_of_operable: 0.0,
            },
        };
        let s = format!("{:?}", data);
        assert!(s.contains("FragmentationData"));
    }

    #[test]
    fn test_const_value() {
        assert_eq!(GET_FRAGMENTATION_DATA, "get_fragmentation_data");
    }
}
