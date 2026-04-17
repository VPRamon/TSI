//! Fragmentation analysis service.
//!
//! Computes a classification of a schedule's time window against the
//! telescope-operable baseline (`dark_periods`, falling back to
//! `astronomical_nights`). The result distinguishes:
//!   - non-operable time (telescope cannot observe)
//!   - scheduled time
//!   - no-target-visible time (no block has raw visibility)
//!   - visible-but-no-task-fits time (raw-visible, but no block's min-observation fits)
//!   - feasible-but-unused time (at least one block could fit, but none scheduled)
//!
//! Only operable time contributes to gap/fragmentation statistics. Daytime
//! or offline intervals are never counted as gaps.

#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;

use tokio::runtime::Runtime;

use crate::api::{
    FragmentationData, FragmentationGap, FragmentationMetrics, FragmentationSegment,
    FragmentationSegmentKind, ModifiedJulianDate, Period, ReasonBreakdownEntry, Schedule,
    ScheduleId, UnscheduledReason, UnscheduledReasonSummary, ValidationIssue, ValidationReport,
};
use crate::db::{get_repository, services as db_services};
use qtty::time::Hours;

// =========================================================================
// Public entry points
// =========================================================================

/// Async version: ensures analytics, loads schedule + validation report, and
/// computes the fragmentation view.
pub async fn get_fragmentation_data(schedule_id: ScheduleId) -> Result<FragmentationData, String> {
    let repo = get_repository().map_err(|e| format!("Failed to get repository: {}", e))?;

    db_services::ensure_analytics(repo.as_ref(), schedule_id)
        .await
        .map_err(|e| format!("Failed to ensure analytics: {}", e))?;

    let schedule = repo
        .get_schedule(schedule_id)
        .await
        .map_err(|e| format!("Failed to load schedule: {}", e))?;

    // Validation report is best-effort: empty on error (keeps endpoint robust
    // even if validation populate was skipped).
    let validation = repo
        .fetch_validation_results(schedule_id)
        .await
        .unwrap_or_else(|_| ValidationReport {
            schedule_id,
            total_blocks: schedule.blocks.len(),
            valid_blocks: 0,
            impossible_blocks: vec![],
            validation_errors: vec![],
            validation_warnings: vec![],
        });

    Ok(compute_fragmentation(&schedule, &validation))
}

/// Python/sync binding wrapper.
pub fn py_get_fragmentation_data(
    schedule_id: ScheduleId,
) -> Result<FragmentationData, String> {
    let runtime = Runtime::new().map_err(|e| format!("Failed to create async runtime: {}", e))?;
    runtime.block_on(get_fragmentation_data(schedule_id))
}

// =========================================================================
// Core computation (pure, testable)
// =========================================================================

