/**
 * Sky Map page - Celestial coordinate visualization.
 * Redesigned with a settings panel for filtering and interactive priority legend.
 */
import { useState, useMemo, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useSkyMap, usePlotlyTheme } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  MetricCard,
  PlotlyChart,
  PageHeader,
  PageContainer,
  MetricsGrid,
  ChartPanel,
  SkyMapFilters,
} from '@/components';
import type { SkyMapFilterState } from '@/components';
import type { LightweightBlock, PriorityBinInfo } from '@/api/types';

// MJD (Modified Julian Date) conversion utilities
const MJD_EPOCH = 2400000.5; // MJD epoch in JD
const UNIX_EPOCH_JD = 2440587.5; // Unix epoch (1970-01-01 00:00:00 UTC) in JD

/** Convert MJD to UTC ISO string */
function mjdToUtc(mjd: number): string {
  const jd = mjd + MJD_EPOCH;
  const unixMs = (jd - UNIX_EPOCH_JD) * 86400000;
  return new Date(unixMs).toISOString();
}

/** Convert UTC ISO string to MJD */
function utcToMjd(utcString: string): number {
  if (!utcString) return 0;
  const unixMs = new Date(utcString).getTime();
  const jd = unixMs / 86400000 + UNIX_EPOCH_JD;
  return jd - MJD_EPOCH;
}

/** Format UTC string for datetime-local input (YYYY-MM-DDTHH:mm) */
function toDatetimeLocal(utcIso: string): string {
  if (!utcIso) return '';
  return utcIso.slice(0, 16); // Remove seconds and timezone
}

/** Create default filter state from data bounds */
function createDefaultFilters(
  priorityMin: number,
  priorityMax: number,
  scheduledTimeMin: number | null,
  scheduledTimeMax: number | null
): SkyMapFilterState {
  return {
    showScheduled: true,
    showUnscheduled: true,
    scheduledBeginUtc: scheduledTimeMin ? toDatetimeLocal(mjdToUtc(scheduledTimeMin)) : '',
    scheduledEndUtc: scheduledTimeMax ? toDatetimeLocal(mjdToUtc(scheduledTimeMax)) : '',
    priorityMin,
    priorityMax,
  };
}

/** Apply filters to blocks */
function filterBlocks(
  blocks: LightweightBlock[],
  filters: SkyMapFilterState
): LightweightBlock[] {
  return blocks.filter((block) => {
    // Status filter
    const isScheduled = block.scheduled_period !== null;
    if (isScheduled && !filters.showScheduled) return false;
    if (!isScheduled && !filters.showUnscheduled) return false;

    // Priority filter
    if (block.priority < filters.priorityMin || block.priority > filters.priorityMax) {
      return false;
    }

    // Scheduled period filter (only applies to scheduled blocks)
    if (isScheduled && block.scheduled_period && filters.scheduledBeginUtc && filters.scheduledEndUtc) {
      const { start, stop } = block.scheduled_period;
      const filterBeginMjd = utcToMjd(filters.scheduledBeginUtc);
      const filterEndMjd = utcToMjd(filters.scheduledEndUtc);
      
      // Block overlaps with filter range if: block_stop >= filter_begin AND block_start <= filter_end
      if (stop < filterBeginMjd || start > filterEndMjd) {
        return false;
      }
    }

    return true;
  });
}

/** Group blocks by priority bin */
function groupByPriorityBin(
  blocks: LightweightBlock[],
  bins: PriorityBinInfo[]
): Map<string, LightweightBlock[]> {
  const grouped = new Map<string, LightweightBlock[]>();
  
  // Initialize all bins
  bins.forEach(bin => grouped.set(bin.label, []));
  
  // Group blocks
  blocks.forEach(block => {
    const bin = bins.find(b => 
      block.priority >= b.min_priority && block.priority <= b.max_priority
    );
    if (bin) {
      grouped.get(bin.label)!.push(block);
    }
  });
  
  return grouped;
}

