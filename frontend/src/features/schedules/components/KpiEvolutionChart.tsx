/**
 * KPI evolution chart (A3).
 *
 * Plots each schedule's KPI components over upload order (schedule_id,
 * which is monotonic per-environment in both the local and postgres
 * repositories). Lets the user eyeball whether configurations are
 * actually trending in the right direction or just flopping around.
 *
 * Pure presentation: receives the already-fetched KPI rows.
 */
import { useMemo } from 'react';
import { ChartPanel, PlotlyChart } from '@/components';
import { usePlotlyChartChrome, usePlotlyTheme } from '@/hooks';
import type { HelpContent } from '@/components/charts';
import type { ScheduleKpi } from '@/api/types';

const KPI_HELP: HelpContent = {
  title: 'KPI evolution',
  summary:
    'Each line is one normalised KPI component (0–1, higher is better) plotted across schedules in upload order.',
  bullets: [
    'Composite score is the weighted sum of all components; the others are its inputs.',
    'A diverging line means a configuration sacrificed one dimension for another.',
    'Hover any marker to read the exact value and schedule name.',
  ],
};

interface KpiEvolutionChartProps {
  kpis: ScheduleKpi[];
}

const MAX_POINTS_PER_TRACE = 500;

/**
 * Downsample a series of points using a simple stride filter, always keeping
 * the first and last points so the temporal extent of the trace is preserved.
 */
export function downsampleTrace<T>(points: T[], maxPoints: number = MAX_POINTS_PER_TRACE): T[] {
  if (points.length <= maxPoints) return points;
  const stride = Math.ceil(points.length / maxPoints);
  const sampled = points.filter((_, i) => i % stride === 0);
  const last = points[points.length - 1];
  if (sampled[sampled.length - 1] !== last) sampled.push(last);
  return sampled;
}

interface SeriesSpec {
  name: string;
  color: string;
  pick: (k: ScheduleKpi) => number;
}

const SERIES: SeriesSpec[] = [
  { name: 'Composite score', color: '#38bdf8', pick: (k) => k.composite_score },
  { name: 'Scheduling rate', color: '#a78bfa', pick: (k) => k.scheduling_rate },
  {
    name: 'Operable time used',
    color: '#34d399',
    pick: (k) => k.scheduled_fraction_of_operable,
  },
  {
    name: 'Visibility utilisation',
    color: '#fbbf24',
    pick: (k) => k.score_components.fit_visibility_utilisation,
  },
  {
    name: 'Priority alignment',
    color: '#f472b6',
    pick: (k) => k.score_components.priority_alignment,
  },
  {
    name: 'Gap compactness',
    color: '#60a5fa',
    pick: (k) => k.score_components.gap_compactness,
  },
];

export function KpiEvolutionChart({ kpis }: KpiEvolutionChartProps) {
  const { layout } = usePlotlyTheme({
    xAxis: { title: 'Schedule (upload order)', type: 'category' },
    yAxis: { title: 'Normalised score (0–1)', range: [0, 1.05] },
    showLegend: true,
  });
  const { config, onInitialized, headerActions, fullscreenOverlay } = usePlotlyChartChrome({
    label: 'KPI evolution',
    help: KPI_HELP,
  });

  const ordered = useMemo(
    () => [...kpis].sort((a, b) => a.schedule_id - b.schedule_id),
    [kpis],
  );

  const traces = useMemo(
    () =>
      SERIES.map((spec) => {
        const sampled = downsampleTrace(ordered);
        return {
          type: 'scatter' as const,
          mode: 'lines+markers' as const,
          name: spec.name,
          x: sampled.map((k) => k.schedule_name),
          y: sampled.map((k) => spec.pick(k)),
          line: { color: spec.color, width: 2 },
          marker: { color: spec.color, size: 7 },
          hovertemplate: `${spec.name}: %{y:.3f}<br>%{x}<extra></extra>`,
        };
      }),
    [ordered],
  );

  if (ordered.length < 2) return null;

  return (
    <ChartPanel title="KPI evolution" headerActions={headerActions}>
      <PlotlyChart
        data={traces}
        layout={{ ...layout, height: 360 }}
        config={config}
        onInitialized={onInitialized}
        ariaLabel="KPI evolution line chart"
      />
      {fullscreenOverlay}
    </ChartPanel>
  );
}

export default KpiEvolutionChart;
