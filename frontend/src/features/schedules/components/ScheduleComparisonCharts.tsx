/**
 * Visual comparison charts for the Compare and EnvironmentCompare pages.
 *
 * Three composed panels:
 *   A – Key Metrics (configurable picker over the metric registry).
 *   B – Priority Distribution (box plot).
 *   E – Time-Use Breakdown (100 % stacked bar).
 *
 * Above the panels, two filter rows let users restrict / annotate the
 * displayed schedule set:
 *   - **Configuration filters** — numeric range sliders + categorical
 *     chip selectors derived from each schedule's `algorithm_config`.
 *   - **Plot filters** — the legacy priority range and scheduled-hours
 *     window. These apply on top of the configuration filters.
 *
 * A toolbar exposes:
 *   - Metric selection (which entries from the metric registry to draw).
 *   - "Group by" (none / a config dimension) for aggregated traces.
 *   - "Collapse equivalent schedules" to merge runs that produced the
 *     same scheduled-task set.
 *   - "Sort by" + horizontal-bar mode for legibility on dense sets.
 */
import { useMemo, useState } from 'react';
import { ChartPanel, PlotlyChart, RangeFilterGroup } from '@/components';
import type { RangeFilterSpec, RangeFilterValue } from '@/components';
import { initialRangeValues } from '@/components';
import { usePlotlyChartChrome, usePlotlyTheme } from '@/hooks';
import type { FragmentationSegmentKind } from '@/api/types';
import type { ScheduleAnalysisData } from '../hooks/useScheduleAnalysisData';
import {
  ALL_METRICS,
  CategoricalFilterGroup,
  DEFAULT_COMPARISON_METRIC_KEYS,
  collapseToRepresentatives,
  getMetric,
  groupEquivalentSchedules,
  type Dimension,
  type MetricSpec,
  useConfigFilters,
} from '../analytics';
import {
  COMPARISON_FILTER_HELP,
  KEY_METRICS_HELP,
  PRIORITY_BOX_HELP,
  TIME_USE_HELP,
} from './comparisonChartHelp';
import { HelpPopover } from '@/components/charts';

const SCHEDULE_PALETTE = [
  '#38bdf8', '#a78bfa', '#34d399', '#fbbf24', '#f472b6',
  '#60a5fa', '#fb923c', '#a3e635', '#f87171', '#22d3ee',
];

const GROUP_PALETTE = [
  '#38bdf8', '#a78bfa', '#34d399', '#fbbf24', '#f472b6',
  '#fb923c', '#a3e635', '#22d3ee', '#f87171', '#c084fc',
];

function colorFor(palette: string[], index: number): string {
  return palette[index % palette.length];
}

// ─── Plot filter (priority range + scheduled hours) ─────────────────────────

interface PlotFilters {
  priority: RangeFilterValue | undefined;
  scheduledHours: RangeFilterValue | undefined;
}

function usePlotFilters(schedules: ScheduleAnalysisData[]): {
  specs: RangeFilterSpec[];
  values: Record<string, RangeFilterValue>;
  setValues: (next: Record<string, RangeFilterValue>) => void;
  filters: PlotFilters;
} {
  const specs = useMemo<RangeFilterSpec[]>(() => {
    const priorities: number[] = [];
    const hours: number[] = [];
    for (const s of schedules) {
      if (s.insights) {
        for (const block of s.insights.blocks) {
          if (block.scheduled) priorities.push(block.priority);
        }
      }
      if (s.fragmentation?.metrics.scheduled_hours != null) {
        hours.push(s.fragmentation.metrics.scheduled_hours);
      }
    }
    const out: RangeFilterSpec[] = [];
    if (priorities.length > 0) {
      out.push({ key: 'priority', label: 'Priority range', values: priorities });
    }
    if (hours.length > 0) {
      out.push({
        key: 'scheduledHours',
        label: 'Min scheduled hours',
        values: hours,
        unit: 'h',
      });
    }
    return out;
  }, [schedules]);

  const initial = useMemo(() => initialRangeValues(specs), [specs]);
  const [values, setValues] = useState<Record<string, RangeFilterValue>>(initial);

  const seedSignature = useMemo(
    () => specs.map((s) => `${s.key}:${s.values.length}`).join('|'),
    [specs],
  );
  const [lastSeed, setLastSeed] = useState(seedSignature);
  if (seedSignature !== lastSeed) {
    setLastSeed(seedSignature);
    setValues(initial);
  }

  return {
    specs,
    values,
    setValues,
    filters: { priority: values.priority, scheduledHours: values.scheduledHours },
  };
}

