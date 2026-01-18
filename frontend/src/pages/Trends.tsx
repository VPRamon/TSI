/**
 * Trends page - Scheduling trends and patterns.
 */
import { useState } from 'react';
import { useParams } from 'react-router-dom';
import { useTrends, usePlotlyTheme } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard, PlotlyChart } from '@/components';
import { STATUS_COLORS } from '@/constants/colors';

function Trends() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);

  // Query parameters
  const [bins, setBins] = useState(10);
  const [bandwidth, setBandwidth] = useState(0.5);

  const { data, isLoading, error, refetch } = useTrends(id, { bins, bandwidth });

  // Call hooks unconditionally (rules of hooks)
  const { layout: byPriorityLayout, config } = usePlotlyTheme({
    title: 'Scheduling Rate by Priority',
    xAxis: { title: 'Priority Bin' },
    yAxis: { title: 'Scheduling Rate (%)', range: [0, 100] },
  });

  const { layout: byVisibilityLayout } = usePlotlyTheme({
    title: 'Scheduling Rate by Visibility',
    xAxis: { title: 'Visibility (hours)' },
    yAxis: { title: 'Scheduling Rate (%)', range: [0, 100] },
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
        title="Failed to load trends"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }

  if (!data) {
    return <ErrorMessage message="No data available" />;
  }

  const { metrics } = data;

  // By Priority chart
  const byPriorityData: Plotly.Data[] = [
    {
      type: 'bar',
      name: 'Scheduling Rate',
      x: data.by_priority.map((p) => p.bin_label),
      y: data.by_priority.map((p) => p.scheduled_rate * 100),
      marker: { color: '#3b82f6' },
      text: data.by_priority.map((p) => `${(p.scheduled_rate * 100).toFixed(1)}%`),
      textposition: 'outside',
    },
  ];

  // By Visibility chart
  const byVisibilityData: Plotly.Data[] = [
    {
      type: 'scatter',
      mode: 'lines+markers',
      name: 'Empirical Rate',
      x: data.by_visibility.map((p) => p.mid_value),
      y: data.by_visibility.map((p) => p.scheduled_rate * 100),
      line: { color: STATUS_COLORS.scheduled },
      marker: { size: 8 },
    },
    {
      type: 'scatter',
      mode: 'lines',
      name: 'Smoothed',
      x: data.smoothed_visibility.map((p) => p.x),
      y: data.smoothed_visibility.map((p) => p.y_smoothed * 100),
      line: { color: STATUS_COLORS.impossible, dash: 'dash' },
    },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Trends</h1>
        <p className="mt-1 text-slate-400">Scheduling trends and patterns analysis</p>
      </div>

      {/* Controls */}
      <Card title="Parameters">
        <div className="flex flex-wrap gap-6">
          <div>
            <label className="mb-1 block text-sm text-slate-400">Number of Bins</label>
            <input
              type="number"
              value={bins}
              onChange={(e) => setBins(parseInt(e.target.value, 10) || 10)}
              min={5}
              max={20}
              className="w-24 rounded-lg border border-slate-600 bg-slate-700 px-3 py-2 text-white"
            />
          </div>
          <div>
            <label className="mb-1 block text-sm text-slate-400">Bandwidth</label>
            <input
              type="number"
              value={bandwidth}
              onChange={(e) => setBandwidth(parseFloat(e.target.value) || 0.5)}
              min={0.1}
              max={2}
              step={0.1}
              className="w-24 rounded-lg border border-slate-600 bg-slate-700 px-3 py-2 text-white"
            />
          </div>
        </div>
      </Card>

      {/* Metrics */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <MetricCard label="Total Blocks" value={metrics.total_count} icon="📊" />
        <MetricCard
          label="Scheduling Rate"
          value={`${(metrics.scheduling_rate * 100).toFixed(1)}%`}
          icon="📈"
        />
        <MetricCard label="Zero Visibility" value={metrics.zero_visibility_count} icon="🚫" />
        <MetricCard label="Priority Mean" value={metrics.priority_mean.toFixed(2)} icon="⭐" />
      </div>

      {/* By Priority chart */}
      <Card title="By Priority">
        <PlotlyChart data={byPriorityData} layout={byPriorityLayout} config={config} height="400px" />
      </Card>

      {/* By Visibility chart */}
      <Card title="By Visibility">
        <PlotlyChart
          data={byVisibilityData}
          layout={byVisibilityLayout}
          config={config}
          height="400px"
        />
      </Card>
    </div>
  );
}

export default Trends;
