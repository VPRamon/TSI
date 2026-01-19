/**
 * Visibility Map page - Histogram of target visibility over observation period.
 */
import { useState, useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { useVisibilityMap, useVisibilityHistogram } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard, PlotlyChart } from '@/components';
import { usePlotlyTheme } from '@/hooks/usePlotlyTheme';

function VisibilityMap() {
  const { scheduleId } = useParams();
  const currentId = parseInt(scheduleId ?? '0', 10);

  // State for filters
  const [numBins, setNumBins] = useState(50);
  const [binDurationMinutes, setBinDurationMinutes] = useState<number | undefined>(undefined);
  const [priorityMin, setPriorityMin] = useState<number | undefined>(undefined);
  const [priorityMax, setPriorityMax] = useState<number | undefined>(undefined);
  const [useCustomDuration, setUseCustomDuration] = useState(false);

  // Fetch visibility map data for metadata
  const { data: mapData, isLoading: mapLoading, error: mapError } = useVisibilityMap(currentId);

  // Build query for histogram
  const histogramQuery = useMemo(() => {
    const query: {
      num_bins?: number;
      bin_duration_minutes?: number;
      priority_min?: number;
      priority_max?: number;
    } = {};

    if (useCustomDuration && binDurationMinutes) {
      query.bin_duration_minutes = binDurationMinutes;
    } else {
      query.num_bins = numBins;
    }

    if (priorityMin !== undefined) {
      query.priority_min = Math.floor(priorityMin);
    }
    if (priorityMax !== undefined) {
      query.priority_max = Math.ceil(priorityMax);
    }

    return query;
  }, [numBins, binDurationMinutes, priorityMin, priorityMax, useCustomDuration]);

  // Fetch histogram data
  const {
    data: histogramData,
    isLoading: histogramLoading,
    error: histogramError,
    refetch,
  } = useVisibilityHistogram(currentId, histogramQuery);

  const plotlyTheme = usePlotlyTheme();

  const isLoading = mapLoading || histogramLoading;
  const error = mapError || histogramError;

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
        title="Failed to load visibility map"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }

  if (!mapData || !histogramData) {
    return <ErrorMessage message="No data available" />;
  }

  // Calculate filtered block count
  const effectivePriorityMin = priorityMin ?? mapData.priority_min;
  const effectivePriorityMax = priorityMax ?? mapData.priority_max;
  const filteredBlocks = mapData.blocks.filter(
    (b) => b.priority >= effectivePriorityMin && b.priority <= effectivePriorityMax
  );

  // Prepare histogram data for Plotly
  const binStarts = histogramData.map((bin) => new Date(bin.bin_start_unix * 1000));
  const binCounts = histogramData.map((bin) => bin.visible_count);
  const binWidths = histogramData.map(
    (bin) => (bin.bin_end_unix - bin.bin_start_unix) * 1000 // milliseconds
  );

  // Calculate bin duration for display
  const avgBinDuration =
    histogramData.length > 0
      ? (histogramData[0].bin_end_unix - histogramData[0].bin_start_unix) / 60
      : 0;

  const durationLabel =
    avgBinDuration >= 24 * 60
      ? `${(avgBinDuration / (24 * 60)).toFixed(1)} day(s)`
      : avgBinDuration >= 60
        ? `${(avgBinDuration / 60).toFixed(1)} hour(s)`
        : `${avgBinDuration.toFixed(1)} minute(s)`;

  const histogramTrace = {
    x: binStarts,
    y: binCounts,
    width: binWidths,
    type: 'bar' as const,
    name: 'Visible Targets',
    marker: {
      color: binCounts,
      colorscale: 'Viridis',
      colorbar: { title: 'Number of<br>Visible Blocks' },
      line: { width: 0.5, color: 'rgba(255, 255, 255, 0.15)' },
    },
    hovertemplate: '<b>%{y} visible blocks</b><br>Time: %{x|%Y-%m-%d %H:%M}<br><extra></extra>',
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Visibility Map</h1>
        <p className="mt-1 text-slate-400">
          Target visibility over the observation period ({filteredBlocks.length} blocks,{' '}
          {histogramData.length} bins, ~{durationLabel} per bin)
        </p>
      </div>

      {/* Statistics */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <MetricCard label="Total Blocks" value={mapData.total_count} icon="📊" />
        <MetricCard label="Filtered Blocks" value={filteredBlocks.length} icon="🔍" />
        <MetricCard label="Scheduled" value={mapData.scheduled_count} icon="✅" />
        <MetricCard
          label="Priority Range"
          value={`${mapData.priority_min.toFixed(1)} - ${mapData.priority_max.toFixed(1)}`}
          icon="⭐"
        />
      </div>

      {/* Filter Controls */}
      <Card title="Filters">
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          {/* Binning method toggle */}
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">
              Binning Method
            </label>
            <select
              className="w-full rounded-lg border border-slate-600 bg-slate-700 px-3 py-2 text-white focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
              value={useCustomDuration ? 'duration' : 'bins'}
              onChange={(e) => setUseCustomDuration(e.target.value === 'duration')}
            >
              <option value="bins">Number of Bins</option>
              <option value="duration">Bin Duration</option>
            </select>
          </div>

          {/* Number of bins OR bin duration */}
          {!useCustomDuration ? (
            <div>
              <label className="mb-2 block text-sm font-medium text-slate-300">
                Number of Bins: {numBins}
              </label>
              <input
                type="range"
                min="10"
                max="200"
                value={numBins}
                onChange={(e) => setNumBins(parseInt(e.target.value, 10))}
                className="w-full"
              />
            </div>
          ) : (
            <div>
              <label className="mb-2 block text-sm font-medium text-slate-300">
                Bin Duration (minutes)
              </label>
              <input
                type="number"
                min="1"
                max="10080"
                value={binDurationMinutes ?? 60}
                onChange={(e) =>
                  setBinDurationMinutes(
                    e.target.value ? parseInt(e.target.value, 10) : undefined
                  )
                }
                className="w-full rounded-lg border border-slate-600 bg-slate-700 px-3 py-2 text-white focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
            </div>
          )}

          {/* Priority min */}
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">
              Min Priority
            </label>
            <input
              type="number"
              min={mapData.priority_min}
              max={mapData.priority_max}
              step="0.1"
              value={priorityMin ?? mapData.priority_min}
              onChange={(e) =>
                setPriorityMin(e.target.value ? parseFloat(e.target.value) : undefined)
              }
              className="w-full rounded-lg border border-slate-600 bg-slate-700 px-3 py-2 text-white focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>

          {/* Priority max */}
          <div>
            <label className="mb-2 block text-sm font-medium text-slate-300">
              Max Priority
            </label>
            <input
              type="number"
              min={mapData.priority_min}
              max={mapData.priority_max}
              step="0.1"
              value={priorityMax ?? mapData.priority_max}
              onChange={(e) =>
                setPriorityMax(e.target.value ? parseFloat(e.target.value) : undefined)
              }
              className="w-full rounded-lg border border-slate-600 bg-slate-700 px-3 py-2 text-white focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>

          {/* Reset button */}
          <div className="flex items-end">
            <button
              onClick={() => {
                setNumBins(50);
                setBinDurationMinutes(undefined);
                setPriorityMin(undefined);
                setPriorityMax(undefined);
                setUseCustomDuration(false);
              }}
              className="w-full rounded-lg bg-slate-600 px-4 py-2 text-white transition-colors hover:bg-slate-500"
            >
              Reset Filters
            </button>
          </div>
        </div>
      </Card>

      {/* Histogram */}
      <Card title="Visibility Histogram">
        <PlotlyChart
          data={[histogramTrace]}
          layout={{
            ...plotlyTheme,
            xaxis: {
              title: { text: 'Observation Period (UTC)' },
              showgrid: true,
              gridcolor: 'rgba(100, 100, 100, 0.3)',
              type: 'date',
            },
            yaxis: {
              title: { text: 'Number of Visible Blocks' },
              showgrid: true,
              gridcolor: 'rgba(100, 100, 100, 0.3)',
            },
            bargap: 0,
            height: 600,
            hovermode: 'x unified',
            showlegend: false,
          }}
          config={{ displayModeBar: true, responsive: true }}
        />
      </Card>
    </div>
  );
}

export default VisibilityMap;
