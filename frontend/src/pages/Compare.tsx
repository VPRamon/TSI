/**
 * Compare page - Compare two schedules.
 * Uses consistent layout primitives and chart theming.
 */
import { useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { useCompare, usePlotlyTheme } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  Icon,
  MetricCard,
  PlotlyChart,
  PageHeader,
  PageContainer,
  MetricsGrid,
  ChartPanel,
} from '@/components';
import { CHANGE_TYPE_COLORS, type ChangeTypeKey } from '@/constants/colors';

/** Consistent pair of colors for current vs comparison schedule */
const SCHEDULE_COLORS = {
  current: '#3b82f6', // blue-500
  currentEdge: '#2563eb', // blue-600
  comparison: '#f59e0b', // amber-500
  comparisonEdge: '#d97706', // amber-600
} as const;

function Compare() {
  const { scheduleId, otherId } = useParams();
  const currentId = parseInt(scheduleId ?? '0', 10);
  const comparisonId = parseInt(otherId ?? '0', 10);

  const { data, isLoading, error, refetch } = useCompare(currentId, comparisonId);

  // ── Chart themes (called unconditionally) ────────────────────────
  const { layout: priorityLayout, config } = usePlotlyTheme({
    title: 'Priority Distribution',
    xAxis: { title: 'Priority' },
    yAxis: { title: 'Count' },
    barMode: 'overlay',
  });

  const { layout: statusLayout } = usePlotlyTheme({
    title: 'Scheduling Status',
    yAxis: { title: 'Number of Blocks' },
    barMode: 'group',
  });

  const { layout: changesLayout } = usePlotlyTheme({
    title: 'Scheduling Changes',
    yAxis: { title: 'Number of Blocks' },
    showLegend: false,
  });

  const { layout: timeLayout } = usePlotlyTheme({
    title: 'Requested Hours Distribution',
    yAxis: { title: 'Requested Hours' },
  });

  // ── Chart data ───────────────────────────────────────────────────
  const priorityDistributionData = useMemo((): Plotly.Data[] => {
    if (!data) return [];
    const currentPriorities = data.current_blocks.map((b) => b.priority);
    const comparisonPriorities = data.comparison_blocks.map((b) => b.priority);

    const currentTrace = {
      x: currentPriorities,
      type: 'histogram' as const,
      name: data.current_name,
      opacity: 0.7,
      marker: { color: SCHEDULE_COLORS.current },
      nbinsx: 30,
    };

    const comparisonTrace = {
      x: comparisonPriorities,
      type: 'histogram' as const,
      name: data.comparison_name,
      opacity: 0.7,
      marker: { color: SCHEDULE_COLORS.comparison },
      nbinsx: 30,
    };

    // Larger dataset first for better visibility
    return currentPriorities.length >= comparisonPriorities.length
      ? [currentTrace, comparisonTrace]
      : [comparisonTrace, currentTrace];
  }, [data]);

  const schedulingStatusData = useMemo((): Plotly.Data[] => {
    if (!data) return [];
    return [
      {
        name: data.current_name,
        x: ['Scheduled', 'Unscheduled'],
        y: [data.current_stats.scheduled_count, data.current_stats.unscheduled_count],
        type: 'bar',
        marker: { color: SCHEDULE_COLORS.current },
        text: [
          data.current_stats.scheduled_count.toLocaleString(),
          data.current_stats.unscheduled_count.toLocaleString(),
        ],
        textposition: 'auto',
        textfont: { color: 'white', size: 12 },
      },
      {
        name: data.comparison_name,
        x: ['Scheduled', 'Unscheduled'],
        y: [data.comparison_stats.scheduled_count, data.comparison_stats.unscheduled_count],
        type: 'bar',
        marker: { color: SCHEDULE_COLORS.comparison },
        text: [
          data.comparison_stats.scheduled_count.toLocaleString(),
          data.comparison_stats.unscheduled_count.toLocaleString(),
        ],
        textposition: 'auto',
        textfont: { color: 'white', size: 12 },
      },
    ];
  }, [data]);

  const changesData = useMemo((): Plotly.Data[] => {
    if (!data) return [];
    const newlyScheduled = data.scheduling_changes.filter(
      (c) => c.change_type === 'newly_scheduled'
    ).length;
    const newlyUnscheduled = data.scheduling_changes.filter(
      (c) => c.change_type === 'newly_unscheduled'
    ).length;

    return [
      {
        x: ['Newly Scheduled', 'Newly Unscheduled'],
        y: [newlyScheduled, newlyUnscheduled],
        type: 'bar',
        marker: { color: ['#22c55e', '#ef4444'] },
        text: [newlyScheduled.toLocaleString(), newlyUnscheduled.toLocaleString()],
        textposition: 'auto',
        textfont: { color: 'white', size: 14 },
      },
    ];
  }, [data]);

  const timeDistributionData = useMemo((): Plotly.Data[] => {
    if (!data) return [];
    return [
      {
        y: data.current_blocks.map((b) => b.requested_hours),
        type: 'box',
        name: data.current_name,
        marker: { color: SCHEDULE_COLORS.current },
        fillcolor: SCHEDULE_COLORS.current,
        line: { color: SCHEDULE_COLORS.currentEdge, width: 2 },
        boxmean: 'sd',
      },
      {
        y: data.comparison_blocks.map((b) => b.requested_hours),
        type: 'box',
        name: data.comparison_name,
        marker: { color: SCHEDULE_COLORS.comparison },
        fillcolor: SCHEDULE_COLORS.comparison,
        line: { color: SCHEDULE_COLORS.comparisonEdge, width: 2 },
        boxmean: 'sd',
      },
    ];
  }, [data]);

  // ── Loading / error states ───────────────────────────────────────
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

  if (!data) {
    return <ErrorMessage message="No data available" />;
  }

  return (
    <PageContainer>
      <PageHeader
        title="Compare Schedules"
        description={`${data.current_name} vs ${data.comparison_name}`}
      />

      {/* Side-by-side stats */}
      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        <ChartPanel title={data.current_name}>
          <div className="grid grid-cols-2 gap-4">
            <MetricCard
              label="Scheduled"
              value={data.current_stats.scheduled_count}
              icon={<Icon name="check-circle" />}
            />
            <MetricCard
              label="Unscheduled"
              value={data.current_stats.unscheduled_count}
              icon={<Icon name="x-circle" />}
            />
            <MetricCard
              label="Mean Priority"
              value={data.current_stats.mean_priority.toFixed(2)}
              icon={<Icon name="star" />}
            />
            <MetricCard
              label="Total Hours"
              value={`${data.current_stats.total_hours.toFixed(1)}h`}
              icon={<Icon name="clock" />}
            />
          </div>
        </ChartPanel>

        <ChartPanel title={data.comparison_name}>
          <div className="grid grid-cols-2 gap-4">
            <MetricCard
              label="Scheduled"
              value={data.comparison_stats.scheduled_count}
              icon={<Icon name="check-circle" />}
            />
            <MetricCard
              label="Unscheduled"
              value={data.comparison_stats.unscheduled_count}
              icon={<Icon name="x-circle" />}
            />
            <MetricCard
              label="Mean Priority"
              value={data.comparison_stats.mean_priority.toFixed(2)}
              icon={<Icon name="star" />}
            />
            <MetricCard
              label="Total Hours"
              value={`${data.comparison_stats.total_hours.toFixed(1)}h`}
              icon={<Icon name="clock" />}
            />
          </div>
        </ChartPanel>
      </div>

      {/* Overlap summary */}
      <MetricsGrid>
        <MetricCard
          label="Common Blocks"
          value={data.common_ids.length}
          icon={<Icon name="link" />}
        />
        <MetricCard
          label={`Only in ${data.current_name}`}
          value={data.only_in_current.length}
          icon={<Icon name="arrow-right" />}
        />
        <MetricCard
          label={`Only in ${data.comparison_name}`}
          value={data.only_in_comparison.length}
          icon={<Icon name="arrow-left" />}
        />
      </MetricsGrid>

      {/* Charts in 2×2 grid */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        <ChartPanel title="Priority Distribution">
          <PlotlyChart
            data={priorityDistributionData}
            layout={priorityLayout}
            config={config}
            height="400px"
          />
        </ChartPanel>

        <ChartPanel title="Scheduling Status">
          <PlotlyChart
            data={schedulingStatusData}
            layout={statusLayout}
            config={config}
            height="400px"
          />
        </ChartPanel>

        <ChartPanel title="Scheduling Changes">
          <PlotlyChart data={changesData} layout={changesLayout} config={config} height="350px" />
        </ChartPanel>

        <ChartPanel title="Requested Hours">
          <PlotlyChart
            data={timeDistributionData}
            layout={timeLayout}
            config={config}
            height="350px"
          />
        </ChartPanel>
      </div>

      {/* Scheduling changes table */}
      {data.scheduling_changes.length > 0 && (
        <ChartPanel title={`Scheduling Changes (${data.scheduling_changes.length})`}>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700">
                  <th className="px-4 py-3 text-left text-slate-400">Block ID</th>
                  <th className="px-4 py-3 text-right text-slate-400">Priority</th>
                  <th className="px-4 py-3 text-center text-slate-400">Change</th>
                </tr>
              </thead>
              <tbody>
                {data.scheduling_changes.slice(0, 20).map((change) => (
                  <tr key={change.scheduling_block_id} className="border-b border-slate-700/50">
                    <td className="px-4 py-3 text-white">{change.scheduling_block_id}</td>
                    <td className="px-4 py-3 text-right text-slate-300">
                      {change.priority.toFixed(2)}
                    </td>
                    <td className="px-4 py-3 text-center">
                      <span
                        className={`rounded px-2 py-1 text-xs ${
                          CHANGE_TYPE_COLORS[change.change_type as ChangeTypeKey] ||
                          'bg-slate-500/20 text-slate-400'
                        }`}
                      >
                        {change.change_type}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {data.scheduling_changes.length > 20 && (
              <p className="mt-4 text-center text-slate-400">
                ... and {data.scheduling_changes.length - 20} more changes
              </p>
            )}
          </div>
        </ChartPanel>
      )}
    </PageContainer>
  );
}

export default Compare;