// ─── Aggregation by group dimension (median + IQR band) ─────────────────────

interface AggBucket {
  key: string;
  members: ScheduleAnalysisData[];
}

function bucketBy(
  schedules: ScheduleAnalysisData[],
  dim: Dimension | null,
): AggBucket[] {
  if (!dim) return schedules.map((s) => ({ key: s.name, members: [s] }));
  const buckets = new Map<string, ScheduleAnalysisData[]>();
  for (const s of schedules) {
    const raw = s.algorithmConfig?.[dim.key];
    const k = raw == null || raw === '' ? '∅' : `${dim.key}=${String(raw)}`;
    const arr = buckets.get(k) ?? [];
    arr.push(s);
    buckets.set(k, arr);
  }
  return Array.from(buckets.entries()).map(([key, members]) => ({ key, members }));
}

function quantile(sorted: number[], q: number): number {
  if (sorted.length === 0) return NaN;
  const pos = (sorted.length - 1) * q;
  const lo = Math.floor(pos);
  const hi = Math.ceil(pos);
  if (lo === hi) return sorted[lo];
  return sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo);
}

interface AggSummary {
  median: number;
  q1: number;
  q3: number;
  count: number;
}

function summarise(values: Array<number | null>): AggSummary | null {
  const finite = values.filter((v): v is number => v != null && Number.isFinite(v));
  if (finite.length === 0) return null;
  const sorted = [...finite].sort((a, b) => a - b);
  return {
    median: quantile(sorted, 0.5),
    q1: quantile(sorted, 0.25),
    q3: quantile(sorted, 0.75),
    count: finite.length,
  };
}

// ─── Plot A: Key Metrics ─────────────────────────────────────────────────────

