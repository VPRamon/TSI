//! Schedule KPI service (A1).
//!
//! Computes a small, comparable set of Key Performance Indicators for a
//! single schedule, derived on-the-fly from the existing analytics
//! services (`insights` + `fragmentation`). The output is shaped for the
//! Workspace verdict / delta / evolution UIs (A2/A3) and is intentionally
//! cheap to compute in batch via the per-environment endpoint.
//!
//! ## Design
//!
//! * **No new DB table this iteration.** The KPIs are pure functions of
//!   the per-block analytics rows that are already populated when a
//!   schedule is stored, so persisting them again would just duplicate
//!   data. A future change can materialise a `schedule_kpis` table for
//!   sub-millisecond batched reads if profiling shows it matters.
//! * **Composite score uses equal weights across normalized components.**
//!   The components are exposed alongside the score so the frontend can
//!   re-weight without another round-trip.
//! * **All component values are normalized to `[0, 1]`, "higher is
//!   better".** Missing or pathological inputs collapse to `0.0` so
//!   incomplete schedules never falsely win the verdict.

use serde::{Deserialize, Serialize};

use crate::api::{FragmentationData, InsightsData, ScheduleId};
use crate::db::FullRepository;

/// Per-component contributions to the composite score. Each value is in
/// `[0, 1]`, where 1.0 is "perfect on this axis". Sums of weighted
/// components form `composite_score`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreComponents {
    /// Fraction of observations that ended up scheduled. From insights
    /// metrics. Higher = more requests satisfied.
    pub scheduling_rate: f64,
    /// Fraction of operable time that is actually scheduled. From
    /// fragmentation metrics. Higher = better operable-time use.
    pub scheduled_fraction_of_operable: f64,
    /// `scheduled_hours / fit_visibility_coverage_hours`, capped at 1.
    /// Higher = better use of the time when a full min-observation would
    /// fit. 0.0 if there is no fit-visibility coverage.
    pub fit_visibility_utilisation: f64,
    /// `mean_priority_scheduled / mean_priority_total`, capped at 1.
    /// Higher = the scheduler preferentially picked the higher-priority
    /// observations. 0.0 when no observations are scheduled.
    pub priority_alignment: f64,
    /// `1 - gap_p90_hours / operable_hours`, clamped to `[0, 1]`.
    /// Higher = the schedule has fewer / smaller idle gaps relative to
    /// operable time. 0.0 if `operable_hours == 0`.
    pub gap_compactness: f64,
}

impl ScoreComponents {
    /// Equal-weights mean across the five components. NaN values are
    /// treated as 0.0 so a single missing axis doesn't poison the score.
    pub fn equal_weights_mean(&self) -> f64 {
        let parts = [
            self.scheduling_rate,
            self.scheduled_fraction_of_operable,
            self.fit_visibility_utilisation,
            self.priority_alignment,
            self.gap_compactness,
        ];
        let sum: f64 = parts.iter().map(|v| sanitise(*v)).sum();
        sum / parts.len() as f64
    }
}

/// Compact KPI summary for one schedule, suitable for batched per-env
/// reads and side-by-side comparison in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleKpi {
    pub schedule_id: ScheduleId,
    pub schedule_name: String,
    /// Total number of observations considered (= `total_count` from
    /// insights, after impossible-block filtering).
    pub total_observations: usize,
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub operable_hours: f64,
    pub scheduled_hours: f64,
    pub idle_operable_hours: f64,
    pub gap_count: usize,
    pub gap_p90_hours: f64,
    pub largest_gap_hours: f64,
    pub scheduling_rate: f64,
    pub scheduled_fraction_of_operable: f64,
    pub fit_visibility_fraction_of_operable: f64,
    pub mean_priority_scheduled: f64,
    pub mean_priority_unscheduled: f64,
    /// `Σ priority(scheduled) / Σ priority(all)` for this schedule. Lives in
    /// [0, 1] for non-negative priorities; 0.0 when no priority mass exists.
    /// Preferred over `mean_priority_scheduled` when comparing schedules
    /// across an environment.
    pub priority_capture_ratio: f64,
    /// Equal-weights default. The frontend may recompute from
    /// `score_components` if the user re-weights.
    pub composite_score: f64,
    pub score_components: ScoreComponents,
}

/// Response wrapper for the per-environment KPI endpoint. Failures for
/// individual schedules are surfaced inline in `errors` so a single
/// broken schedule doesn't blank out the whole comparison page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentKpisResponse {
    pub environment_id: i64,
    pub kpis: Vec<ScheduleKpi>,
    pub errors: Vec<EnvironmentKpiError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentKpiError {
    pub schedule_id: i64,
    pub reason: String,
}

