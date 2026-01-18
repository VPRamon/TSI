/**
 * Sky Map page - Celestial coordinate visualization.
 */
import { useParams, useNavigate } from 'react-router-dom';
import Plot from 'react-plotly.js';
import { useSkyMap } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard } from '@/components';

function SkyMap() {
  const { scheduleId } = useParams();
  const navigate = useNavigate();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useSkyMap(id);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    const errorMessage = (error as Error).message;
    const isNotFound = errorMessage.includes('not found') || errorMessage.includes('Not found');
    
    return (
      <div className="flex items-center justify-center h-full p-4">
        <div className="max-w-md">
          <ErrorMessage
            title={isNotFound ? 'Schedule Not Found' : 'Failed to load sky map'}
            message={isNotFound 
              ? `Schedule ${id} does not exist. It may have been deleted or the server may have restarted.`
              : errorMessage
            }
            onRetry={isNotFound ? undefined : () => refetch()}
          />
          <button
            onClick={() => navigate('/')}
            className="mt-4 w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
          >
            Return to Schedule List
          </button>
        </div>
      </div>
    );
  }

  if (!data) {
    return <ErrorMessage message="No data available" />;
  }

  // Prepare data for Plotly scatter plot
  const scheduled = data.blocks.filter((b) => b.scheduled_period !== null);
  const unscheduled = data.blocks.filter((b) => b.scheduled_period === null);

  const plotData: Plotly.Data[] = [
    {
      type: 'scattergl',
      mode: 'markers',
      name: 'Scheduled',
      x: scheduled.map((b) => b.target_ra_deg),
      y: scheduled.map((b) => b.target_dec_deg),
      marker: {
        size: 8,
        color: '#22c55e',
        opacity: 0.7,
      },
      text: scheduled.map((b) => `ID: ${b.original_block_id}<br>Priority: ${b.priority.toFixed(1)}`),
      hoverinfo: 'text',
    },
    {
      type: 'scattergl',
      mode: 'markers',
      name: 'Unscheduled',
      x: unscheduled.map((b) => b.target_ra_deg),
      y: unscheduled.map((b) => b.target_dec_deg),
      marker: {
        size: 6,
        color: '#ef4444',
        opacity: 0.5,
      },
      text: unscheduled.map((b) => `ID: ${b.original_block_id}<br>Priority: ${b.priority.toFixed(1)}`),
      hoverinfo: 'text',
    },
  ];

  const layout: Partial<Plotly.Layout> = {
    title: {
      text: 'Sky Map - Observation Targets',
      font: { color: '#fff' },
    },
    paper_bgcolor: 'transparent',
    plot_bgcolor: '#1e293b',
    font: { color: '#94a3b8' },
    xaxis: {
      title: { text: 'Right Ascension (degrees)' },
      range: [0, 360],
      gridcolor: '#334155',
      zerolinecolor: '#475569',
    },
    yaxis: {
      title: { text: 'Declination (degrees)' },
      range: [-90, 90],
      gridcolor: '#334155',
      zerolinecolor: '#475569',
    },
    legend: {
      orientation: 'h',
      y: -0.15,
      font: { color: '#94a3b8' },
    },
    margin: { t: 50, r: 20, b: 60, l: 60 },
  };

  const config: Partial<Plotly.Config> = {
    responsive: true,
    displayModeBar: true,
    modeBarButtonsToRemove: ['lasso2d', 'select2d'],
  };

  const schedulingRate = data.total_count > 0
    ? ((data.scheduled_count / data.total_count) * 100).toFixed(1)
    : '0';

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Sky Map</h1>
        <p className="text-slate-400 mt-1">
          Visualization of observation targets in celestial coordinates
        </p>
      </div>

      {/* Metrics */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <MetricCard label="Total Blocks" value={data.total_count} icon="🎯" />
        <MetricCard label="Scheduled" value={data.scheduled_count} icon="✅" />
        <MetricCard
          label="Scheduling Rate"
          value={`${schedulingRate}%`}
          icon="📊"
        />
        <MetricCard
          label="Priority Range"
          value={`${data.priority_min.toFixed(1)} - ${data.priority_max.toFixed(1)}`}
          icon="⭐"
        />
      </div>

      {/* Plot */}
      <Card title="Celestial Coordinates">
        <div className="h-[500px]">
          <Plot
            data={plotData}
            layout={layout}
            config={config}
            style={{ width: '100%', height: '100%' }}
            useResizeHandler
          />
        </div>
      </Card>

      {/* Priority bins legend */}
      <Card title="Priority Bins">
        <div className="flex flex-wrap gap-4">
          {data.priority_bins.map((bin) => (
            <div key={bin.label} className="flex items-center gap-2">
              <div
                className="w-4 h-4 rounded"
                style={{ backgroundColor: bin.color }}
              />
              <span className="text-sm text-slate-300">
                {bin.label}: {bin.min_priority.toFixed(1)} - {bin.max_priority.toFixed(1)}
              </span>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}

export default SkyMap;