function MetricBarChart({
  metric,
  schedules,
  buckets,
  filters,
  horizontal,
  groupDimension,
}: {
  metric: MetricSpec;
  schedules: ScheduleAnalysisData[];
  buckets: AggBucket[];
  filters: PlotFilters;
  horizontal: boolean;
  groupDimension: Dimension | null;
}) {
  const { layout } = usePlotlyTheme({
    yAxis: { title: horizontal ? '' : metric.axisTitle },
    xAxis: { title: horizontal ? metric.axisTitle : '' },
    showLegend: false,
  });
  const { config, onInitialized, headerActions, fullscreenOverlay } = usePlotlyChartChrome({
    label: metric.label,
    help: KEY_METRICS_HELP,
  });

  const valueOf = (s: ScheduleAnalysisData): number | null => {
    if (metric.key === 'cumulative_priority_sum' && s.insights && filters.priority) {
      // Honour the priority range when computing the cumulative-priority metric.
      const lo = filters.priority.min;
      const hi = filters.priority.max;
      return s.insights.blocks
        .filter((b) => b.scheduled && b.priority >= lo && b.priority <= hi)
        .reduce((sum, b) => sum + b.priority, 0);
    }
    return metric.getValue(s);
  };

  const data = useMemo(() => {
    if (groupDimension) {
      const labels: string[] = [];
      const medians: number[] = [];
      const q1s: number[] = [];
      const q3s: number[] = [];
      const errPlus: number[] = [];
      const errMinus: number[] = [];
      const counts: number[] = [];
      for (const b of buckets) {
        const summary = summarise(b.members.map(valueOf));
        if (!summary) continue;
        labels.push(b.key);
        medians.push(summary.median);
        q1s.push(summary.q1);
        q3s.push(summary.q3);
        errPlus.push(summary.q3 - summary.median);
        errMinus.push(summary.median - summary.q1);
        counts.push(summary.count);
      }
      return [
        {
          type: 'bar' as const,
          orientation: horizontal ? ('h' as const) : ('v' as const),
          x: horizontal ? medians : labels,
          y: horizontal ? labels : medians,
          marker: {
            color: labels.map((_, i) => colorFor(GROUP_PALETTE, i)),
          },
          error_x: horizontal
            ? { type: 'data' as const, symmetric: false, array: errPlus, arrayminus: errMinus }
            : undefined,
          error_y: horizontal
            ? undefined
            : { type: 'data' as const, symmetric: false, array: errPlus, arrayminus: errMinus },
          customdata: counts,
          hovertemplate:
            `%{${horizontal ? 'y' : 'x'}}<br>` +
            `${metric.label} (median): %{${horizontal ? 'x' : 'y'}:.2f}<br>` +
            `n = %{customdata}<extra></extra>`,
        },
      ];
    }
    const labels = schedules.map((s) => s.name);
    const values = schedules.map((s) => valueOf(s) ?? 0);
    return [
      {
        type: 'bar' as const,
        orientation: horizontal ? ('h' as const) : ('v' as const),
        x: horizontal ? values : labels,
        y: horizontal ? labels : values,
        marker: {
          color: schedules.map((_, i) => colorFor(SCHEDULE_PALETTE, i)),
        },
        hovertemplate:
          `%{${horizontal ? 'y' : 'x'}}<br>${metric.label}: %{${horizontal ? 'x' : 'y'}:.2f}<extra></extra>`,
      },
    ];
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [schedules, buckets, metric, filters, horizontal, groupDimension]);

  const mergedLayout = useMemo(
    () => ({
      ...layout,
      xaxis: {
        ...layout.xaxis,
        tickangle: horizontal ? 0 : -30,
        automargin: true,
      },
      yaxis: { ...layout.yaxis, automargin: true },
      margin: { t: 30, b: horizontal ? 50 : 80, l: horizontal ? 120 : 55, r: 10 },
    }),
    [layout, horizontal],
  );

  // Keep horizontal bars readable: ~22px per bar, capped.
  const rowCount = groupDimension ? buckets.length : schedules.length;
  const dynamicHeight = horizontal
    ? Math.min(680, Math.max(220, rowCount * 24 + 60))
    : 240;

  return (
    <ChartPanel title={metric.label} headerActions={headerActions}>
      <PlotlyChart
        data={data}
        layout={mergedLayout}
        config={config}
        onInitialized={onInitialized}
        height={`${dynamicHeight}px`}
        ariaLabel={`${metric.label} comparison bar chart`}
      />
      {fullscreenOverlay}
    </ChartPanel>
  );
}

// ─── Plot B: Priority Distribution ──────────────────────────────────────────

function PriorityBoxPlot({
  schedules,
  filters,
}: {
  schedules: ScheduleAnalysisData[];
  filters: PlotFilters;
}) {
  const { layout } = usePlotlyTheme({
    yAxis: { title: 'Priority' },
    showLegend: false,
  });
  const { config, onInitialized, headerActions, fullscreenOverlay } = usePlotlyChartChrome({
    label: 'Scheduled task priority distribution',
    help: PRIORITY_BOX_HELP,
  });

  const data = useMemo(() => {
    const lo = filters.priority?.min ?? -Infinity;
    const hi = filters.priority?.max ?? Infinity;
    return schedules.map((s, i) => ({
      type: 'box' as const,
      name: s.name,
      y:
        s.insights?.blocks
          .filter((b) => b.scheduled && b.priority >= lo && b.priority <= hi)
          .map((b) => b.priority) ?? [],
      marker: { color: colorFor(SCHEDULE_PALETTE, i) },
      boxpoints: 'outliers' as const,
      hovertemplate: `%{name}<br>Priority: %{y}<extra></extra>`,
    }));
  }, [schedules, filters]);

  const mergedLayout = useMemo(
    () => ({
      ...layout,
      xaxis: { ...layout.xaxis, tickangle: -30, automargin: true },
      margin: { t: 20, b: 80, l: 55, r: 10 },
    }),
    [layout],
  );

  const dynamicHeight = Math.min(640, Math.max(280, schedules.length * 30 + 120));

  return (
    <ChartPanel title="Scheduled Task Priority Distribution" headerActions={headerActions}>
      <PlotlyChart
        data={data}
        layout={mergedLayout}
        config={config}
        onInitialized={onInitialized}
        height={`${dynamicHeight}px`}
        ariaLabel="Box plot of scheduled task priority distribution per schedule"
      />
      {fullscreenOverlay}
    </ChartPanel>
  );
}

// ─── Plot E: Time-Use Breakdown ──────────────────────────────────────────────

const SEGMENT_LABELS: Record<FragmentationSegmentKind, string> = {
  scheduled: 'Scheduled',
  feasible_but_unused: 'Feasible but unused',
  visible_but_no_task_fits: 'Visible – no task fits',
  no_target_visible: 'No target visible',
  non_operable: 'Non-operable',
};
const SEGMENT_COLORS: Record<FragmentationSegmentKind, string> = {
  scheduled: '#10b981',
  feasible_but_unused: '#fbbf24',
  visible_but_no_task_fits: '#fb923c',
  no_target_visible: '#60a5fa',
  non_operable: '#475569',
};
const SEGMENT_ORDER: FragmentationSegmentKind[] = [
  'scheduled',
  'feasible_but_unused',
  'visible_but_no_task_fits',
  'no_target_visible',
  'non_operable',
];

function TimeUseBreakdown({ schedules }: { schedules: ScheduleAnalysisData[] }) {
  const { layout } = usePlotlyTheme({
    xAxis: { title: 'Fraction of schedule window' },
    showLegend: true,
    barMode: 'stack',
  });
  const { config, onInitialized, headerActions, fullscreenOverlay } = usePlotlyChartChrome({
    label: 'Time-use breakdown',
    help: TIME_USE_HELP,
  });

  const data = useMemo(() => {
    const names = schedules.map((s) => s.name);
    return SEGMENT_ORDER.map((kind) => {
      const values = schedules.map((s) => {
        const frag = s.fragmentation;
        if (!frag || frag.metrics.schedule_hours === 0) return 0;
        const opFrac = frag.metrics.operable_hours / frag.metrics.schedule_hours;
        if (kind === 'non_operable') return Math.max(0, 1 - opFrac);
        const entry = frag.reason_breakdown.find((rb) => rb.kind === kind);
        return entry ? entry.fraction_of_operable * opFrac : 0;
      });
      return {
        type: 'bar' as const,
        name: SEGMENT_LABELS[kind],
        x: values,
        y: names,
        orientation: 'h' as const,
        marker: { color: SEGMENT_COLORS[kind] },
        hovertemplate: `%{y}<br>${SEGMENT_LABELS[kind]}: %{x:.1%}<extra></extra>`,
      };
    });
  }, [schedules]);

  const perBar = schedules.length > 12 ? 28 : 44;
  const barHeight = Math.min(720, Math.max(180, schedules.length * perBar + 80));

  const mergedLayout = useMemo(
    () => ({
      ...layout,
      xaxis: {
        ...layout.xaxis,
        tickformat: '.0%',
        range: [0, 1],
        automargin: true,
      },
      yaxis: { ...layout.yaxis, automargin: true },
      margin: { t: 20, b: 50, l: 10, r: 10 },
    }),
    [layout],
  );

  return (
    <ChartPanel title="Time-Use Breakdown" headerActions={headerActions}>
      <PlotlyChart
        data={data}
        layout={mergedLayout}
        config={config}
        onInitialized={onInitialized}
        height={`${barHeight}px`}
        ariaLabel="Stacked bar chart showing schedule time-use breakdown per schedule"
      />
      {fullscreenOverlay}
    </ChartPanel>
  );
}

// ─── Toolbar UI ──────────────────────────────────────────────────────────────

function MetricSelector({
  selected,
  onToggle,
}: {
  selected: Set<string>;
  onToggle: (key: string) => void;
}) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-xs font-semibold uppercase tracking-wider text-slate-400">
        Metrics
      </span>
      <div className="flex flex-wrap gap-1.5">
        {ALL_METRICS.map((m) => {
          const active = selected.has(m.key);
          return (
            <button
              key={m.key}
              type="button"
              onClick={() => onToggle(m.key)}
              title={m.description ?? m.label}
              className={`rounded-full border px-2.5 py-0.5 text-xs transition-colors ${
                active
                  ? 'border-sky-400 bg-sky-500/20 text-sky-100'
                  : 'border-slate-700 bg-slate-800 text-slate-400 hover:border-slate-500 hover:text-slate-200'
              }`}
              aria-pressed={active}
            >
              {m.label}
            </button>
          );
        })}
      </div>
    </div>
  );
}

