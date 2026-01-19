/**
 * Distributions page - Statistical analysis of schedule properties.
 * Redesigned with consistent layout primitives and improved chart presentation.
 */
import { useParams } from 'react-router-dom';
import { useDistributions, usePlotlyTheme } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  MetricCard,
  PlotlyChart,
  PageHeader,
  PageContainer,
  MetricsGrid,
  ChartPanel,
} from '@/components';
import { STATUS_COLORS } from '@/constants/colors';

function Distributions() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useDistributions(id);

  // Call hooks unconditionally (rules of hooks)
  const { layout: priorityLayout, config } = usePlotlyTheme({
    title: 'Priority Distribution',
    xAxis: { title: 'Priority' },
    yAxis: { title: 'Count' },
    barMode: 'overlay',
  });

  const { layout: visibilityLayout } = usePlotlyTheme({
    title: 'Visibility Distribution',
    xAxis: { title: 'Total Visibility (hours)' },
    yAxis: { title: 'Count' },
    barMode: 'overlay',
  });

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
        title="Failed to load distributions"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }

  if (!data) {
    return <ErrorMessage message="No data available" />;
  }

  // Priority histogram
  const priorityHistogram: Plotly.Data[] = [
    {
      type: 'histogram',
      x: data.blocks.filter((b) => b.scheduled).map((b) => b.priority),
      name: 'Scheduled',
      marker: { color: STATUS_COLORS.scheduled },
      opacity: 0.7,
    },
    {
      type: 'histogram',
      x: data.blocks.filter((b) => !b.scheduled).map((b) => b.priority),
      name: 'Unscheduled',
      marker: { color: STATUS_COLORS.unscheduled },
      opacity: 0.7,
    },
  ];

  // Visibility histogram
  const visibilityHistogram: Plotly.Data[] = [
    {
      type: 'histogram',
      x: data.blocks.filter((b) => b.scheduled).map((b) => b.total_visibility_hours),
      name: 'Scheduled',
      marker: { color: STATUS_COLORS.scheduled },
      opacity: 0.7,
    },
    {
      type: 'histogram',
      x: data.blocks.filter((b) => !b.scheduled).map((b) => b.total_visibility_hours),
      name: 'Unscheduled',
      marker: { color: STATUS_COLORS.unscheduled },
      opacity: 0.7,
    },
  ];

  return (
    <PageContainer>
      {/* Header */}
      <PageHeader
        title="Distributions"
        description="Statistical analysis of schedule properties"
      />

      {/* Summary metrics */}
      <MetricsGrid>
        <MetricCard label="Total Blocks" value={data.total_count} icon="📊" />
        <MetricCard label="Scheduled" value={data.scheduled_count} icon="✅" />
        <MetricCard label="Unscheduled" value={data.unscheduled_count} icon="❌" />
        <MetricCard label="Impossible" value={data.impossible_count} icon="🚫" />
      </MetricsGrid>

      {/* Priority section */}
      <section>
        <div className="mb-4 flex items-baseline justify-between">
          <h2 className="text-lg font-semibold text-white">Priority Distribution</h2>
        </div>

        {/* Priority stats */}
        <div className="mb-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
          <div className="rounded-lg border border-slate-700 bg-slate-800/30 p-3">
            <p className="text-xs text-slate-400">Mean</p>
            <p className="text-lg font-semibold text-white">
              {data.priority_stats.mean.toFixed(2)}
            </p>
          </div>
          <div className="rounded-lg border border-slate-700 bg-slate-800/30 p-3">
            <p className="text-xs text-slate-400">Median</p>
            <p className="text-lg font-semibold text-white">
              {data.priority_stats.median.toFixed(2)}
            </p>
          </div>
          <div className="rounded-lg border border-slate-700 bg-slate-800/30 p-3">
            <p className="text-xs text-slate-400">Std Dev</p>
            <p className="text-lg font-semibold text-white">
              {data.priority_stats.std_dev.toFixed(2)}
            </p>
          </div>
          <div className="rounded-lg border border-slate-700 bg-slate-800/30 p-3">
            <p className="text-xs text-slate-400">Range</p>
            <p className="text-lg font-semibold text-white">
              {data.priority_stats.min.toFixed(1)} – {data.priority_stats.max.toFixed(1)}
            </p>
          </div>
        </div>

        <ChartPanel>
          <PlotlyChart
            data={priorityHistogram}
            layout={priorityLayout}
            config={config}
            height="350px"
          />
        </ChartPanel>
      </section>

      {/* Visibility section */}
      <section>
        <div className="mb-4 flex items-baseline justify-between">
          <h2 className="text-lg font-semibold text-white">Visibility Distribution</h2>
        </div>

        {/* Visibility stats */}
        <div className="mb-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
          <div className="rounded-lg border border-slate-700 bg-slate-800/30 p-3">
            <p className="text-xs text-slate-400">Mean</p>
            <p className="text-lg font-semibold text-white">
              {data.visibility_stats.mean.toFixed(1)}h
            </p>
          </div>
          <div className="rounded-lg border border-slate-700 bg-slate-800/30 p-3">
            <p className="text-xs text-slate-400">Median</p>
            <p className="text-lg font-semibold text-white">
              {data.visibility_stats.median.toFixed(1)}h
            </p>
          </div>
          <div className="rounded-lg border border-slate-700 bg-slate-800/30 p-3">
            <p className="text-xs text-slate-400">Std Dev</p>
            <p className="text-lg font-semibold text-white">
              {data.visibility_stats.std_dev.toFixed(1)}h
            </p>
          </div>
          <div className="rounded-lg border border-slate-700 bg-slate-800/30 p-3">
            <p className="text-xs text-slate-400">Range</p>
            <p className="text-lg font-semibold text-white">
              {data.visibility_stats.min.toFixed(0)} – {data.visibility_stats.max.toFixed(0)}h
            </p>
          </div>
        </div>

        <ChartPanel>
          <PlotlyChart
            data={visibilityHistogram}
            layout={visibilityLayout}
            config={config}
            height="350px"
          />
        </ChartPanel>
      </section>
    </PageContainer>
  );
}

export default Distributions;
