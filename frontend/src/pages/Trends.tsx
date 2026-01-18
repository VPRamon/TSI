/**
 * Trends page - Scheduling trends and patterns.
 */
import { useState } from 'react';
import { useParams } from 'react-router-dom';
import Plot from 'react-plotly.js';
import { useTrends } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard } from '@/components';

function Trends() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);
  
  // Query parameters
  const [bins, setBins] = useState(10);
  const [bandwidth, setBandwidth] = useState(0.5);
  
  const { data, isLoading, error, refetch } = useTrends(id, { bins, bandwidth });

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
      line: { color: '#22c55e' },
      marker: { size: 8 },
    },
    {
      type: 'scatter',
      mode: 'lines',
      name: 'Smoothed',
      x: data.smoothed_visibility.map((p) => p.x),
      y: data.smoothed_visibility.map((p) => p.y_smoothed * 100),
      line: { color: '#f59e0b', dash: 'dash' },
    },
  ];

  const baseLayout: Partial<Plotly.Layout> = {
    paper_bgcolor: 'transparent',
    plot_bgcolor: '#1e293b',
    font: { color: '#94a3b8' },
    margin: { t: 50, r: 20, b: 60, l: 60 },
    legend: {
      orientation: 'h',
      y: -0.15,
      font: { color: '#94a3b8' },
    },
  };

  const byPriorityLayout: Partial<Plotly.Layout> = {
    ...baseLayout,
    title: { text: 'Scheduling Rate by Priority', font: { color: '#fff' } },
    xaxis: { title: 'Priority Bin', gridcolor: '#334155' },
    yaxis: { title: 'Scheduling Rate (%)', gridcolor: '#334155', range: [0, 100] },
  };

  const byVisibilityLayout: Partial<Plotly.Layout> = {
    ...baseLayout,
    title: { text: 'Scheduling Rate by Visibility', font: { color: '#fff' } },
    xaxis: { title: 'Visibility (hours)', gridcolor: '#334155' },
    yaxis: { title: 'Scheduling Rate (%)', gridcolor: '#334155', range: [0, 100] },
  };

  const config: Partial<Plotly.Config> = {
    responsive: true,
    displayModeBar: true,
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Trends</h1>
        <p className="text-slate-400 mt-1">
          Scheduling trends and patterns analysis
        </p>
      </div>

      {/* Controls */}
      <Card title="Parameters">
        <div className="flex flex-wrap gap-6">
          <div>
            <label className="block text-sm text-slate-400 mb-1">Number of Bins</label>
            <input
              type="number"
              value={bins}
              onChange={(e) => setBins(parseInt(e.target.value, 10) || 10)}
              min={5}
              max={20}
              className="w-24 px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white"
            />
          </div>
          <div>
            <label className="block text-sm text-slate-400 mb-1">Bandwidth</label>
            <input
              type="number"
              value={bandwidth}
              onChange={(e) => setBandwidth(parseFloat(e.target.value) || 0.5)}
              min={0.1}
              max={2}
              step={0.1}
              className="w-24 px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white"
            />
          </div>
        </div>
      </Card>

      {/* Metrics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <MetricCard
          label="Total Blocks"
          value={metrics.total_count}
          icon="📊"
        />
        <MetricCard
          label="Scheduling Rate"
          value={`${(metrics.scheduling_rate * 100).toFixed(1)}%`}
          icon="📈"
        />
        <MetricCard
          label="Zero Visibility"
          value={metrics.zero_visibility_count}
          icon="🚫"
        />
        <MetricCard
          label="Priority Mean"
          value={metrics.priority_mean.toFixed(2)}
          icon="⭐"
        />
      </div>

      {/* By Priority chart */}
      <Card title="By Priority">
        <div className="h-[400px]">
          <Plot
            data={byPriorityData}
            layout={byPriorityLayout}
            config={config}
            style={{ width: '100%', height: '100%' }}
            useResizeHandler
          />
        </div>
      </Card>

      {/* By Visibility chart */}
      <Card title="By Visibility">
        <div className="h-[400px]">
          <Plot
            data={byVisibilityData}
            layout={byVisibilityLayout}
            config={config}
            style={{ width: '100%', height: '100%' }}
            useResizeHandler
          />
        </div>
      </Card>
    </div>
  );
}

export default Trends;