// ─── Composed export ────────────────────────────────────────────────────────

export function ComparisonCharts({ schedules }: { schedules: ScheduleAnalysisData[] }) {
  // Configuration filters (e/k/b/…) over algorithm_config.
  const configFilters = useConfigFilters({
    items: schedules,
    getConfig: (s) => s.algorithmConfig,
  });

  // Plot filters (priority window, scheduled-hours window).
  const plot = usePlotFilters(schedules);

  // Toolbar state.
  const [selectedMetricKeys, setSelectedMetricKeys] = useState<Set<string>>(
    new Set(DEFAULT_COMPARISON_METRIC_KEYS),
  );
  const [groupKey, setGroupKey] = useState<string>(''); // '' = no grouping
  const [collapseEquivalents, setCollapseEquivalents] = useState(false);
  const [horizontalBars, setHorizontalBars] = useState(schedules.length > 12);
  const [sortMetricKey, setSortMetricKey] = useState<string>('');
  const [sortDir, setSortDir] = useState<'asc' | 'desc'>('desc');

  const groupDimension = useMemo(() => {
    if (!groupKey) return null;
    return (
      configFilters.numericSpecs.find((s) => s.key === groupKey)
        ? ({
            key: groupKey,
            kind: 'numeric' as const,
            values:
              configFilters.numericSpecs.find((s) => s.key === groupKey)?.values ?? [],
          } as Dimension)
        : configFilters.categorical.find((d) => d.key === groupKey) ?? null
    );
  }, [groupKey, configFilters.numericSpecs, configFilters.categorical]);

  // Apply config filters first, then the schedule-hours floor.
  const configFiltered = configFilters.filtered;

  const visibleSchedules = useMemo(() => {
    const min = plot.filters.scheduledHours?.min ?? -Infinity;
    const max = plot.filters.scheduledHours?.max ?? Infinity;
    const filtered = configFiltered.filter((s) => {
      const h = s.fragmentation?.metrics.scheduled_hours;
      if (h == null) return true;
      return h >= min && h <= max;
    });

    if (!collapseEquivalents) return filtered;
    const eq = groupEquivalentSchedules(filtered, (s) => s.insights);
    return collapseToRepresentatives(filtered, eq);
  }, [configFiltered, plot.filters.scheduledHours, collapseEquivalents]);

  const equivalence = useMemo(
    () => groupEquivalentSchedules(visibleSchedules, (s) => s.insights),
    [visibleSchedules],
  );

  const sortedSchedules = useMemo(() => {
    if (!sortMetricKey) return visibleSchedules;
    const m = getMetric(sortMetricKey);
    if (!m) return visibleSchedules;
    const dir = sortDir === 'asc' ? 1 : -1;
    return [...visibleSchedules].sort((a, b) => {
      const va = m.getValue(a);
      const vb = m.getValue(b);
      if (va == null && vb == null) return 0;
      if (va == null) return 1;
      if (vb == null) return -1;
      return (va - vb) * dir;
    });
  }, [visibleSchedules, sortMetricKey, sortDir]);

  const buckets = useMemo(
    () => bucketBy(sortedSchedules, groupDimension),
    [sortedSchedules, groupDimension],
  );

  const selectedMetrics = useMemo(
    () => ALL_METRICS.filter((m) => selectedMetricKeys.has(m.key)),
    [selectedMetricKeys],
  );

  const toggleMetric = (key: string) => {
    setSelectedMetricKeys((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

  if (schedules.length === 0) return null;

  const groupOptions: Array<{ key: string; label: string }> = [
    { key: '', label: 'None (per schedule)' },
    ...configFilters.numericSpecs.map((s) => ({ key: s.key, label: `${s.key} (numeric)` })),
    ...configFilters.categorical.map((d) => ({
      key: d.key,
      label: `${d.key} (categorical)`,
    })),
  ];

  const equivalentGroups = equivalence.groups.filter((g) => g.members.length > 1);

  return (
    <div className="flex flex-col gap-6">
      {/* Configuration filters */}
      {(configFilters.numericSpecs.length > 0 || configFilters.categorical.length > 0) && (
        <div className="flex flex-col gap-3 rounded-2xl border border-slate-800 bg-slate-900/40 p-4">
          <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-300">
                Configuration filters
              </h2>
              <HelpPopover
                content={{
                  title: 'Configuration filters',
                  summary:
                    'Restrict which schedules feed every chart and table below by ranging over the algorithm configuration knobs (e, k, b, …).',
                  bullets: [
                    'Numeric knobs become dual-thumb range sliders.',
                    'Categorical knobs become chip selectors.',
                    'Filters are derived from each schedule\u2019s `algorithm_config` blob.',
                  ],
                }}
                ariaLabel="Help: configuration filters"
              />
              <span className="text-xs text-slate-500">
                · {configFiltered.length}/{schedules.length} schedules pass
              </span>
            </div>
            {configFilters.hasAnyFilter ? (
              <button
                type="button"
                onClick={configFilters.reset}
                className="text-xs text-slate-400 hover:text-slate-200"
              >
                ✕ Reset config filters
              </button>
            ) : null}
          </div>
          {configFilters.numericSpecs.length > 0 ? (
            <RangeFilterGroup
              specs={configFilters.numericSpecs}
              values={configFilters.numericValues}
              onChange={configFilters.setNumericValues}
            />
          ) : null}
          {configFilters.categorical.length > 0 ? (
            <CategoricalFilterGroup
              dimensions={configFilters.categorical}
              values={configFilters.categoricalValues}
              onChange={configFilters.setCategoricalValue}
            />
          ) : null}
        </div>
      )}

      {/* Plot filters */}
      {plot.specs.length > 0 && (
        <div className="flex flex-col gap-2">
          <div className="flex items-center gap-2">
            <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-300">
              Plot filters
            </h2>
            <HelpPopover content={COMPARISON_FILTER_HELP} ariaLabel="Help: comparison filters" />
          </div>
          <RangeFilterGroup specs={plot.specs} values={plot.values} onChange={plot.setValues} />
        </div>
      )}

      {/* Toolbar */}
      <div className="flex flex-wrap items-end gap-4 rounded-2xl border border-slate-800 bg-slate-900/40 p-4">
        <MetricSelector selected={selectedMetricKeys} onToggle={toggleMetric} />

        <div className="flex flex-col gap-1">
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-400">
            Group by
          </span>
          <select
            value={groupKey}
            onChange={(e) => setGroupKey(e.target.value)}
            className="rounded border border-slate-700 bg-slate-800 px-2 py-1 text-sm text-slate-100"
          >
            {groupOptions.map((o) => (
              <option key={o.key || '_none'} value={o.key}>
                {o.label}
              </option>
            ))}
          </select>
        </div>

        <div className="flex flex-col gap-1">
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-400">
            Sort by
          </span>
          <div className="flex items-center gap-1">
            <select
              value={sortMetricKey}
              onChange={(e) => setSortMetricKey(e.target.value)}
              className="rounded border border-slate-700 bg-slate-800 px-2 py-1 text-sm text-slate-100"
            >
              <option value="">— input order —</option>
              {ALL_METRICS.map((m) => (
                <option key={m.key} value={m.key}>
                  {m.label}
                </option>
              ))}
            </select>
            <button
              type="button"
              onClick={() => setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'))}
              className="rounded border border-slate-700 bg-slate-800 px-2 py-1 text-xs text-slate-300 hover:text-white"
              title={sortDir === 'asc' ? 'Ascending' : 'Descending'}
            >
              {sortDir === 'asc' ? '↑' : '↓'}
            </button>
          </div>
        </div>

        <label className="flex items-center gap-2 text-sm text-slate-300">
          <input
            type="checkbox"
            checked={horizontalBars}
            onChange={(e) => setHorizontalBars(e.target.checked)}
            className="accent-sky-500"
          />
          Horizontal bars
        </label>

        <label className="flex items-center gap-2 text-sm text-slate-300">
          <input
            type="checkbox"
            checked={collapseEquivalents}
            onChange={(e) => setCollapseEquivalents(e.target.checked)}
            className="accent-sky-500"
          />
          Collapse equivalent schedules
        </label>
      </div>

      {/* Equivalence summary */}
      {equivalentGroups.length > 0 ? (
        <div className="rounded-lg border border-emerald-700/40 bg-emerald-900/20 px-4 py-2 text-xs text-emerald-200">
          {equivalentGroups.length} equivalence group
          {equivalentGroups.length === 1 ? '' : 's'} detected (same scheduled task set):{' '}
          {equivalentGroups
            .slice(0, 5)
            .map((g) => g.members.map((m) => m.name).join(' = '))
            .join(' · ')}
          {equivalentGroups.length > 5 ? ' · …' : ''}
        </div>
      ) : null}

      {/* Plot A: Key Metrics */}
      {selectedMetrics.length > 0 ? (
        <div
          className={`grid gap-4 ${
            selectedMetrics.length === 1
              ? 'grid-cols-1'
              : selectedMetrics.length === 2
                ? 'md:grid-cols-2'
                : selectedMetrics.length === 3
                  ? 'md:grid-cols-2 xl:grid-cols-3'
                  : 'md:grid-cols-2 xl:grid-cols-4'
          }`}
        >
          {selectedMetrics.map((metric) => (
            <MetricBarChart
              key={metric.key}
              metric={metric}
              schedules={sortedSchedules}
              buckets={buckets}
              filters={plot.filters}
              horizontal={horizontalBars}
              groupDimension={groupDimension}
            />
          ))}
        </div>
      ) : (
        <div className="rounded-lg border border-dashed border-slate-700 px-4 py-6 text-center text-sm text-slate-400">
          Pick at least one metric above to display.
        </div>
      )}

      {/* Plot B: Priority Distribution */}
      <PriorityBoxPlot schedules={sortedSchedules} filters={plot.filters} />

      {/* Plot E: Time-Use Breakdown */}
      <TimeUseBreakdown schedules={sortedSchedules} />
    </div>
  );
}
