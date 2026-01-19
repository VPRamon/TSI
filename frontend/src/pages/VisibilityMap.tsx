/**
 * Visibility Map page - Histogram of target visibility over observation period.
 * Redesigned with SplitPane layout: controls on left, chart on right (desktop).
 */
import { useState, useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { useVisibilityMap, useVisibilityHistogram } from '@/hooks';
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

  const handleReset = () => {
    setNumBins(50);
    setBinDurationMinutes(undefined);
    setPriorityMin(undefined);
    setPriorityMax(undefined);
    setUseCustomDuration(false);
  };

  // Controls panel content
  const controlsContent = (
    <div className="space-y-5">
      <h3 className="text-sm font-medium text-slate-200">Filters</h3>

      {/* Binning method toggle */}
      <div>
        <label className="mb-1.5 block text-xs font-medium text-slate-400">
          Binning Method
        </label>
        <select
          className="w-full rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
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
          <label className="mb-1.5 block text-xs font-medium text-slate-400">
            Number of Bins
          </label>
          <div className="flex items-center gap-3">
            <input
              type="range"
              min="10"
              max="200"
              value={numBins}
              onChange={(e) => setNumBins(parseInt(e.target.value, 10))}
              className="h-2 flex-1 cursor-pointer appearance-none rounded-lg bg-slate-600"
            />
            <span className="w-10 text-right text-sm font-medium text-white">{numBins}</span>
          </div>
        </div>
      ) : (
        <div>
          <label className="mb-1.5 block text-xs font-medium text-slate-400">
            Bin Duration (min)
          </label>
          <input
            type="number"
            min="1"
            max="10080"
            value={binDurationMinutes ?? 60}
            onChange={(e) =>
              setBinDurationMinutes(e.target.value ? parseInt(e.target.value, 10) : undefined)
            }
            className="w-full rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
        </div>
      )}

      <div className="border-t border-slate-700 pt-4">
        <h4 className="mb-3 text-xs font-medium uppercase tracking-wide text-slate-500">
          Priority Range
        </h4>

        {/* Priority min */}
        <div className="mb-3">
          <label className="mb-1.5 block text-xs font-medium text-slate-400">
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
            className="w-full rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
        </div>

        {/* Priority max */}
        <div>
          <label className="mb-1.5 block text-xs font-medium text-slate-400">
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
            className="w-full rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
        </div>
      </div>

      {/* Reset button */}
      <button
        onClick={handleReset}
        className="w-full rounded-md bg-slate-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-slate-500 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-800"
      >
        Reset Filters
      </button>

      {/* Current settings summary */}
      <div className="rounded-md bg-slate-700/50 p-3">
        <p className="text-xs text-slate-400">Current Settings</p>
        <p className="mt-1 text-sm text-slate-200">
          {histogramData.length} bins • ~{durationLabel}/bin
        </p>
        <p className="mt-0.5 text-sm text-slate-200">
          Priority: {effectivePriorityMin.toFixed(1)} – {effectivePriorityMax.toFixed(1)}
        </p>
      </div>
    </div>
  );

  return (
    <PageContainer>
      {/* Header */}
      <PageHeader
        title="Visibility Map"
        description={`Target visibility over the observation period (${filteredBlocks.length} blocks)`}
      />

      {/* Metrics */}
      <MetricsGrid>
        <MetricCard label="Total Blocks" value={mapData.total_count} icon="📊" />
        <MetricCard label="Filtered Blocks" value={filteredBlocks.length} icon="🔍" />
        <MetricCard label="Scheduled" value={mapData.scheduled_count} icon="✅" />
        <MetricCard
          label="Priority Range"
          value={`${mapData.priority_min.toFixed(1)} - ${mapData.priority_max.toFixed(1)}`}
          icon="⭐"
        />
      </MetricsGrid>

      {/* Split layout: controls left, chart right */}
      <SplitPane controls={controlsContent} controlsWidth="sm">
        <ChartPanel title="Visibility Histogram">
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
              height: 550,
              hovermode: 'x unified',
              showlegend: false,
            }}
            config={{ displayModeBar: true, responsive: true }}
          />
        </ChartPanel>
      </SplitPane>
    </PageContainer>
  );
}

export default VisibilityMap;
