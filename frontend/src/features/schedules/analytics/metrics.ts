/**
 * Headline-metric registry used by the comparison and Pareto panels.
 *
 * Each entry knows how to extract its value from a `ScheduleAnalysisData`
 * row, how to render it, what axis label to use in plots, and whether
 * higher or lower is better (for sort + dominance computations).
 *
 * Adding a metric here automatically lets it appear in the metric
 * picker, the Pareto axis pickers, and the sweep/sensitivity colour
 * scales.
 */
import type { ScheduleAnalysisData } from '../hooks/useScheduleAnalysisData';

export type MetricDirection = 'max' | 'min';

export interface MetricSpec {
  /** Stable id used in URL state and option pickers. */
  key: string;
  /** Short label rendered in pickers and chart titles. */
  label: string;
  /** Optional longer description for tooltips. */
  description?: string;
  /** Axis title (includes unit suffix when applicable). */
  axisTitle: string;
  /** Optional unit appended after numeric values. */
  unit?: string;
  /** Higher is better (max) or lower is better (min). */
  direction: MetricDirection;
  /** Returns the metric value for a schedule, or null when unavailable. */
  getValue: (s: ScheduleAnalysisData) => number | null;
  /** Formats a value for display in tables / hover labels. */
  format: (v: number | null) => string;
}

const fmtPct = (v: number | null): string =>
  v == null || !Number.isFinite(v) ? '—' : `${(v * 100).toFixed(1)} %`;
const fmtNum = (digits: number) => (v: number | null): string =>
  v == null || !Number.isFinite(v) ? '—' : v.toFixed(digits);
const fmtInt = (v: number | null): string =>
  v == null || !Number.isFinite(v) ? '—' : Math.round(v).toLocaleString();
const fmtHours = (v: number | null): string =>
  v == null || !Number.isFinite(v) ? '—' : `${v.toFixed(2)} h`;

function cumulativePriority(s: ScheduleAnalysisData): number | null {
  if (!s.insights) return null;
  // Prefer the metric reported by the API when present.
  const m = s.insights.metrics.sum_priority_scheduled;
  if (typeof m === 'number' && Number.isFinite(m)) return m;
  return s.insights.blocks
    .filter((b) => b.scheduled)
    .reduce((sum, b) => sum + b.priority, 0);
}

export const METRIC_SCHEDULING_RATE: MetricSpec = {
  key: 'scheduling_rate',
  label: 'Scheduling rate',
  description: 'Fraction of tasks placed in the schedule.',
  axisTitle: 'Scheduling rate (%)',
  unit: '%',
  direction: 'max',
  getValue: (s) =>
    s.insights ? s.insights.metrics.scheduling_rate * 100 : null,
  format: (v) => (v == null ? '—' : `${v.toFixed(1)} %`),
};

export const METRIC_PRIORITY_CAPTURE: MetricSpec = {
  key: 'priority_capture_ratio',
  label: 'Priority capture',
  description:
    'Σ priority(scheduled) / Σ priority(all). Rewards schedules that ' +
    'pick the highest-priority tasks.',
  axisTitle: 'Priority capture (%)',
  unit: '%',
  direction: 'max',
  getValue: (s) =>
    s.insights ? s.insights.metrics.priority_capture_ratio * 100 : null,
  format: (v) => (v == null ? '—' : `${v.toFixed(1)} %`),
};

export const METRIC_CUMULATIVE_PRIORITY: MetricSpec = {
  key: 'cumulative_priority_sum',
  label: 'Cumulative priority',
  description: 'Σ priority over scheduled blocks.',
  axisTitle: 'Σ priority (scheduled)',
  direction: 'max',
  getValue: cumulativePriority,
  format: fmtNum(2),
};

export const METRIC_MEAN_PRIORITY: MetricSpec = {
  key: 'mean_priority_scheduled',
  label: 'Mean priority (sched.)',
  description: 'Average priority over scheduled blocks.',
  axisTitle: 'Mean priority (scheduled)',
  direction: 'max',
  getValue: (s) => s.insights?.metrics.mean_priority_scheduled ?? null,
  format: fmtNum(2),
};

export const METRIC_SCHEDULED_COUNT: MetricSpec = {
  key: 'scheduled_count',
  label: 'Scheduled count',
  description: 'Number of scheduled blocks.',
  axisTitle: 'Scheduled count',
  direction: 'max',
  getValue: (s) => s.insights?.metrics.scheduled_count ?? null,
  format: fmtInt,
};

export const METRIC_SCHEDULED_HOURS: MetricSpec = {
  key: 'scheduled_hours',
  label: 'Scheduled hours',
  axisTitle: 'Hours',
  unit: 'h',
  direction: 'max',
  getValue: (s) => s.fragmentation?.metrics.scheduled_hours ?? null,
  format: fmtHours,
};

export const METRIC_GAP_COUNT: MetricSpec = {
  key: 'gap_count',
  label: 'Gap count',
  description: 'Number of idle gaps between scheduled blocks.',
  axisTitle: 'Gaps',
  direction: 'min',
  getValue: (s) => s.fragmentation?.metrics.gap_count ?? null,
  format: fmtInt,
};

export const METRIC_IDLE_FRACTION: MetricSpec = {
  key: 'idle_fraction_of_operable',
  label: 'Idle fraction',
  description: 'Idle operable hours / operable hours.',
  axisTitle: 'Idle fraction of operable',
  direction: 'min',
  getValue: (s) =>
    s.fragmentation
      ? s.fragmentation.metrics.idle_fraction_of_operable
      : null,
  format: fmtPct,
};

export const ALL_METRICS: MetricSpec[] = [
  METRIC_SCHEDULING_RATE,
  METRIC_PRIORITY_CAPTURE,
  METRIC_CUMULATIVE_PRIORITY,
  METRIC_MEAN_PRIORITY,
  METRIC_SCHEDULED_COUNT,
  METRIC_SCHEDULED_HOURS,
  METRIC_GAP_COUNT,
  METRIC_IDLE_FRACTION,
];

export const METRICS_BY_KEY: Record<string, MetricSpec> = Object.fromEntries(
  ALL_METRICS.map((m) => [m.key, m]),
);

/** Default metric set surfaced in the comparison metric picker. */
export const DEFAULT_COMPARISON_METRIC_KEYS = [
  METRIC_SCHEDULING_RATE.key,
  METRIC_PRIORITY_CAPTURE.key,
  METRIC_CUMULATIVE_PRIORITY.key,
];

export function getMetric(key: string): MetricSpec | undefined {
  return METRICS_BY_KEY[key];
}
