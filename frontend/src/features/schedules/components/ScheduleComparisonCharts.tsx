/**
 * Visual comparison charts for the Compare page.
 *
 * Three panels:
 *   A – Key Metrics: grouped bar charts for scheduling rate, cumulative priority,
 *       scheduled hours, and gap count.
 *   B – Priority Distribution: box plots showing the spread of scheduled-task
 *       priorities per schedule.
 *   E – Time-Use Breakdown: horizontal 100 % stacked bar decomposing the
 *       schedule window into segment kinds (scheduled, feasible-but-unused,
 *       visible-no-fit, no-target-visible, non-operable).
 */
import { useMemo } from 'react';
import { PlotlyChart } from '@/components';
import { usePlotlyTheme } from '@/hooks';
import type { FragmentationSegmentKind } from '@/api/types';
import type { ScheduleAnalysisData } from '../hooks/useScheduleAnalysisData';

// Each schedule gets a distinct colour; index 0 = reference (sky-blue).
const SCHEDULE_PALETTE = [
  '#38bdf8', // sky-400   – reference
  '#a78bfa', // violet-400
  '#34d399', // emerald-400
  '#fbbf24', // amber-400
  '#f472b6', // pink-400
  '#60a5fa', // blue-400
  '#fb923c', // orange-400
  '#a3e635', // lime-400
];

function scheduleColor(index: number): string {
  return SCHEDULE_PALETTE[index % SCHEDULE_PALETTE.length];
}

function ChartPanel({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="rounded-lg border border-slate-700 bg-slate-900 p-4">
      <h2 className="mb-4 text-lg font-semibold text-white">{title}</h2>
      {children}
    </section>
  );
}

// ─── Plot A: Key Metrics ─────────────────────────────────────────────────────

type MetricSpec = {
  label: string;
  unit: string;
  yAxisTitle: string;
  getValue: (s: ScheduleAnalysisData) => number | null;
};

const KEY_METRICS: MetricSpec[] = [
  {
    label: 'Scheduling Rate',
    unit: '%',
    yAxisTitle: 'Rate (%)',
    getValue: (s) => (s.insights != null ? s.insights.metrics.scheduling_rate * 100 : null),
  },
  {
    label: 'Cumulative Priority',
    unit: '',
    yAxisTitle: 'Priority score',
    getValue: (s) =>
      s.insights != null
        ? s.insights.blocks.filter((b) => b.scheduled).reduce((sum, b) => sum + b.priority, 0)
        : null,
  },
  {
    label: 'Scheduled Hours',
    unit: 'h',
    yAxisTitle: 'Hours',
    getValue: (s) => s.fragmentation?.metrics.scheduled_hours ?? null,
  },
  {
    label: 'Gap Count',
    unit: '',
    yAxisTitle: 'Gaps',
    getValue: (s) => s.fragmentation?.metrics.gap_count ?? null,
  },
];

function MetricBarChart({
  metric,
  schedules,
}: {
  metric: MetricSpec;
  schedules: ScheduleAnalysisData[];
}) {
  const { layout, config } = usePlotlyTheme({
    yAxis: { title: metric.yAxisTitle },
    showLegend: false,
  });

  const data = useMemo(
    () => [
      {
        type: 'bar' as const,
        x: schedules.map((s) => s.name),
        y: schedules.map((s) => metric.getValue(s) ?? 0),
        marker: { color: schedules.map((_, i) => scheduleColor(i)) },
        hovertemplate: `%{x}<br>${metric.label}: %{y:.2f}${metric.unit}<extra></extra>`,
      },
    ],
    [schedules, metric]
  );

  const mergedLayout = useMemo(
    () => ({
      ...layout,
      xaxis: { ...layout.xaxis, tickangle: -30, automargin: true },
      margin: { t: 30, b: 80, l: 55, r: 10 },
    }),
    [layout]
  );

  return (
    <div>
      <p className="mb-1 text-center text-sm font-medium text-slate-300">{metric.label}</p>
      <PlotlyChart
        data={data}
        layout={mergedLayout}
        config={config}
        height="220px"
        ariaLabel={`${metric.label} comparison bar chart`}
      />
    </div>
  );
}

function KeyMetricsCharts({ schedules }: { schedules: ScheduleAnalysisData[] }) {
  return (
    <ChartPanel title="Key Metrics">
      <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
        {KEY_METRICS.map((metric) => (
          <MetricBarChart key={metric.label} metric={metric} schedules={schedules} />
        ))}
      </div>
    </ChartPanel>
  );
}

// ─── Plot B: Priority Distribution ──────────────────────────────────────────

function PriorityBoxPlot({ schedules }: { schedules: ScheduleAnalysisData[] }) {
  const { layout, config } = usePlotlyTheme({
    yAxis: { title: 'Priority' },
    showLegend: false,
  });

  const data = useMemo(
    () =>
      schedules.map((s, i) => ({
        type: 'box' as const,
        name: s.name,
        y: s.insights?.blocks.filter((b) => b.scheduled).map((b) => b.priority) ?? [],
        marker: { color: scheduleColor(i) },
        boxpoints: 'outliers' as const,
        hovertemplate: `%{name}<br>Priority: %{y}<extra></extra>`,
      })),
    [schedules]
  );

  const mergedLayout = useMemo(
    () => ({
      ...layout,
      xaxis: { ...layout.xaxis, tickangle: -30, automargin: true },
      margin: { t: 20, b: 80, l: 55, r: 10 },
    }),
    [layout]
  );

  return (
    <ChartPanel title="Scheduled Task Priority Distribution">
      <PlotlyChart
        data={data}
        layout={mergedLayout}
        config={config}
        height="320px"
        ariaLabel="Box plot of scheduled task priority distribution per schedule"
      />
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

// Display order: best outcome first, non-operable last.
const SEGMENT_ORDER: FragmentationSegmentKind[] = [
  'scheduled',
  'feasible_but_unused',
  'visible_but_no_task_fits',
  'no_target_visible',
  'non_operable',
];

function TimeUseBreakdown({ schedules }: { schedules: ScheduleAnalysisData[] }) {
  const { layout, config } = usePlotlyTheme({
    xAxis: { title: 'Fraction of schedule window' },
    showLegend: true,
    barMode: 'stack',
  });

  const data = useMemo(() => {
    const names = schedules.map((s) => s.name);

    return SEGMENT_ORDER.map((kind) => {
      const values = schedules.map((s) => {
        const frag = s.fragmentation;
        if (!frag || frag.metrics.schedule_hours === 0) return 0;

        const opFrac = frag.metrics.operable_hours / frag.metrics.schedule_hours;

        if (kind === 'non_operable') {
          return Math.max(0, 1 - opFrac);
        }

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

  const barHeight = Math.max(180, schedules.length * 50 + 80);

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
    [layout]
  );

  return (
    <ChartPanel title="Time-Use Breakdown">
      <PlotlyChart
        data={data}
        layout={mergedLayout}
        config={config}
        height={`${barHeight}px`}
        ariaLabel="Stacked bar chart showing schedule time-use breakdown per schedule"
      />
    </ChartPanel>
  );
}

// ─── Composed export ──────────────────────────────────────────────────────────

export function ComparisonCharts({ schedules }: { schedules: ScheduleAnalysisData[] }) {
  if (schedules.length === 0) return null;

  return (
    <div className="flex flex-col gap-6">
      <KeyMetricsCharts schedules={schedules} />
      <PriorityBoxPlot schedules={schedules} />
      <TimeUseBreakdown schedules={schedules} />
    </div>
  );
}
