/**
 * Compare page - Audit-oriented comparison of two schedules.
 *
 * Block matching is keyed on non-empty `original_block_id`, never on row
 * position or database block id. Gap metrics come from the fragmentation
 * endpoint so they reflect operable-only gaps.
 */
import { useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { useCompare, useFragmentation, usePlotlyTheme, useSchedules } from '@/hooks';
import {
  ChartPanel,
  ErrorMessage,
  Icon,
  LoadingSpinner,
  MetricCard,
  PageContainer,
  PageHeader,
  PlotlyChart,
} from '@/components';
import { mjdToDate, isValidDate } from '@/constants/dates';
import type {
  CompareDiffBlock,
  FragmentationMetrics,
  RetimedBlockChange,
} from '@/api/types';

const SCHEDULE_COLORS = {
  current: '#3b82f6',
  comparison: '#f59e0b',
} as const;

function formatMjdUtc(mjd: number | null | undefined): string {
  if (mjd == null || !Number.isFinite(mjd)) return '—';
  const d = mjdToDate(mjd);
  if (!isValidDate(d)) return '—';
  return d.toISOString().replace('T', ' ').replace(/\.\d+Z$/, 'Z');
}

function formatHours(h: number | null | undefined, digits = 2): string {
  if (h == null || !Number.isFinite(h)) return '—';
  return `${h.toFixed(digits)} h`;
}

function SummaryPanel({
  title,
  color,
  scheduledCount,
  unscheduledCount,
  cumulativePriority,
  scheduledHours,
  metrics,
}: {
  title: string;
  color: string;
  scheduledCount: number;
  unscheduledCount: number;
  cumulativePriority: number;
  scheduledHours: number;
  metrics: FragmentationMetrics | undefined;
}) {
  return (
    <ChartPanel title={title}>
      <div className="mb-3 h-1 w-12 rounded" style={{ background: color }} />
      <div className="grid grid-cols-2 gap-3 md:grid-cols-3">
        <MetricCard label="Scheduled" value={scheduledCount.toLocaleString()} icon={<Icon name="check-circle" />} />
        <MetricCard label="Unscheduled" value={unscheduledCount.toLocaleString()} icon={<Icon name="x-circle" />} />
        <MetricCard label="Cumulative priority" value={cumulativePriority.toFixed(2)} icon={<Icon name="star" />} />
        <MetricCard label="Scheduled hours" value={formatHours(scheduledHours, 1)} icon={<Icon name="clock" />} />
        <MetricCard label="Gap count" value={metrics ? metrics.gap_count.toLocaleString() : '—'} />
        <MetricCard label="Gap mean" value={metrics ? formatHours(metrics.gap_mean_hours) : '—'} />
        <MetricCard label="Gap stddev" value={metrics ? formatHours(metrics.gap_std_dev_hours) : '—'} />
        <MetricCard label="Gap p90" value={metrics ? formatHours(metrics.gap_p90_hours) : '—'} />
        <MetricCard label="Largest gap" value={metrics ? formatHours(metrics.largest_gap_hours) : '—'} />
      </div>
    </ChartPanel>
  );
}

function DiffTable({
  title,
  rows,
  currentName,
  comparisonName,
}: {
  title: string;
  rows: CompareDiffBlock[];
  currentName: string;
  comparisonName: string;
}) {
  if (rows.length === 0) return null;
  return (
    <ChartPanel title={`${title} (${rows.length})`}>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-slate-700 text-slate-400">
              <th className="px-3 py-2 text-left">Original ID</th>
              <th className="px-3 py-2 text-left">Name</th>
              <th className="px-3 py-2 text-right">Priority</th>
              <th className="px-3 py-2 text-right">Requested</th>
              <th className="px-3 py-2 text-left">{currentName} start (UTC)</th>
              <th className="px-3 py-2 text-left">{currentName} stop (UTC)</th>
              <th className="px-3 py-2 text-left">{comparisonName} start (UTC)</th>
              <th className="px-3 py-2 text-left">{comparisonName} stop (UTC)</th>
            </tr>
          </thead>
          <tbody>
            {rows.slice(0, 100).map((r) => (
              <tr key={r.original_block_id} className="border-b border-slate-700/50">
                <td className="px-3 py-2 font-mono text-white">{r.original_block_id}</td>
                <td className="px-3 py-2 text-slate-300">{r.block_name || '—'}</td>
                <td className="px-3 py-2 text-right text-slate-300">{r.priority.toFixed(2)}</td>
                <td className="px-3 py-2 text-right text-slate-300">{formatHours(r.requested_hours)}</td>
                <td className="px-3 py-2 text-slate-400">{formatMjdUtc(r.current_scheduled_start_mjd)}</td>
                <td className="px-3 py-2 text-slate-400">{formatMjdUtc(r.current_scheduled_stop_mjd)}</td>
                <td className="px-3 py-2 text-slate-400">{formatMjdUtc(r.comparison_scheduled_start_mjd)}</td>
                <td className="px-3 py-2 text-slate-400">{formatMjdUtc(r.comparison_scheduled_stop_mjd)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {rows.length > 100 && (
          <p className="mt-3 text-center text-slate-400">… and {rows.length - 100} more rows</p>
        )}
      </div>
    </ChartPanel>
  );
}

function RetimedTable({
  rows,
  currentName,
  comparisonName,
}: {
  rows: RetimedBlockChange[];
  currentName: string;
  comparisonName: string;
}) {
  if (rows.length === 0) return null;
  return (
    <ChartPanel title={`Retimed common blocks (${rows.length})`}>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-slate-700 text-slate-400">
              <th className="px-3 py-2 text-left">Original ID</th>
              <th className="px-3 py-2 text-left">Name</th>
              <th className="px-3 py-2 text-right">Priority</th>
              <th className="px-3 py-2 text-right">Start shift</th>
              <th className="px-3 py-2 text-right">Stop shift</th>
              <th className="px-3 py-2 text-left">{currentName} start (UTC)</th>
              <th className="px-3 py-2 text-left">{comparisonName} start (UTC)</th>
            </tr>
          </thead>
          <tbody>
            {rows.slice(0, 100).map((r) => (
              <tr key={r.original_block_id} className="border-b border-slate-700/50">
                <td className="px-3 py-2 font-mono text-white">{r.original_block_id}</td>
                <td className="px-3 py-2 text-slate-300">{r.block_name || '—'}</td>
                <td className="px-3 py-2 text-right text-slate-300">{r.priority.toFixed(2)}</td>
                <td className="px-3 py-2 text-right text-slate-300">{formatHours(r.start_shift_hours)}</td>
                <td className="px-3 py-2 text-right text-slate-300">{formatHours(r.stop_shift_hours)}</td>
                <td className="px-3 py-2 text-slate-400">{formatMjdUtc(r.current_scheduled_start_mjd)}</td>
                <td className="px-3 py-2 text-slate-400">{formatMjdUtc(r.comparison_scheduled_start_mjd)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {rows.length > 100 && (
          <p className="mt-3 text-center text-slate-400">… and {rows.length - 100} more rows</p>
        )}
      </div>
    </ChartPanel>
  );
}

function Compare() {
  const { scheduleId, otherId } = useParams();
  const currentId = parseInt(scheduleId ?? '0', 10);
  const comparisonId = parseInt(otherId ?? '0', 10);
  const { data: schedulesData } = useSchedules();

  const compareQuery = useMemo(() => {
    const currentName = schedulesData?.schedules.find((s) => s.schedule_id === currentId)?.schedule_name;
    const comparisonName = schedulesData?.schedules.find((s) => s.schedule_id === comparisonId)?.schedule_name;
    if (!currentName && !comparisonName) return undefined;
    return { current_name: currentName, comparison_name: comparisonName };
  }, [comparisonId, currentId, schedulesData?.schedules]);

  const { data, isLoading, error, refetch } = useCompare(currentId, comparisonId, compareQuery);
  const currentFrag = useFragmentation(currentId);
  const comparisonFrag = useFragmentation(comparisonId);

  const { layout: priorityLayout, config } = usePlotlyTheme({
    title: 'Priority Distribution',
    xAxis: { title: 'Priority' },
    yAxis: { title: 'Count' },
    barMode: 'overlay',
  });

  const { layout: hoursLayout } = usePlotlyTheme({
    title: 'Requested Hours',
    yAxis: { title: 'Requested Hours' },
  });

  const priorityDistributionData = useMemo((): Plotly.Data[] => {
    if (!data) return [];
    return [
      {
        x: data.current_blocks.map((b) => b.priority),
        type: 'histogram' as const,
        name: data.current_name,
        opacity: 0.7,
        marker: { color: SCHEDULE_COLORS.current },
      },
      {
        x: data.comparison_blocks.map((b) => b.priority),
        type: 'histogram' as const,
        name: data.comparison_name,
        opacity: 0.7,
        marker: { color: SCHEDULE_COLORS.comparison },
      },
    ];
  }, [data]);

  const hoursData = useMemo((): Plotly.Data[] => {
    if (!data) return [];
    return [
      {
        y: data.current_blocks.map((b) => b.requested_hours),
        type: 'box',
        name: data.current_name,
        marker: { color: SCHEDULE_COLORS.current },
        boxmean: 'sd',
      },
      {
        y: data.comparison_blocks.map((b) => b.requested_hours),
        type: 'box',
        name: data.comparison_name,
        marker: { color: SCHEDULE_COLORS.comparison },
        boxmean: 'sd',
      },
    ];
  }, [data]);

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    );
  }
  if (error) {
    return (
      <ErrorMessage
        title="Failed to load comparison"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }
  if (!data) return <ErrorMessage message="No data available" />;

  const deltaScheduled =
    data.comparison_stats.scheduled_count - data.current_stats.scheduled_count;
  const deltaPriority =
    data.comparison_stats.total_priority - data.current_stats.total_priority;
  const deltaHours =
    data.comparison_stats.total_hours - data.current_stats.total_hours;

  return (
    <PageContainer>
      <PageHeader
        title="Compare Schedules"
        description={`${data.current_name} vs ${data.comparison_name}`}
      />

      {/* Side-by-side schedule summary panels */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <SummaryPanel
          title={data.current_name}
          color={SCHEDULE_COLORS.current}
          scheduledCount={data.current_stats.scheduled_count}
          unscheduledCount={data.current_stats.unscheduled_count}
          cumulativePriority={data.current_stats.total_priority}
          scheduledHours={data.current_stats.total_hours}
          metrics={currentFrag.data?.metrics}
        />
        <SummaryPanel
          title={data.comparison_name}
          color={SCHEDULE_COLORS.comparison}
          scheduledCount={data.comparison_stats.scheduled_count}
          unscheduledCount={data.comparison_stats.unscheduled_count}
          cumulativePriority={data.comparison_stats.total_priority}
          scheduledHours={data.comparison_stats.total_hours}
          metrics={comparisonFrag.data?.metrics}
        />
      </div>

      {/* Compact delta strip */}
      <ChartPanel title="Delta (comparison − current)">
        <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
          <MetricCard label="Δ scheduled" value={deltaScheduled >= 0 ? `+${deltaScheduled}` : `${deltaScheduled}`} />
          <MetricCard
            label="Δ cumulative priority"
            value={deltaPriority >= 0 ? `+${deltaPriority.toFixed(2)}` : deltaPriority.toFixed(2)}
          />
          <MetricCard label="Δ scheduled hours" value={formatHours(deltaHours)} />
          <MetricCard label="Retimed blocks" value={data.retimed_blocks.length} />
        </div>
      </ChartPanel>

      {/* Detailed tables */}
      <DiffTable
        title={`Scheduled only in ${data.current_name}`}
        rows={data.scheduled_only_current}
        currentName={data.current_name}
        comparisonName={data.comparison_name}
      />
      <DiffTable
        title={`Scheduled only in ${data.comparison_name}`}
        rows={data.scheduled_only_comparison}
        currentName={data.current_name}
        comparisonName={data.comparison_name}
      />
      <DiffTable
        title={`Only present in ${data.current_name}`}
        rows={data.only_in_current_blocks}
        currentName={data.current_name}
        comparisonName={data.comparison_name}
      />
      <DiffTable
        title={`Only present in ${data.comparison_name}`}
        rows={data.only_in_comparison_blocks}
        currentName={data.current_name}
        comparisonName={data.comparison_name}
      />
      <RetimedTable
        rows={data.retimed_blocks}
        currentName={data.current_name}
        comparisonName={data.comparison_name}
      />

      {/* Secondary context distributions */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <ChartPanel title="Priority distribution">
          <PlotlyChart data={priorityDistributionData} layout={priorityLayout} config={config} height="350px" />
        </ChartPanel>
        <ChartPanel title="Requested hours distribution">
          <PlotlyChart data={hoursData} layout={hoursLayout} config={config} height="350px" />
        </ChartPanel>
      </div>
    </PageContainer>
  );
}

export default Compare;
