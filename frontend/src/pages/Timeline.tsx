/**
 * Timeline page - Scheduled observations over time.
 */
import { useParams } from 'react-router-dom';
import Plot from 'react-plotly.js';
import { useTimeline } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard } from '@/components';

// Convert MJD to JavaScript Date
function mjdToDate(mjd: number): Date {
  const MJD_EPOCH = Date.UTC(1858, 10, 17); // November 17, 1858
  return new Date(MJD_EPOCH + mjd * 86400000);
}

function Timeline() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useTimeline(id);

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

  // Add dark periods as background shapes
  const shapes: Plotly.Shape[] = data.dark_periods.map((period) => ({
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

  const layout: Partial<Plotly.Layout> = {
    title: {
      text: 'Observation Timeline',
      font: { color: '#fff' },
    },
    paper_bgcolor: 'transparent',
    plot_bgcolor: '#1e293b',
    font: { color: '#94a3b8' },
    xaxis: {
      title: 'Date',
      type: 'date',
      gridcolor: '#334155',
    },
    yaxis: {
      title: 'Observation',
      showticklabels: false,
      gridcolor: '#334155',
    },
    shapes,
    margin: { t: 50, r: 20, b: 60, l: 60 },
    hovermode: 'closest',
  };

  const config: Partial<Plotly.Config> = {
    responsive: true,
    displayModeBar: true,
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Timeline</h1>
        <p className="text-slate-400 mt-1">
          Scheduled observations over time
        </p>
      </div>

      {/* Metrics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <MetricCard label="Total Blocks" value={data.total_count} icon="📅" />
        <MetricCard label="Scheduled" value={data.scheduled_count} icon="✅" />
        <MetricCard
          label="Unique Months"
          value={data.unique_months.length}
          icon="📆"
        />
        <MetricCard
          label="Dark Periods"
          value={data.dark_periods.length}
          icon="🌙"
        />
      </div>

      {/* Timeline chart */}
      <Card title="Schedule Timeline">
        <div className="h-[600px]">
          <Plot
            data={plotData}
            layout={layout}
            config={config}
            style={{ width: '100%', height: '100%' }}
            useResizeHandler
          />
        </div>
      </Card>

      {/* Months list */}
      <Card title="Covered Months">
        <div className="flex flex-wrap gap-2">
          {data.unique_months.map((month) => (
            <span
              key={month}
              className="px-3 py-1 bg-slate-700 rounded-full text-sm text-slate-300"
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
