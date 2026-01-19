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

  // Call hook unconditionally (rules of hooks)
  const { layout, config } = usePlotlyTheme({
    title: 'Observation Timeline',
    xAxis: { title: 'Day of Month' },
    yAxis: { title: 'Month' },
    showLegend: false,
  });

  // Override axes for calendar-style layout
  const timelineLayout = {
    ...layout,
    xaxis: {
      ...layout.xaxis,
      title: 'Day of Month',
      range: [0, 32],
      dtick: 1,
    },
    yaxis: {
      ...layout.yaxis,
      title: 'Month',
      tickmode: 'array',
      tickvals: data.unique_months.map((_, index) => index),
      ticktext: data.unique_months,
      autorange: 'reversed',
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

  // Group blocks by month and create month mapping
  const monthMap = new Map<string, number>();
  data.unique_months.forEach((month, index) => {
    monthMap.set(month, index);
  });

  // Create calendar-style timeline with scatter plot
  const plotData: Plotly.Data[] = data.blocks.map((block) => {
    const startDate = mjdToDate(block.scheduled_start_mjd);
    const stopDate = mjdToDate(block.scheduled_stop_mjd);
    const monthKey = `${startDate.getFullYear()}-${String(startDate.getMonth() + 1).padStart(2, '0')}`;
    const monthIndex = monthMap.get(monthKey) ?? 0;
    
    // Calculate fractional day positions for start and stop
    const startDay = startDate.getDate() + startDate.getHours() / 24 + startDate.getMinutes() / 1440;
    const stopDay = stopDate.getDate() + stopDate.getHours() / 24 + stopDate.getMinutes() / 1440;
    
    return {
      type: 'scatter',
      mode: 'lines',
      name: block.original_block_id,
      x: [startDay, stopDay],
      y: [monthIndex, monthIndex],
      line: {
        width: 20,
        color: `hsl(${(block.priority / 10) * 240}, 70%, 50%)`,
      },
      text: `${block.original_block_id}<br>Start: ${startDate.toISOString()}<br>Priority: ${block.priority.toFixed(1)}<br>Duration: ${block.requested_hours.toFixed(1)}h`,
      hoverinfo: 'text',
      showlegend: false,
    };
  });

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
