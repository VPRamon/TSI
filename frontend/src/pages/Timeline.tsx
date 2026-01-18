/**
 * Timeline page - Scheduled observations over time.
 */
import { useParams } from 'react-router-dom';
import { useTimeline, usePlotlyTheme } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard, PlotlyChart } from '@/components';
import { mjdToDate } from '@/constants/dates';

function Timeline() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useTimeline(id);

  // Add dark periods as background shapes (compute before hook call)
  const shapes: Partial<Plotly.Shape>[] = (data?.dark_periods ?? []).map((period) => ({
    type: 'rect',
    xref: 'x',
    yref: 'paper',
    x0: mjdToDate(period.start),
    x1: mjdToDate(period.stop),
    y0: 0,
    y1: 1,
    fillcolor: 'rgba(30, 58, 138, 0.2)',
    line: { width: 0 },
  }));

  // Call hook unconditionally (rules of hooks)
  const { layout, config } = usePlotlyTheme({
    title: 'Observation Timeline',
    xAxis: { title: 'Date', type: 'date' },
    yAxis: { title: 'Observation' },
    shapes,
    showLegend: false,
  });

  // Override yaxis to hide tick labels
  const timelineLayout = {
    ...layout,
    yaxis: {
      ...layout.yaxis,
      showticklabels: false,
    },
  };

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
        title="Failed to load timeline"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }

  if (!data) {
    return <ErrorMessage message="No data available" />;
  }

  // Create Gantt-like chart with scatter plot
  const plotData: Plotly.Data[] = data.blocks.map((block, index) => ({
    type: 'scatter',
    mode: 'lines',
    name: block.original_block_id,
    x: [mjdToDate(block.scheduled_start_mjd), mjdToDate(block.scheduled_stop_mjd)],
    y: [index, index],
    line: {
      width: 8,
      color: `hsl(${(block.priority / 10) * 240}, 70%, 50%)`,
    },
    text: `${block.original_block_id}<br>Priority: ${block.priority.toFixed(1)}<br>Duration: ${block.requested_hours.toFixed(1)}h`,
    hoverinfo: 'text',
    showlegend: false,
  }));

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Timeline</h1>
        <p className="mt-1 text-slate-400">Scheduled observations over time</p>
      </div>

      {/* Metrics */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <MetricCard label="Total Blocks" value={data.total_count} icon="📅" />
        <MetricCard label="Scheduled" value={data.scheduled_count} icon="✅" />
        <MetricCard label="Unique Months" value={data.unique_months.length} icon="📆" />
        <MetricCard label="Dark Periods" value={data.dark_periods.length} icon="🌙" />
      </div>

      {/* Timeline chart */}
      <Card title="Schedule Timeline">
        <PlotlyChart data={plotData} layout={timelineLayout} config={config} height="600px" />
      </Card>

      {/* Months list */}
      <Card title="Covered Months">
        <div className="flex flex-wrap gap-2">
          {data.unique_months.map((month) => (
            <span
              key={month}
              className="rounded-full bg-slate-700 px-3 py-1 text-sm text-slate-300"
            >
              {month}
            </span>
          ))}
        </div>
      </Card>
    </div>
  );
}

export default Timeline;
