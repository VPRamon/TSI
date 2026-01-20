/**
 * Timeline page - Scheduled observations over time.
 * Redesigned with consistent layout primitives.
 */
import { useParams } from 'react-router-dom';
import { useTimeline, usePlotlyTheme } from '@/hooks';
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

  // Height of each observation bar in data units (0.4 = 40% of row height)
  const barHeight = 0.4;

  // Create shapes for each observation block (renders reliably, no aliasing)
  const shapes = data.blocks.map((block) => {
    const startDate = mjdToDate(block.scheduled_start_mjd);
    const stopDate = mjdToDate(block.scheduled_stop_mjd);
    const monthKey = `${startDate.getFullYear()}-${String(startDate.getMonth() + 1).padStart(2, '0')}`;
    const monthIndex = monthMap.get(monthKey) ?? 0;
    
    // Calculate fractional day positions for start and stop
    const startDay = startDate.getDate() + startDate.getHours() / 24 + startDate.getMinutes() / 1440;
    const stopDay = stopDate.getDate() + stopDate.getHours() / 24 + stopDate.getMinutes() / 1440;
    
    return {
      type: 'rect' as const,
      xref: 'x' as const,
      yref: 'y' as const,
      x0: startDay,
      x1: stopDay,
      y0: monthIndex - barHeight / 2,
      y1: monthIndex + barHeight / 2,
      fillcolor: `hsl(${(block.priority / 10) * 240}, 70%, 50%)`,
      line: { width: 0 },
      layer: 'above' as const,
    };
  });

  // Create hover trace (invisible scatter points at block centers for tooltips)
  const hoverData = data.blocks.map((block) => {
    const startDate = mjdToDate(block.scheduled_start_mjd);
    const stopDate = mjdToDate(block.scheduled_stop_mjd);
    const monthKey = `${startDate.getFullYear()}-${String(startDate.getMonth() + 1).padStart(2, '0')}`;
    const monthIndex = monthMap.get(monthKey) ?? 0;
    
    const startDay = startDate.getDate() + startDate.getHours() / 24 + startDate.getMinutes() / 1440;
    const stopDay = stopDate.getDate() + stopDate.getHours() / 24 + stopDate.getMinutes() / 1440;
    const centerDay = (startDay + stopDay) / 2;
    
    return {
      x: centerDay,
      y: monthIndex,
      text: `${block.original_block_id}<br>Start: ${startDate.toISOString()}<br>Priority: ${block.priority.toFixed(1)}<br>Duration: ${block.requested_hours.toFixed(1)}h`,
    };
  });

  const plotData: Plotly.Data[] = [{
    type: 'scatter',
    mode: 'markers',
    x: hoverData.map(d => d.x),
    y: hoverData.map(d => d.y),
    text: hoverData.map(d => d.text),
    hoverinfo: 'text',
    marker: {
      size: 20,
      opacity: 0, // Invisible but still capturable for hover
    },
    showlegend: false,
  }];

  // Final layout with shapes
  const timelineLayout = {
    ...layout,
    xaxis: {
      ...layout.xaxis,
      title: { text: 'Day of Month' },
      range: [0, 32],
      dtick: 1,
    },
    yaxis: {
      ...layout.yaxis,
      title: { text: 'Month' },
      tickmode: 'array' as const,
      tickvals: data.unique_months.map((_, index) => index),
      ticktext: data.unique_months,
      autorange: 'reversed' as const,
    },
    shapes,
  };

  return (
    <PageContainer>
      {/* Header */}
      <PageHeader
        title="Timeline"
        description="Scheduled observations over time"
      />

      {/* Metrics */}
      <MetricsGrid>
        <MetricCard label="Total Blocks" value={data.total_count} icon="📅" />
        <MetricCard label="Scheduled" value={data.scheduled_count} icon="✅" />
        <MetricCard label="Unique Months" value={data.unique_months.length} icon="📆" />
        <MetricCard label="Dark Periods" value={data.dark_periods.length} icon="🌙" />
      </MetricsGrid>

      {/* Timeline chart */}
      <ChartPanel title="Schedule Timeline">
        <PlotlyChart data={plotData} layout={timelineLayout} config={config} height="800px" />
      </ChartPanel>

    </PageContainer>
  );
}

export default Timeline;
