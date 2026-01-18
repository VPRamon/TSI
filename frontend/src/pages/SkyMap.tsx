/**
 * Sky Map page - Celestial coordinate visualization.
 */
import { useParams, useNavigate } from 'react-router-dom';
import { useSkyMap, usePlotlyTheme } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard, PlotlyChart } from '@/components';
import { STATUS_COLORS } from '@/constants/colors';

function SkyMap() {
  const { scheduleId } = useParams();
  const navigate = useNavigate();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useSkyMap(id);

  // Call hook unconditionally (rules of hooks)
  const { layout, config } = usePlotlyTheme({
    title: 'Sky Map - Observation Targets',
    xAxis: { title: 'Right Ascension (degrees)', range: [0, 360] },
    yAxis: { title: 'Declination (degrees)', range: [-90, 90] },
    configPreset: 'skymap',
  });

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    const errorMessage = (error as Error).message;
    const isNotFound = errorMessage.includes('not found') || errorMessage.includes('Not found');

    return (
      <div className="flex h-full items-center justify-center p-4">
        <div className="max-w-md">
          <ErrorMessage
            title={isNotFound ? 'Schedule Not Found' : 'Failed to load sky map'}
            message={
              isNotFound
                ? `Schedule ${id} does not exist. It may have been deleted or the server may have restarted.`
                : errorMessage
            }
            onRetry={isNotFound ? undefined : () => refetch()}
          />
          <button
            onClick={() => navigate('/')}
            className="mt-4 w-full rounded-lg bg-blue-600 px-4 py-2 text-white transition-colors hover:bg-blue-700"
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
        color: STATUS_COLORS.scheduled,
        opacity: 0.7,
      },
      text: scheduled.map(
        (b) => `ID: ${b.original_block_id}<br>Priority: ${b.priority.toFixed(1)}`
      ),
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
        color: STATUS_COLORS.unscheduled,
        opacity: 0.5,
      },
      text: unscheduled.map(
        (b) => `ID: ${b.original_block_id}<br>Priority: ${b.priority.toFixed(1)}`
      ),
      hoverinfo: 'text',
    },
  ];

  const schedulingRate =
    data.total_count > 0 ? ((data.scheduled_count / data.total_count) * 100).toFixed(1) : '0';

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Sky Map</h1>
        <p className="mt-1 text-slate-400">
          Visualization of observation targets in celestial coordinates
        </p>
      </div>

      {/* Metrics */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <MetricCard label="Total Blocks" value={data.total_count} icon="🎯" />
        <MetricCard label="Scheduled" value={data.scheduled_count} icon="✅" />
        <MetricCard label="Scheduling Rate" value={`${schedulingRate}%`} icon="📊" />
        <MetricCard
          label="Priority Range"
          value={`${data.priority_min.toFixed(1)} - ${data.priority_max.toFixed(1)}`}
          icon="⭐"
        />
      </div>

      {/* Plot */}
      <Card title="Celestial Coordinates">
        <PlotlyChart data={plotData} layout={layout} config={config} height="500px" />
      </Card>

      {/* Priority bins legend */}
      <Card title="Priority Bins">
        <div className="flex flex-wrap gap-4">
          {data.priority_bins.map((bin) => (
            <div key={bin.label} className="flex items-center gap-2">
              <div className="h-4 w-4 rounded" style={{ backgroundColor: bin.color }} />
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