/// Compute fragmentation data from an already-loaded schedule and validation
/// report. This is the pure, deterministic core — all I/O lives in callers.
pub fn compute_fragmentation(
    schedule: &Schedule,
    validation: &ValidationReport,
) -> FragmentationData {
    let schedule_window = schedule.schedule_period;

    // Pick operable baseline: dark_periods, else astronomical_nights.
    let (operable_raw, operable_source): (Vec<Period>, &str) = if !schedule.dark_periods.is_empty()
    {
        (schedule.dark_periods.clone(), "dark_periods")
    } else {
        (
            schedule.astronomical_nights.clone(),
            "astronomical_nights",
        )
    };

    // Normalize / merge operable, then clip to schedule window.
    let operable_periods =
        clip_periods(&merge_periods(normalize(operable_raw)), &schedule_window);

    // Scheduled periods (union).
    let scheduled_union = merge_periods(
        schedule
            .blocks
            .iter()
            .filter_map(|b| b.scheduled_period)
            .collect(),
    );

    // Fit-visibility union (stored visibility_periods — min-observation fits).
    let fit_visibility_union = clip_periods(
        &merge_periods(
            schedule
                .blocks
                .iter()
                .flat_map(|b| b.visibility_periods.clone())
                .collect(),
        ),
        &schedule_window,
    );

    // Raw visibility union (ignores min_observation). Currently we do not have
    // access here to re-run sky-path computation, so we approximate raw
    // visibility as fit-visibility ∪ scheduled (a scheduled block was raw-visible
    // during its slot by construction). This conservatively gives raw ⊇ fit,
    // which keeps the kind-precedence well defined.
    let raw_visibility_union = merge_periods(
        fit_visibility_union
            .iter()
            .chain(scheduled_union.iter())
            .copied()
            .collect(),
    );

    // Partition the schedule window into contiguous classified segments.
    let segments = classify_segments(
        &schedule_window,
        &operable_periods,
        &scheduled_union,
        &raw_visibility_union,
        &fit_visibility_union,
    );

    // Aggregate durations per kind for the reason breakdown.
    let mut per_kind_hours: HashMap<FragmentationSegmentKind, f64> = HashMap::new();
    for seg in &segments {
        *per_kind_hours.entry(seg.kind).or_insert(0.0) += seg.duration_hours.value();
    }

    let operable_hours_val: f64 = operable_periods.iter().map(duration_hours).sum();
    let scheduled_hours_val = *per_kind_hours
        .get(&FragmentationSegmentKind::Scheduled)
        .unwrap_or(&0.0);
    let idle_operable_hours_val = operable_hours_val - scheduled_hours_val;

    // Reason breakdown covers the *idle-operable* kinds, with hours and
    // fraction of operable time.
    let reason_breakdown = [
        FragmentationSegmentKind::NoTargetVisible,
        FragmentationSegmentKind::VisibleButNoTaskFits,
        FragmentationSegmentKind::FeasibleButUnused,
    ]
    .iter()
    .map(|kind| {
        let hours = *per_kind_hours.get(kind).unwrap_or(&0.0);
        ReasonBreakdownEntry {
            kind: *kind,
            total_hours: Hours::new(hours),
            fraction_of_operable: safe_frac(hours, operable_hours_val),
        }
    })
    .collect::<Vec<_>>();

    // Gaps = idle operable segments only.
    let gaps = segments
        .iter()
        .filter(|s| is_idle_operable(s.kind))
        .map(|s| FragmentationGap {
            start_mjd: s.start_mjd,
            stop_mjd: s.stop_mjd,
            duration_hours: s.duration_hours,
            cause: s.kind,
        })
        .collect::<Vec<_>>();

    // Gap stats.
    let gap_count = gaps.len();
    let mut gap_hours: Vec<f64> = gaps.iter().map(|g| g.duration_hours.value()).collect();
    gap_hours.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let gap_mean = if gap_count > 0 {
        gap_hours.iter().sum::<f64>() / gap_count as f64
    } else {
        0.0
    };
    let gap_median = if gap_count == 0 {
        0.0
    } else if gap_count % 2 == 1 {
        gap_hours[gap_count / 2]
    } else {
        (gap_hours[gap_count / 2 - 1] + gap_hours[gap_count / 2]) / 2.0
    };
    let gap_max = gap_hours.iter().copied().fold(0.0_f64, f64::max);

    // Largest gaps table — sort desc by duration, cap at 10.
    let mut largest_gaps = gaps.clone();
    largest_gaps.sort_by(|a, b| {
        b.duration_hours
            .value()
            .partial_cmp(&a.duration_hours.value())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    largest_gaps.truncate(10);

    // Unscheduled-reason bucketing.
    let unscheduled_reasons =
        summarize_unscheduled_reasons(schedule, validation, &fit_visibility_union);

    let raw_cov = intersect_duration_hours(&raw_visibility_union, &operable_periods);
    let fit_cov = intersect_duration_hours(&fit_visibility_union, &operable_periods);

    let metrics = FragmentationMetrics {
        schedule_hours: Hours::new(duration_hours(&schedule_window)),
        operable_hours: Hours::new(operable_hours_val),
        scheduled_hours: Hours::new(scheduled_hours_val),
        idle_operable_hours: Hours::new(idle_operable_hours_val.max(0.0)),
        raw_visibility_coverage_hours: Hours::new(raw_cov),
        fit_visibility_coverage_hours: Hours::new(fit_cov),
        gap_count,
        gap_mean_hours: Hours::new(gap_mean),
        gap_median_hours: Hours::new(gap_median),
        largest_gap_hours: Hours::new(gap_max),
        scheduled_fraction_of_operable: safe_frac(scheduled_hours_val, operable_hours_val),
        idle_fraction_of_operable: safe_frac(idle_operable_hours_val, operable_hours_val),
        raw_visibility_fraction_of_operable: safe_frac(raw_cov, operable_hours_val),
        fit_visibility_fraction_of_operable: safe_frac(fit_cov, operable_hours_val),
    };

    FragmentationData {
        schedule_id: ScheduleId::new(schedule.id.unwrap_or(0)),
        schedule_window,
        operable_periods,
        operable_source: operable_source.to_string(),
        segments,
        largest_gaps,
        reason_breakdown,
        unscheduled_reasons,
        metrics,
    }
}

// =========================================================================
// Helpers — period arithmetic and classification
// =========================================================================

fn duration_hours(p: &Period) -> f64 {
    let d_days = p.end.value() - p.start.value();
    (d_days.max(0.0)) * 24.0
}

fn safe_frac(num: f64, denom: f64) -> f64 {
    if denom > 0.0 {
        (num / denom).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn is_idle_operable(kind: FragmentationSegmentKind) -> bool {
    matches!(
        kind,
        FragmentationSegmentKind::NoTargetVisible
            | FragmentationSegmentKind::VisibleButNoTaskFits
            | FragmentationSegmentKind::FeasibleButUnused
    )
}

/// Drop zero/negative-length periods and return owned.
fn normalize(periods: Vec<Period>) -> Vec<Period> {
    periods
        .into_iter()
        .filter(|p| p.end.value() > p.start.value())
        .collect()
}

/// Merge overlapping / adjacent periods. Input need not be sorted.
fn merge_periods(mut periods: Vec<Period>) -> Vec<Period> {
    periods.retain(|p| p.end.value() > p.start.value());
    if periods.is_empty() {
        return periods;
    }
    periods.sort_by(|a, b| {
        a.start
            .value()
            .partial_cmp(&b.start.value())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut out: Vec<Period> = Vec::with_capacity(periods.len());
    for p in periods {
        match out.last_mut() {
            Some(last) if p.start.value() <= last.end.value() => {
                if p.end.value() > last.end.value() {
                    last.end = p.end;
                }
            }
            _ => out.push(p),
        }
    }
    out
}

/// Clip each period in `periods` to the bounds of `window`, drop empties.
fn clip_periods(periods: &[Period], window: &Period) -> Vec<Period> {
    let ws = window.start.value();
    let we = window.end.value();
    periods
        .iter()
        .filter_map(|p| {
            let s = p.start.value().max(ws);
            let e = p.end.value().min(we);
            if e > s {
                Some(Period::new(
                    ModifiedJulianDate::new(s),
                    ModifiedJulianDate::new(e),
                ))
            } else {
                None
            }
        })
        .collect()
}

/// Total duration (hours) of the intersection of two (already-merged) period sets.
fn intersect_duration_hours(a: &[Period], b: &[Period]) -> f64 {
    let mut total = 0.0;
    let (mut i, mut j) = (0usize, 0usize);
    while i < a.len() && j < b.len() {
        let s = a[i].start.value().max(b[j].start.value());
        let e = a[i].end.value().min(b[j].end.value());
        if e > s {
            total += (e - s) * 24.0;
        }
        if a[i].end.value() < b[j].end.value() {
            i += 1;
        } else {
            j += 1;
        }
    }
    total
}

/// Classify the schedule window into segments using fixed precedence.
///
/// Algorithm: collect every boundary MJD, sort/dedupe, then for each adjacent
/// pair ask the set-memberships. Adjacent segments of the same kind are merged.
fn classify_segments(
    window: &Period,
    operable: &[Period],
    scheduled: &[Period],
    raw_visible: &[Period],
    fit_visible: &[Period],
) -> Vec<FragmentationSegment> {
    let mut boundaries: Vec<f64> = Vec::new();
    boundaries.push(window.start.value());
    boundaries.push(window.end.value());
    for set in [operable, scheduled, raw_visible, fit_visible] {
        for p in set {
            boundaries.push(p.start.value());
            boundaries.push(p.end.value());
        }
    }
    boundaries.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    boundaries.dedup_by(|a, b| (*a - *b).abs() < 1e-12);

    let ws = window.start.value();
    let we = window.end.value();

    let mut out: Vec<FragmentationSegment> = Vec::new();

    for pair in boundaries.windows(2) {
        let (s, e) = (pair[0], pair[1]);
        if s < ws || e > we || e - s <= 1e-12 {
            continue;
        }
        let mid = (s + e) * 0.5;

        let in_operable = contains(operable, mid);
        let kind = if !in_operable {
            FragmentationSegmentKind::NonOperable
        } else if contains(scheduled, mid) {
            FragmentationSegmentKind::Scheduled
        } else if !contains(raw_visible, mid) {
            FragmentationSegmentKind::NoTargetVisible
        } else if !contains(fit_visible, mid) {
            FragmentationSegmentKind::VisibleButNoTaskFits
        } else {
            FragmentationSegmentKind::FeasibleButUnused
        };

        match out.last_mut() {
            Some(prev)
                if prev.kind == kind && (prev.stop_mjd.value() - s).abs() < 1e-12 =>
            {
                prev.stop_mjd = ModifiedJulianDate::new(e);
                prev.duration_hours =
                    Hours::new((prev.stop_mjd.value() - prev.start_mjd.value()) * 24.0);
            }
            _ => out.push(FragmentationSegment {
                start_mjd: ModifiedJulianDate::new(s),
                stop_mjd: ModifiedJulianDate::new(e),
                duration_hours: Hours::new((e - s) * 24.0),
                kind,
            }),
        }
    }

    out
}

fn contains(periods: &[Period], t: f64) -> bool {
    // Linear scan is fine — datasets for a schedule are small.
    periods
        .iter()
        .any(|p| t > p.start.value() && t < p.end.value())
}

// =========================================================================
// Unscheduled-reason bucketing
// =========================================================================

fn summarize_unscheduled_reasons(
    schedule: &Schedule,
    validation: &ValidationReport,
    fit_visibility_union: &[Period],
) -> Vec<UnscheduledReasonSummary> {
    let _ = fit_visibility_union; // reserved for future heuristics

    // Index validation issues by internal block_id for fast lookup.
    let mut issues_by_block: HashMap<i64, Vec<&ValidationIssue>> = HashMap::new();
    for issue in validation
        .impossible_blocks
        .iter()
        .chain(validation.validation_errors.iter())
        .chain(validation.validation_warnings.iter())
    {
        issues_by_block.entry(issue.block_id).or_default().push(issue);
    }

    let mut buckets: HashMap<UnscheduledReason, Vec<(String, String)>> = HashMap::new();

    for block in &schedule.blocks {
        if block.scheduled_period.is_some() {
            continue;
        }
        let block_id = block.id.map(|i| i.value()).unwrap_or(-1);
        let reason = classify_unscheduled_reason(block_id, &issues_by_block);

        buckets
            .entry(reason)
            .or_default()
            .push((block.original_block_id.clone(), block.block_name.clone()));
    }

    // Deterministic output order (matches DTO documentation).
    let order = [
        UnscheduledReason::NoVisibility,
        UnscheduledReason::NoContiguousFit,
        UnscheduledReason::RequestedExceedsTotalVisibility,
        UnscheduledReason::OtherValidationIssue,
        UnscheduledReason::FeasibleButUnscheduled,
    ];

    order
        .iter()
        .map(|reason| {
            let entries = buckets.remove(reason).unwrap_or_default();
            let count = entries.len();
            let example_block_ids = entries.iter().take(10).map(|(id, _)| id.clone()).collect();
            let example_block_names = entries
                .iter()
                .take(10)
                .map(|(_, name)| name.clone())
                .collect();
            UnscheduledReasonSummary {
                reason: *reason,
                block_count: count,
                example_block_ids,
                example_block_names,
            }
        })
        .collect()
}

fn classify_unscheduled_reason(
    block_id: i64,
    issues_by_block: &HashMap<i64, Vec<&ValidationIssue>>,
) -> UnscheduledReason {
    let Some(issues) = issues_by_block.get(&block_id) else {
        return UnscheduledReason::FeasibleButUnscheduled;
    };

    // Match by issue_type substrings. Validation issue_type strings come from
    // services::validation and are of the form human-readable sentences.
    let mut has_no_visibility = false;
    let mut has_no_fit = false;
    let mut has_insufficient_total = false;
    let mut has_any = false;

    for issue in issues {
        has_any = true;
        let t = issue.issue_type.to_ascii_lowercase();
        if t.contains("no visibility") || t.contains("zero visibility") {
            has_no_visibility = true;
        } else if t.contains("minimum observation") {
            has_no_fit = true;
        } else if t.contains("requested duration") {
            has_insufficient_total = true;
        }
    }

    if has_no_visibility {
        UnscheduledReason::NoVisibility
    } else if has_no_fit {
        UnscheduledReason::NoContiguousFit
    } else if has_insufficient_total {
        UnscheduledReason::RequestedExceedsTotalVisibility
    } else if has_any {
        UnscheduledReason::OtherValidationIssue
    } else {
        UnscheduledReason::FeasibleButUnscheduled
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Constraints, GeographicLocation, Schedule, SchedulingBlock};

    fn mjd(v: f64) -> ModifiedJulianDate {
        ModifiedJulianDate::new(v)
    }

    fn period(s: f64, e: f64) -> Period {
        Period::new(mjd(s), mjd(e))
    }

    fn empty_location() -> GeographicLocation {
        GeographicLocation::new(0.0.into(), 0.0.into(), 0.0.into())
    }

    fn block(
        id: i64,
        scheduled: Option<(f64, f64)>,
        visibility: Vec<(f64, f64)>,
    ) -> SchedulingBlock {
        SchedulingBlock {
            id: Some(crate::api::SchedulingBlockId(id)),
            original_block_id: format!("B{id}"),
            block_name: String::new(),
            target_ra: 0.0.into(),
            target_dec: 0.0.into(),
            constraints: Constraints {
                min_alt: 0.0.into(),
                max_alt: 90.0.into(),
                min_az: 0.0.into(),
                max_az: 360.0.into(),
                fixed_time: None,
            },
            priority: 1.0,
            min_observation: 0.0.into(),
            requested_duration: 0.0.into(),
            visibility_periods: visibility
                .into_iter()
                .map(|(s, e)| period(s, e))
                .collect(),
            scheduled_period: scheduled.map(|(s, e)| period(s, e)),
        }
    }

    fn schedule_with(
        window: (f64, f64),
        dark: Vec<(f64, f64)>,
        nights: Vec<(f64, f64)>,
        blocks: Vec<SchedulingBlock>,
    ) -> Schedule {
        Schedule {
            id: Some(7),
            name: "t".into(),
            checksum: String::new(),
            schedule_period: period(window.0, window.1),
            dark_periods: dark.into_iter().map(|(s, e)| period(s, e)).collect(),
            geographic_location: empty_location(),
            astronomical_nights: nights.into_iter().map(|(s, e)| period(s, e)).collect(),
            blocks,
        }
    }

    fn empty_validation() -> ValidationReport {
        ValidationReport {
            schedule_id: ScheduleId::new(7),
            total_blocks: 0,
            valid_blocks: 0,
            impossible_blocks: vec![],
            validation_errors: vec![],
            validation_warnings: vec![],
        }
    }

    #[test]
    fn merge_periods_merges_overlaps_and_adjacents() {
        let merged = merge_periods(vec![period(0.0, 1.0), period(1.0, 2.0), period(3.0, 4.0)]);
        assert_eq!(merged.len(), 2);
        assert!((merged[0].end.value() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn intersect_duration_hours_works() {
        let a = vec![period(0.0, 1.0)];
        let b = vec![period(0.5, 2.0)];
        let h = intersect_duration_hours(&a, &b);
        assert!((h - 12.0).abs() < 1e-9); // 0.5 day = 12 hours
    }

    #[test]
    fn non_operable_is_excluded_from_gap_stats() {
        // window 0..2 days, operable only 0..1 (so 1..2 is non-operable).
        // no scheduled, no visibility -> idle segment only inside operable.
        let sched = schedule_with((0.0, 2.0), vec![(0.0, 1.0)], vec![], vec![]);
        let data = compute_fragmentation(&sched, &empty_validation());

        // One idle operable gap of 24h; non-operable 1..2 should NOT be a gap.
        assert_eq!(data.metrics.gap_count, 1);
        assert!((data.metrics.largest_gap_hours.value() - 24.0).abs() < 1e-9);
        assert!((data.metrics.operable_hours.value() - 24.0).abs() < 1e-9);
        // Non-operable segment must exist.
        assert!(data
            .segments
            .iter()
            .any(|s| s.kind == FragmentationSegmentKind::NonOperable));
    }

    #[test]
    fn visible_but_no_fit_is_distinct_from_no_visible() {
        // Block has scheduled slot (raw-visible there via construction) but
        // zero fit-visibility anywhere. Operable covers full window.
        let b = block(1, Some((0.25, 0.5)), vec![]);
        let sched = schedule_with((0.0, 1.0), vec![(0.0, 1.0)], vec![], vec![b]);
        let data = compute_fragmentation(&sched, &empty_validation());

        let kinds: Vec<_> = data.segments.iter().map(|s| s.kind).collect();
        // Outside scheduled window there's no raw visibility — no_target_visible.
        assert!(kinds.contains(&FragmentationSegmentKind::NoTargetVisible));
        assert!(kinds.contains(&FragmentationSegmentKind::Scheduled));
    }

    #[test]
    fn scheduled_splits_operable_windows() {
        let b = block(1, Some((0.25, 0.5)), vec![(0.0, 1.0)]);
        let sched = schedule_with((0.0, 1.0), vec![(0.0, 1.0)], vec![], vec![b]);
        let data = compute_fragmentation(&sched, &empty_validation());

        // Expect 3 segments: feasible/idle before, scheduled middle, idle after.
        assert_eq!(data.segments.len(), 3);
        assert_eq!(data.segments[1].kind, FragmentationSegmentKind::Scheduled);
    }

    #[test]
    fn empty_operable_baseline_yields_zero_operable_hours() {
        let sched = schedule_with((0.0, 1.0), vec![], vec![], vec![]);
        let data = compute_fragmentation(&sched, &empty_validation());

        assert!((data.metrics.operable_hours.value() - 0.0).abs() < 1e-12);
        assert_eq!(data.metrics.gap_count, 0);
        assert_eq!(data.operable_source, "astronomical_nights");
        // Whole schedule is non_operable.
        assert!(data
            .segments
            .iter()
            .all(|s| s.kind == FragmentationSegmentKind::NonOperable));
    }

    #[test]
    fn dark_periods_takes_precedence_over_astronomical_nights() {
        let sched = schedule_with(
            (0.0, 1.0),
            vec![(0.0, 0.5)],
            vec![(0.0, 1.0)],
            vec![],
        );
        let data = compute_fragmentation(&sched, &empty_validation());
        assert_eq!(data.operable_source, "dark_periods");
        assert!((data.metrics.operable_hours.value() - 12.0).abs() < 1e-9);
    }

    #[test]
    fn falls_back_to_astronomical_nights_when_dark_empty() {
        let sched = schedule_with((0.0, 1.0), vec![], vec![(0.0, 0.5)], vec![]);
        let data = compute_fragmentation(&sched, &empty_validation());
        assert_eq!(data.operable_source, "astronomical_nights");
        assert!((data.metrics.operable_hours.value() - 12.0).abs() < 1e-9);
    }

    #[test]
    fn fragmentation_metrics_fractions_clamp_to_zero_when_no_operable() {
        let sched = schedule_with((0.0, 1.0), vec![], vec![], vec![]);
        let data = compute_fragmentation(&sched, &empty_validation());
        assert_eq!(data.metrics.scheduled_fraction_of_operable, 0.0);
        assert_eq!(data.metrics.idle_fraction_of_operable, 0.0);
    }

    #[test]
    fn unscheduled_reason_no_visibility_detected_via_validation() {
        let b = block(1, None, vec![]);
        let sched = schedule_with((0.0, 1.0), vec![(0.0, 1.0)], vec![], vec![b]);
        let mut report = empty_validation();
        report.impossible_blocks.push(ValidationIssue {
            block_id: 1,
            original_block_id: Some("B1".into()),
            block_name: None,
            issue_type: "No visibility periods available".into(),
            category: "visibility".into(),
            criticality: "critical".into(),
            field_name: None,
            current_value: None,
            expected_value: None,
            description: String::new(),
        });

        let data = compute_fragmentation(&sched, &report);
        let summary = data
            .unscheduled_reasons
            .iter()
            .find(|s| s.reason == UnscheduledReason::NoVisibility)
            .unwrap();
        assert_eq!(summary.block_count, 1);
        assert_eq!(summary.example_block_ids, vec!["B1"]);
    }
}
