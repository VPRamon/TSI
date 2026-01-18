/**
 * Distributions page - Statistical analysis of schedule properties.
 */
import { useParams } from 'react-router-dom';
import { useDistributions, usePlotlyTheme } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard, PlotlyChart } from '@/components';
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
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Distributions</h1>
        <p className="mt-1 text-slate-400">Statistical analysis of schedule properties</p>
      </div>

      {/* Metrics */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <MetricCard label="Total Blocks" value={data.total_count} icon="📊" />
        <MetricCard label="Scheduled" value={data.scheduled_count} icon="✅" />
        <MetricCard label="Unscheduled" value={data.unscheduled_count} icon="❌" />
        <MetricCard label="Impossible" value={data.impossible_count} icon="🚫" />
      </div>

      {/* Priority stats */}
      <Card title="Priority Statistics">
        <div className="mb-6 grid grid-cols-2 gap-4 md:grid-cols-4">
          <MetricCard label="Mean" value={data.priority_stats.mean.toFixed(2)} />
          <MetricCard label="Median" value={data.priority_stats.median.toFixed(2)} />
          <MetricCard label="Std Dev" value={data.priority_stats.std_dev.toFixed(2)} />
          <MetricCard
            label="Range"
            value={`${data.priority_stats.min.toFixed(1)} - ${data.priority_stats.max.toFixed(1)}`}
          />
        </div>
        <PlotlyChart data={priorityHistogram} layout={priorityLayout} config={config} height="400px" />
      </Card>

      {/* Visibility stats */}
      <Card title="Visibility Statistics">
        <div className="mb-6 grid grid-cols-2 gap-4 md:grid-cols-4">
          <MetricCard label="Mean" value={`${data.visibility_stats.mean.toFixed(1)}h`} />
          <MetricCard label="Median" value={`${data.visibility_stats.median.toFixed(1)}h`} />
          <MetricCard label="Std Dev" value={`${data.visibility_stats.std_dev.toFixed(1)}h`} />
          <MetricCard
            label="Range"
            value={`${data.visibility_stats.min.toFixed(0)} - ${data.visibility_stats.max.toFixed(0)}h`}
          />
        </div>
        <PlotlyChart
          data={visibilityHistogram}
          layout={visibilityLayout}
          config={config}
          height="400px"
        />
      </Card>
    </div>
  );
}

export default Distributions;