/// Compute the KPI summary for a single schedule. This delegates to the
/// existing insights and fragmentation services, so it inherits their
/// cache / pre-computation behaviour.
pub async fn compute_kpi_summary(
    repo: &(dyn FullRepository + 'static),
    schedule_id: ScheduleId,
) -> Result<ScheduleKpi, String> {
    let insights = crate::services::insights::get_insights_data(repo, schedule_id)
        .await
        .map_err(|e| format!("insights: {e}"))?;
    let fragmentation = crate::services::fragmentation::get_fragmentation_data(repo, schedule_id)
        .await
        .map_err(|e| format!("fragmentation: {e}"))?;
    Ok(kpi_from_parts(schedule_id, &insights, &fragmentation))
}

/// Pure derivation from insights + fragmentation. Split out so it can be
/// unit-tested without touching the DB.
pub fn kpi_from_parts(
    schedule_id: ScheduleId,
    insights: &InsightsData,
    fragmentation: &FragmentationData,
) -> ScheduleKpi {
    let metrics = &fragmentation.metrics;
    let im = &insights.metrics;

    let operable_hours = metrics.operable_hours.value();
    let scheduled_hours = metrics.scheduled_hours.value();
    let fit_cov_hours = metrics.fit_visibility_coverage_hours.value();
    let unscheduled_count = im.total_observations.saturating_sub(im.scheduled_count);

    // Component 1: scheduling rate (already in [0,1]).
    let scheduling_rate = clamp01(im.scheduling_rate);

    // Component 2: scheduled fraction of operable (already in [0,1]).
    let scheduled_fraction_of_operable = clamp01(metrics.scheduled_fraction_of_operable);

    // Component 3: fit-visibility utilisation. How much of the time
    // when a full observation would have fit is actually scheduled.
    let fit_visibility_utilisation = if fit_cov_hours > 0.0 {
        clamp01(scheduled_hours / fit_cov_hours)
    } else {
        0.0
    };

    // Component 4: priority alignment. Mean priority of scheduled
    // observations vs the population mean. >1 means we picked the more
    // important targets; we cap at 1.0 to avoid letting a tiny
    // high-priority subset dominate.
    let priority_alignment = if im.mean_priority > 0.0 && im.scheduled_count > 0 {
        clamp01(im.mean_priority_scheduled / im.mean_priority)
    } else {
        0.0
    };

    // Component 5: gap compactness. p90 gap as a fraction of operable
    // time, inverted. A schedule whose worst-decile gap eats half the
    // operable time gets 0.5; one with no gaps gets 1.0.
    let gap_compactness = if operable_hours > 0.0 {
        clamp01(1.0 - metrics.gap_p90_hours.value() / operable_hours)
    } else {
        0.0
    };

    let score_components = ScoreComponents {
        scheduling_rate,
        scheduled_fraction_of_operable,
        fit_visibility_utilisation,
        priority_alignment,
        gap_compactness,
    };
    let composite_score = score_components.equal_weights_mean();

    ScheduleKpi {
        schedule_id,
        schedule_name: fragmentation.schedule_name.clone(),
        total_observations: im.total_observations,
        scheduled_count: im.scheduled_count,
        unscheduled_count,
        operable_hours,
        scheduled_hours,
        idle_operable_hours: metrics.idle_operable_hours.value(),
        gap_count: metrics.gap_count,
        gap_p90_hours: metrics.gap_p90_hours.value(),
        largest_gap_hours: metrics.largest_gap_hours.value(),
        scheduling_rate: im.scheduling_rate,
        scheduled_fraction_of_operable: metrics.scheduled_fraction_of_operable,
        fit_visibility_fraction_of_operable: metrics.fit_visibility_fraction_of_operable,
        mean_priority_scheduled: im.mean_priority_scheduled,
        mean_priority_unscheduled: im.mean_priority_unscheduled,
        priority_capture_ratio: im.priority_capture_ratio,
        composite_score,
        score_components,
    }
}

#[inline]
fn clamp01(x: f64) -> f64 {
    if x.is_nan() {
        0.0
    } else {
        x.clamp(0.0, 1.0)
    }
}

#[inline]
fn sanitise(x: f64) -> f64 {
    if x.is_nan() {
        0.0
    } else {
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{
        AnalyticsMetrics, FragmentationData, FragmentationMetrics, InsightsData,
        ModifiedJulianDate, Period,
    };
    use qtty::Hours;

    fn make_insights(scheduled: usize, total: usize, mean_pri_sched: f64) -> InsightsData {
        InsightsData {
            blocks: vec![],
            metrics: AnalyticsMetrics {
                total_observations: total,
                scheduled_count: scheduled,
                unscheduled_count: total - scheduled,
                scheduling_rate: scheduled as f64 / total as f64,
                mean_priority: 5.0,
                median_priority: 5.0,
                mean_priority_scheduled: mean_pri_sched,
                mean_priority_unscheduled: 5.0,
                priority_capture_ratio: if total > 0 {
                    (mean_pri_sched * scheduled as f64) / (5.0 * total as f64)
                } else {
                    0.0
                },
                sum_priority_scheduled: mean_pri_sched * scheduled as f64,
                sum_priority_total: 5.0 * total as f64,
                total_visibility_hours: Hours::new(100.0),
                mean_requested_hours: Hours::new(2.0),
            },
            correlations: vec![],
            top_priority: vec![],
            top_visibility: vec![],
            conflicts: vec![],
            total_count: total,
            scheduled_count: scheduled,
            impossible_count: 0,
        }
    }

    fn make_fragmentation(
        operable: f64,
        scheduled: f64,
        fit_cov: f64,
        gap_p90: f64,
    ) -> FragmentationData {
        FragmentationData {
            schedule_id: ScheduleId::new(1),
            schedule_name: "test".into(),
            schedule_window: Period {
                start: ModifiedJulianDate::new(0.0),
                end: ModifiedJulianDate::new(1.0),
            },
            operable_periods: vec![],
            operable_source: "astronomical_nights".into(),
            segments: vec![],
            largest_gaps: vec![],
            reason_breakdown: vec![],
            unscheduled_reasons: vec![],
            metrics: FragmentationMetrics {
                schedule_hours: Hours::new(scheduled),
                requested_hours: Hours::new(scheduled * 2.0),
                operable_hours: Hours::new(operable),
                scheduled_hours: Hours::new(scheduled),
                idle_operable_hours: Hours::new((operable - scheduled).max(0.0)),
                raw_visibility_coverage_hours: Hours::new(fit_cov * 1.2),
                fit_visibility_coverage_hours: Hours::new(fit_cov),
                gap_count: 3,
                gap_mean_hours: Hours::new(gap_p90 * 0.5),
                gap_median_hours: Hours::new(gap_p90 * 0.4),
                gap_std_dev_hours: Hours::new(0.1),
                gap_p90_hours: Hours::new(gap_p90),
                largest_gap_hours: Hours::new(gap_p90 * 1.5),
                scheduled_fraction_of_operable: if operable > 0.0 {
                    scheduled / operable
                } else {
                    0.0
                },
                idle_fraction_of_operable: if operable > 0.0 {
                    (operable - scheduled).max(0.0) / operable
                } else {
                    0.0
                },
                raw_visibility_fraction_of_operable: if operable > 0.0 {
                    fit_cov * 1.2 / operable
                } else {
                    0.0
                },
                fit_visibility_fraction_of_operable: if operable > 0.0 {
                    fit_cov / operable
                } else {
                    0.0
                },
            },
        }
    }

    #[test]
    fn perfect_schedule_scores_one() {
        let insights = make_insights(10, 10, 10.0);
        let fragmentation = make_fragmentation(10.0, 10.0, 10.0, 0.0);
        let kpi = kpi_from_parts(ScheduleId::new(1), &insights, &fragmentation);
        assert!(
            (kpi.composite_score - 1.0).abs() < 1e-9,
            "expected perfect score, got {}",
            kpi.composite_score
        );
    }

    #[test]
    fn empty_schedule_scores_zero() {
        let insights = make_insights(0, 10, 0.0);
        let fragmentation = make_fragmentation(10.0, 0.0, 0.0, 10.0);
        let kpi = kpi_from_parts(ScheduleId::new(1), &insights, &fragmentation);
        assert!(
            kpi.composite_score < 0.05,
            "expected near-zero score, got {}",
            kpi.composite_score
        );
        assert_eq!(kpi.unscheduled_count, 10);
    }

    #[test]
    fn components_are_clamped_to_unit_interval() {
        // mean_priority_scheduled > mean_priority should still cap at 1
        let insights = make_insights(5, 10, 100.0);
        let fragmentation = make_fragmentation(10.0, 5.0, 5.0, 1.0);
        let kpi = kpi_from_parts(ScheduleId::new(1), &insights, &fragmentation);
        assert!(kpi.score_components.priority_alignment <= 1.0);
        assert!(kpi.score_components.fit_visibility_utilisation <= 1.0);
        assert!(kpi.score_components.gap_compactness >= 0.0);
    }

    #[test]
    fn nan_inputs_do_not_poison_score() {
        let comps = ScoreComponents {
            scheduling_rate: f64::NAN,
            scheduled_fraction_of_operable: 0.5,
            fit_visibility_utilisation: 0.5,
            priority_alignment: 0.5,
            gap_compactness: 0.5,
        };
        let m = comps.equal_weights_mean();
        assert!(m.is_finite());
        assert!((m - 0.4).abs() < 1e-9);
    }
}
