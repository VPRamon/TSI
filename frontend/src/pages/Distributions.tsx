/**
 * Distributions page - Statistical analysis of schedule properties.
 */
import { useParams } from 'react-router-dom';
import Plot from 'react-plotly.js';
import { useDistributions } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard } from '@/components';

function Distributions() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useDistributions(id);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
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
      marker: { color: '#22c55e' },
      opacity: 0.7,
    },
    {
      type: 'histogram',
      x: data.blocks.filter((b) => !b.scheduled).map((b) => b.priority),
      name: 'Unscheduled',
      marker: { color: '#ef4444' },
      opacity: 0.7,
    },
  ];

  // Visibility histogram
  const visibilityHistogram: Plotly.Data[] = [
    {
      type: 'histogram',
      x: data.blocks.filter((b) => b.scheduled).map((b) => b.total_visibility_hours),
      name: 'Scheduled',
      marker: { color: '#22c55e' },
      opacity: 0.7,
    },
    {
      type: 'histogram',
      x: data.blocks.filter((b) => !b.scheduled).map((b) => b.total_visibility_hours),
      name: 'Unscheduled',
      marker: { color: '#ef4444' },
      opacity: 0.7,
    },
  ];

  const baseLayout: Partial<Plotly.Layout> = {
    paper_bgcolor: 'transparent',
    plot_bgcolor: '#1e293b',
    font: { color: '#94a3b8' },
    barmode: 'overlay',
    legend: {
      orientation: 'h',
      y: -0.15,
      font: { color: '#94a3b8' },
    },
    margin: { t: 50, r: 20, b: 60, l: 60 },
  };

  const priorityLayout: Partial<Plotly.Layout> = {
    ...baseLayout,
    title: { text: 'Priority Distribution', font: { color: '#fff' } },
    xaxis: {
      title: { text: 'Priority' },
      gridcolor: '#334155',
    },
    yaxis: {
      title: { text: 'Count' },
      gridcolor: '#334155',
    },
  };

  const visibilityLayout: Partial<Plotly.Layout> = {
    ...baseLayout,
    title: { text: 'Visibility Distribution', font: { color: '#fff' } },
    xaxis: {
      title: { text: 'Total Visibility (hours)' },
      gridcolor: '#334155',
    },
    yaxis: {
      title: { text: 'Count' },
      gridcolor: '#334155',
    },
  };

  const config: Partial<Plotly.Config> = {
    responsive: true,
    displayModeBar: true,
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Distributions</h1>
        <p className="text-slate-400 mt-1">
          Statistical analysis of schedule properties
        </p>
      </div>

      {/* Metrics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <MetricCard label="Total Blocks" value={data.total_count} icon="📊" />
        <MetricCard label="Scheduled" value={data.scheduled_count} icon="✅" />
        <MetricCard label="Unscheduled" value={data.unscheduled_count} icon="❌" />
        <MetricCard label="Impossible" value={data.impossible_count} icon="🚫" />
      </div>

      {/* Priority stats */}
      <Card title="Priority Statistics">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
          <MetricCard label="Mean" value={data.priority_stats.mean.toFixed(2)} />
          <MetricCard label="Median" value={data.priority_stats.median.toFixed(2)} />
          <MetricCard label="Std Dev" value={data.priority_stats.std_dev.toFixed(2)} />
          <MetricCard
            label="Range"
            value={`${data.priority_stats.min.toFixed(1)} - ${data.priority_stats.max.toFixed(1)}`}
          />
        </div>
        <div className="h-[400px]">
          <Plot
            data={priorityHistogram}
            layout={priorityLayout}
            config={config}
            style={{ width: '100%', height: '100%' }}
            useResizeHandler
          />
        </div>
      </Card>

      {/* Visibility stats */}
      <Card title="Visibility Statistics">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
          <MetricCard label="Mean" value={`${data.visibility_stats.mean.toFixed(1)}h`} />
          <MetricCard label="Median" value={`${data.visibility_stats.median.toFixed(1)}h`} />
          <MetricCard label="Std Dev" value={`${data.visibility_stats.std_dev.toFixed(1)}h`} />
          <MetricCard
            label="Range"
            value={`${data.visibility_stats.min.toFixed(0)} - ${data.visibility_stats.max.toFixed(0)}h`}
          />
        </div>
        <div className="h-[400px]">
          <Plot
            data={visibilityHistogram}
            layout={visibilityLayout}
            config={config}
            style={{ width: '100%', height: '100%' }}
            useResizeHandler
          />
        </div>
      </Card>
    </div>
  );
}

export default Distributions;