function SkyMap() {
  const { scheduleId } = useParams();
  const navigate = useNavigate();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useSkyMap(id);

  // Filter state - initialized once data is available
  const [filters, setFilters] = useState<SkyMapFilterState | null>(null);

  // Initialize filters when data loads
  const activeFilters = useMemo(() => {
    if (!data) return null;
    if (filters) return filters;
    return createDefaultFilters(
      data.priority_min, 
      data.priority_max,
      data.scheduled_time_min,
      data.scheduled_time_max
    );
  }, [data, filters]);

  // Memoized filter application
  const filteredBlocks = useMemo(() => {
    if (!data || !activeFilters) return { all: [], scheduled: [], unscheduled: [] };
    const filtered = filterBlocks(data.blocks, activeFilters);
    return {
      all: filtered,
      scheduled: filtered.filter((b) => b.scheduled_period !== null),
      unscheduled: filtered.filter((b) => b.scheduled_period === null),
    };
  }, [data, activeFilters]);

  // Reset handler
  const handleReset = useCallback(() => {
    if (data) {
      setFilters(createDefaultFilters(
        data.priority_min, 
        data.priority_max,
        data.scheduled_time_min,
        data.scheduled_time_max
      ));
    }
  }, [data]);

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
            className="mt-4 w-full rounded-lg bg-primary-600 px-4 py-2 text-white transition-colors hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-900"
          >
            Return to Schedule List
          </button>
        </div>
      </div>
    );
  }

  if (!data || !activeFilters) {
    return <ErrorMessage message="No data available" />;
  }

  const { all: allFiltered, scheduled, unscheduled } = filteredBlocks;

  // Group scheduled and unscheduled blocks by priority bin
  const scheduledByBin = groupByPriorityBin(scheduled, data.priority_bins);
  const unscheduledByBin = groupByPriorityBin(unscheduled, data.priority_bins);

  // Create Plotly traces - one for each priority bin, split by scheduled/unscheduled
  const plotData: Plotly.Data[] = [];

  // Add scheduled traces (one per bin)
  data.priority_bins.forEach((bin) => {
    const blocks = scheduledByBin.get(bin.label) || [];
    if (blocks.length > 0 || activeFilters.showScheduled) {
      plotData.push({
        type: 'scattergl',
        mode: 'markers',
        name: `${bin.label} (Scheduled)`,
        legendgroup: bin.label,
        x: blocks.map((b) => b.target_ra_deg),
        y: blocks.map((b) => b.target_dec_deg),
        marker: {
          size: 8,
          color: bin.color,
          opacity: 0.8,
          line: { width: 1, color: '#ffffff' },
        },
        text: blocks.map(
          (b) => `ID: ${b.original_block_id}<br>Priority: ${b.priority.toFixed(1)}<br>Bin: ${bin.label}<br>Status: Scheduled`
        ),
        hoverinfo: 'text',
      });
    }
  });

  // Add unscheduled traces (one per bin, with different marker style)
  data.priority_bins.forEach((bin) => {
    const blocks = unscheduledByBin.get(bin.label) || [];
    if (blocks.length > 0 || activeFilters.showUnscheduled) {
      plotData.push({
        type: 'scattergl',
        mode: 'markers',
        name: `${bin.label} (Unscheduled)`,
        legendgroup: bin.label,
        x: blocks.map((b) => b.target_ra_deg),
        y: blocks.map((b) => b.target_dec_deg),
        marker: {
          size: 6,
          color: bin.color,
          opacity: 0.4,
          symbol: 'circle-open',
        },
        text: blocks.map(
          (b) => `ID: ${b.original_block_id}<br>Priority: ${b.priority.toFixed(1)}<br>Bin: ${bin.label}<br>Status: Unscheduled`
        ),
        hoverinfo: 'text',
      });
    }
  });

  // Compute metrics for filtered data
  const totalFiltered = allFiltered.length;
  const schedulingRate =
    totalFiltered > 0 ? ((scheduled.length / totalFiltered) * 100).toFixed(1) : '0';

  // Convert MJD times to UTC for display in filter
  const scheduledTimeRange = {
    min: data.scheduled_time_min ? mjdToUtc(data.scheduled_time_min) : null,
    max: data.scheduled_time_max ? mjdToUtc(data.scheduled_time_max) : null,
  };

  return (
    <PageContainer>
      {/* Header */}
      <PageHeader
        title="Sky Map"
        description="Visualization of observation targets in celestial coordinates"
      />

      {/* Metrics */}
      <MetricsGrid>
        <MetricCard
          label="Displayed Blocks"
          value={`${totalFiltered} / ${data.total_count}`}
          icon="🎯"
        />
        <MetricCard label="Scheduled" value={scheduled.length} icon="✅" />
        <MetricCard label="Scheduling Rate" value={`${schedulingRate}%`} icon="📊" />
        <MetricCard
          label="Priority Range"
          value={`${data.priority_min.toFixed(1)} - ${data.priority_max.toFixed(1)}`}
          icon="⭐"
        />
      </MetricsGrid>

      {/* Main content: Filters sidebar + Chart */}
      <div className="grid gap-4 lg:grid-cols-[280px_1fr]">
        {/* Filters Panel */}
        <div className="order-2 lg:order-1">
          <SkyMapFilters
            filters={activeFilters}
            onChange={setFilters}
            scheduledTimeRange={scheduledTimeRange}
            priorityRange={{
              min: data.priority_min,
              max: data.priority_max,
            }}
            onReset={handleReset}
          />
        </div>

        {/* Chart */}
        <div className="order-1 flex flex-col gap-4 lg:order-2">
          <ChartPanel title="Celestial Coordinates">
            <PlotlyChart data={plotData} layout={layout} config={config} height="500px" />
          </ChartPanel>
        </div>
      </div>
    </PageContainer>
  );
}

export default SkyMap;
