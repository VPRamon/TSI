/**
 * Trends page - Scheduling trends and patterns.
 * Redesigned with controls alongside charts.
 */
import { useState } from 'react';
import { useParams } from 'react-router-dom';
import { useTrends, usePlotlyTheme } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  MetricCard,
  PlotlyChart,
  PageHeader,
  PageContainer,
  SplitPane,
  MetricsGrid,
  ChartPanel,
} from '@/components';
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

  // Controls panel content
  const controlsContent = (
    <div className="space-y-5">
      <h3 className="text-sm font-medium text-slate-200">Parameters</h3>

      {/* Number of bins */}
      <div>
        <label className="mb-1.5 block text-xs font-medium text-slate-400">
          Number of Bins
        </label>
        <div className="flex items-center gap-3">
          <input
            type="range"
            min="5"
            max="20"
            value={bins}
            onChange={(e) => setBins(parseInt(e.target.value, 10))}
            className="h-2 flex-1 cursor-pointer appearance-none rounded-lg bg-slate-600"
          />
          <input
            type="number"
            value={bins}
            onChange={(e) => setBins(parseInt(e.target.value, 10) || 10)}
            min={5}
            max={20}
            className="w-16 rounded-md border border-slate-600 bg-slate-700 px-2 py-1.5 text-center text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
        </div>
        <p className="mt-1 text-xs text-slate-500">Controls histogram resolution</p>
      </div>

      {/* Bandwidth */}
      <div>
        <label className="mb-1.5 block text-xs font-medium text-slate-400">
          Smoothing Bandwidth
        </label>
        <div className="flex items-center gap-3">
          <input
            type="range"
            min="0.1"
            max="2"
            step="0.1"
            value={bandwidth}
            onChange={(e) => setBandwidth(parseFloat(e.target.value))}
            className="h-2 flex-1 cursor-pointer appearance-none rounded-lg bg-slate-600"
          />
          <input
            type="number"
            value={bandwidth}
            onChange={(e) => setBandwidth(parseFloat(e.target.value) || 0.5)}
            min={0.1}
            max={2}
            step={0.1}
            className="w-16 rounded-md border border-slate-600 bg-slate-700 px-2 py-1.5 text-center text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
        </div>
        <p className="mt-1 text-xs text-slate-500">Kernel bandwidth for smoothed curve</p>
      </div>

      {/* Summary stats */}
      <div className="border-t border-slate-700 pt-4">
        <h4 className="mb-3 text-xs font-medium uppercase tracking-wide text-slate-500">
          Summary
        </h4>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-slate-400">Scheduling Rate</span>
            <span className="font-medium text-white">
              {(metrics.scheduling_rate * 100).toFixed(1)}%
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-slate-400">Zero Visibility</span>
            <span className="font-medium text-white">{metrics.zero_visibility_count}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-slate-400">Mean Priority</span>
            <span className="font-medium text-white">{metrics.priority_mean.toFixed(2)}</span>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <PageContainer>
      {/* Header */}
      <PageHeader
        title="Trends"
        description="Scheduling trends and patterns analysis"
      />

      {/* Metrics */}
      <MetricsGrid>
        <MetricCard label="Total Blocks" value={metrics.total_count} icon="📊" />
        <MetricCard
          label="Scheduling Rate"
          value={`${(metrics.scheduling_rate * 100).toFixed(1)}%`}
          icon="📈"
        />
        <MetricCard label="Zero Visibility" value={metrics.zero_visibility_count} icon="🚫" />
        <MetricCard label="Priority Mean" value={metrics.priority_mean.toFixed(2)} icon="⭐" />
      </MetricsGrid>

      {/* Split layout: controls left, charts right */}
      <SplitPane controls={controlsContent} controlsWidth="sm">
        <div className="flex flex-col gap-6">
          <ChartPanel title="By Priority">
            <PlotlyChart
              data={byPriorityData}
              layout={byPriorityLayout}
              config={config}
              height="350px"
            />
          </ChartPanel>

          <ChartPanel title="By Visibility">
            <PlotlyChart
              data={byVisibilityData}
              layout={byVisibilityLayout}
              config={config}
              height="350px"
            />
          </ChartPanel>
        </div>
      </SplitPane>
    </PageContainer>
  );
}

export default Trends;
