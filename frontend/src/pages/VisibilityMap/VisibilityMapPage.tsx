/**
 * VisibilityMap page - Histogram of target visibility + block-level drill-down.
 *
 * Reworked for astrophysicist schedule analysis:
 * - Visibility histogram: when are targets observable?
 * - Block table: drill down to individual blocks, filter/sort/select
 * - Details drawer: full block info, add to selection for cross-view tracking
 * - Integrated with AnalysisContext for consistent filtering across views
 * - URL sync for shareable analysis states
 *
 * ARCHITECTURE:
 * - Uses shared VisibilityMapData/VisibilityBin types from api/types.ts
 * - FilterSettings for histogram controls
 * - BlocksTable + BlockDetailsDrawer for drill-down
 * - AnalysisContext for cross-view filter/selection state
 */
import { useState, useCallback, useMemo, memo, useEffect, useRef } from 'react';
import { useParams } from 'react-router-dom';
import { useVisibilityMap, useVisibilityHistogram } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  Icon,
  PageContainer,
  PageHeader,
  MetricsGrid,
  MetricCard,
} from '@/components';
import {
  OpportunitiesHistogram,
  BlocksTable,
  BlockDetailsDrawer,
  ExportMenu,
  useAnalysis,
  type FilterParams,
  type TableBlock,
} from '@/features/schedules';
import type { VisibilityMapData, VisibilityBin, VisibilityHistogramQuery } from '@/api/types';

// =============================================================================
// Types
// =============================================================================

// Extend visibility block summary to match TableBlock interface
interface VisibilityBlock extends TableBlock {
  num_visibility_periods: number;
}

// =============================================================================
// Default Values
// =============================================================================

const DEFAULT_FILTERS: FilterParams = {
  numBins: 50,
  binDurationMinutes: undefined,
  priorityMin: undefined,
  priorityMax: undefined,
  useCustomDuration: false,
};

const FILTER_DEBOUNCE_MS = 150;
const PRIORITY_SLIDER_CLASS =
  'pointer-events-none absolute inset-x-0 top-1/2 h-6 w-full -translate-y-1/2 appearance-none bg-transparent focus:outline-none [&::-moz-range-thumb]:pointer-events-auto [&::-moz-range-thumb]:h-4 [&::-moz-range-thumb]:w-4 [&::-moz-range-thumb]:cursor-pointer [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-2 [&::-moz-range-thumb]:border-slate-950 [&::-moz-range-thumb]:bg-primary-400 [&::-moz-range-track]:h-2 [&::-moz-range-track]:rounded-full [&::-moz-range-track]:border-0 [&::-moz-range-track]:bg-transparent [&::-webkit-slider-runnable-track]:h-2 [&::-webkit-slider-runnable-track]:rounded-full [&::-webkit-slider-runnable-track]:bg-transparent [&::-webkit-slider-thumb]:pointer-events-auto [&::-webkit-slider-thumb]:mt-[-4px] [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:border-2 [&::-webkit-slider-thumb]:border-slate-950 [&::-webkit-slider-thumb]:bg-primary-400';

// =============================================================================
// Sub-Components
// =============================================================================

interface SummaryMetricsProps {
  mapData: VisibilityMapData;
  filteredCount: number;
  selectionCount: number;
}

const SummaryMetrics = memo(function SummaryMetrics({
  mapData,
  filteredCount,
  selectionCount,
}: SummaryMetricsProps) {
  const schedulingRate =
    mapData.total_count > 0
      ? ((mapData.scheduled_count / mapData.total_count) * 100).toFixed(1)
      : '0';
  const filteredOutCount = Math.max(mapData.total_count - filteredCount, 0);

  return (
    <>
      <PageHeader
        title="Visibility Analysis"
        description="Target visibility over the observation period with block-level drill-down"
      />
      <MetricsGrid>
        <MetricCard
          label="Total Blocks"
          value={mapData.total_count}
          icon={<Icon name="chart-bar" />}
        />
        <MetricCard
          label="Scheduled"
          value={`${mapData.scheduled_count} (${schedulingRate}%)`}
          icon={<Icon name="check-circle" />}
        />
        <MetricCard
          label="Priority Range"
          value={`${mapData.priority_min.toFixed(1)} – ${mapData.priority_max.toFixed(1)}`}
          icon={<Icon name="star" />}
        />
        <MetricCard label="Filtered" value={filteredOutCount} icon={<Icon name="search" />} />
        {selectionCount > 0 && (
          <MetricCard label="Selected" value={selectionCount} icon={<Icon name="check" />} />
        )}
      </MetricsGrid>
    </>
  );
});

// =============================================================================
// Main Page Content
// =============================================================================

interface VisibilityMapContentProps {
  mapData: VisibilityMapData;
  histogramData: VisibilityBin[] | undefined;
  histogramLoading: boolean;
  filters: FilterParams;
  onFiltersChange: (params: FilterParams) => void;
}

const VisibilityMapContent = memo(function VisibilityMapContent({
  mapData,
  histogramData,
  histogramLoading,
  filters,
  onFiltersChange,
}: VisibilityMapContentProps) {
  const {
    state,
    setActiveBlock,
    selectionCount,
    setScheduledFilter,
    setPriorityFilter,
  } = useAnalysis();
  const [activeBlock, setActiveBlockLocal] = useState<VisibilityBlock | null>(null);

  // Sync local priority filter → AnalysisContext so table and histogram agree
  useEffect(() => {
    setPriorityFilter({
      min: filters.priorityMin,
      max: filters.priorityMax,
    });
  }, [filters.priorityMin, filters.priorityMax, setPriorityFilter]);

  // Convert API blocks to table format with filtering
  const blocks = useMemo((): VisibilityBlock[] => {
    return mapData.blocks.map((b) => ({
      scheduling_block_id: b.scheduling_block_id,
      original_block_id: b.original_block_id,
      priority: b.priority,
      scheduled: b.scheduled,
      num_visibility_periods: b.num_visibility_periods,
      // These are not in the visibility map endpoint but could be added
      total_visibility_hours: undefined,
      requested_hours: undefined,
    }));
  }, [mapData.blocks]);

  // Filter blocks based on analysis context
  const filteredBlocks = useMemo(() => {
    let result = blocks;

    // Priority filter from context
    if (state.priorityFilter.min !== undefined) {
      result = result.filter((b) => b.priority >= state.priorityFilter.min!);
    }
    if (state.priorityFilter.max !== undefined) {
      result = result.filter((b) => b.priority <= state.priorityFilter.max!);
    }

    // Scheduled filter from context
    if (state.scheduledFilter === 'scheduled') {
      result = result.filter((b) => b.scheduled);
    } else if (state.scheduledFilter === 'unscheduled') {
      result = result.filter((b) => !b.scheduled);
    }

    return result;
  }, [blocks, state.priorityFilter, state.scheduledFilter]);

  // Handle block click for details drawer
  const handleBlockClick = useCallback(
    (block: VisibilityBlock) => {
      setActiveBlockLocal(block);
      setActiveBlock(block.scheduling_block_id);
    },
    [setActiveBlock]
  );

  // Close details drawer
  const handleCloseDrawer = useCallback(() => {
    setActiveBlockLocal(null);
    setActiveBlock(null);
  }, [setActiveBlock]);

  const handleResetFilters = useCallback(() => {
    onFiltersChange(DEFAULT_FILTERS);
    setScheduledFilter('all');
    setPriorityFilter({});
  }, [onFiltersChange, setPriorityFilter, setScheduledFilter]);

  return (
    <PageContainer>
      {/* Summary metrics */}
      <SummaryMetrics
        mapData={mapData}
        filteredCount={filteredBlocks.length}
        selectionCount={selectionCount}
      />

      <div className="flex flex-col gap-4">
        <VisibilityFiltersBar
          filters={filters}
          priorityRange={{ min: mapData.priority_min, max: mapData.priority_max }}
          scheduledFilter={state.scheduledFilter}
          onFiltersChange={onFiltersChange}
          onScheduledFilterChange={setScheduledFilter}
          onReset={handleResetFilters}
        />

        <div className="min-w-0">
          <OpportunitiesHistogram histogramData={histogramData} isLoading={histogramLoading} />
        </div>
      </div>

      {/* Blocks table for drill-down */}
      <div className="mt-6">
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-white">Observation Blocks</h2>
          <ExportMenu
            blocks={filteredBlocks}
            totalBlocks={mapData.total_count}
            columns={[
              'scheduling_block_id',
              'original_block_id',
              'priority',
              'scheduled',
              'num_visibility_periods',
            ]}
          />
        </div>
        <BlocksTable
          blocks={filteredBlocks}
          title=""
          maxRows={200}
          onBlockClick={handleBlockClick}
          showSelection
        />
      </div>

      {/* Details drawer */}
      <BlockDetailsDrawer
        block={activeBlock}
        onClose={handleCloseDrawer}
        renderExtraDetails={(block) => (
          <div className="rounded-lg border border-slate-700 bg-slate-900/50 p-4">
            <h3 className="mb-2 text-xs font-medium uppercase tracking-wider text-slate-400">
              Visibility Windows
            </h3>
            <p className="text-lg font-semibold text-white">
              {block.num_visibility_periods} period{block.num_visibility_periods !== 1 ? 's' : ''}
            </p>
          </div>
        )}
      />
    </PageContainer>
  );
});

interface VisibilityFiltersBarProps {
  filters: FilterParams;
  priorityRange: { min: number; max: number };
  scheduledFilter: 'all' | 'scheduled' | 'unscheduled';
  onFiltersChange: (params: FilterParams) => void;
  onScheduledFilterChange: (filter: 'all' | 'scheduled' | 'unscheduled') => void;
  onReset: () => void;
}

const VisibilityFiltersBar = memo(function VisibilityFiltersBar({
  filters,
  priorityRange,
  scheduledFilter,
  onFiltersChange,
  onScheduledFilterChange,
  onReset,
}: VisibilityFiltersBarProps) {
  const [localFilters, setLocalFilters] = useState(filters);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    setLocalFilters(filters);
  }, [filters]);

  useEffect(() => {
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  const scheduleApply = useCallback((nextFilters: FilterParams) => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    debounceRef.current = setTimeout(() => {
      onFiltersChange(nextFilters);
    }, FILTER_DEBOUNCE_MS);
  }, [onFiltersChange]);

  const updateFilters = useCallback((patch: Partial<FilterParams>) => {
    setLocalFilters((current) => {
      const next = { ...current, ...patch };
      scheduleApply(next);
      return next;
    });
  }, [scheduleApply]);

  const handleReset = useCallback(() => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    setLocalFilters(DEFAULT_FILTERS);
    onReset();
  }, [onReset]);

  const priorityMinDisplay = localFilters.priorityMin ?? priorityRange.min;
  const priorityMaxDisplay = localFilters.priorityMax ?? priorityRange.max;
  const prioritySpan = Math.max(priorityRange.max - priorityRange.min, 0.1);
  const priorityMinPercent =
    ((priorityMinDisplay - priorityRange.min) / prioritySpan) * 100;
  const priorityMaxPercent =
    ((priorityMaxDisplay - priorityRange.min) / prioritySpan) * 100;

  return (
    <div className="rounded-lg border border-slate-700 bg-slate-800/50 px-5 py-4">
      <div className="mb-3 flex items-center justify-between gap-3">
        <h3 className="text-xs font-semibold uppercase tracking-wide text-slate-400">Filters</h3>
        <button
          type="button"
          onClick={handleReset}
          className="rounded px-2 py-0.5 text-xs text-slate-500 transition-colors hover:bg-slate-700/50 hover:text-slate-300"
        >
          Reset
        </button>
      </div>

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-[auto_1px_auto_1px_1fr] lg:items-start lg:gap-0">
        <div className="flex flex-col gap-1.5 lg:pr-5">
          <p className="text-[11px] font-medium uppercase tracking-wide text-slate-400">
            Block Status
          </p>
          <ScheduledFilterButtons
            scheduledFilter={scheduledFilter}
            onChange={onScheduledFilterChange}
          />
        </div>

        <div className="hidden self-stretch bg-slate-700 lg:block" />

        <div className="flex flex-col gap-1.5 lg:px-5">
          <p className="text-[11px] font-medium uppercase tracking-wide text-slate-400">
            Binning Method
          </p>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-[minmax(11rem,auto)_minmax(14rem,1fr)_auto] sm:items-center">
            <select
              value={localFilters.useCustomDuration ? 'duration' : 'bins'}
              onChange={(event) =>
                updateFilters({ useCustomDuration: event.target.value === 'duration' })
              }
              className="rounded border border-slate-600 bg-slate-700 px-2 py-1.5 text-xs text-slate-200 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            >
              <option value="bins">Number of Bins</option>
              <option value="duration">Bin Duration</option>
            </select>

            {!localFilters.useCustomDuration ? (
              <input
                type="range"
                min="10"
                max="200"
                value={localFilters.numBins}
                onChange={(event) =>
                  updateFilters({ numBins: parseInt(event.target.value, 10) })
                }
                className="h-2 min-w-0 cursor-pointer appearance-none rounded-lg bg-slate-600"
              />
            ) : (
              <input
                type="range"
                min="15"
                max="10080"
                step="15"
                value={localFilters.binDurationMinutes ?? 60}
                onChange={(event) =>
                  updateFilters({
                    binDurationMinutes: parseInt(event.target.value, 10),
                  })
                }
                className="h-2 min-w-0 cursor-pointer appearance-none rounded-lg bg-slate-600"
              />
            )}

            <span className="w-16 text-right text-sm font-medium text-white">
              {!localFilters.useCustomDuration
                ? localFilters.numBins
                : `${localFilters.binDurationMinutes ?? 60}m`}
            </span>
          </div>
        </div>

        <div className="hidden self-stretch bg-slate-700 lg:block" />

        <div className="flex min-w-0 flex-col gap-2 lg:pl-5">
          <div className="flex items-center justify-between">
            <p className="text-[11px] font-medium uppercase tracking-wide text-slate-400">
              Priority Range
            </p>
            <span className="text-[11px] tabular-nums text-slate-500">
              {priorityMinDisplay.toFixed(1)} - {priorityMaxDisplay.toFixed(1)}
            </span>
          </div>
          <div className="w-full">
            <div className="relative h-6">
              <div className="absolute inset-x-0 top-1/2 h-2 -translate-y-1/2 rounded-full bg-slate-700" />
              <div
                className="absolute top-1/2 h-2 -translate-y-1/2 rounded-full bg-primary-500"
                style={{
                  left: `${priorityMinPercent}%`,
                  width: `${Math.max(priorityMaxPercent - priorityMinPercent, 0)}%`,
                }}
              />
              <input
                aria-label="Minimum priority"
                type="range"
                min={priorityRange.min}
                max={priorityRange.max}
                step="0.1"
                value={priorityMinDisplay}
                onChange={(event) => {
                  const nextMin = Math.min(parseFloat(event.target.value), priorityMaxDisplay);
                  updateFilters({ priorityMin: nextMin });
                }}
                className={`${PRIORITY_SLIDER_CLASS} z-10`}
              />
              <input
                aria-label="Maximum priority"
                type="range"
                min={priorityRange.min}
                max={priorityRange.max}
                step="0.1"
                value={priorityMaxDisplay}
                onChange={(event) => {
                  const nextMax = Math.max(parseFloat(event.target.value), priorityMinDisplay);
                  updateFilters({ priorityMax: nextMax });
                }}
                className={`${PRIORITY_SLIDER_CLASS} z-20`}
              />
            </div>
            <div className="mt-0.5 flex items-center justify-between text-[11px] text-slate-600">
              <span>{priorityRange.min.toFixed(1)}</span>
              <span>{priorityRange.max.toFixed(1)}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
});

// =============================================================================
// Scheduled Filter Buttons (uses AnalysisContext)
// =============================================================================

interface ScheduledFilterButtonsProps {
  scheduledFilter: 'all' | 'scheduled' | 'unscheduled';
  onChange: (filter: 'all' | 'scheduled' | 'unscheduled') => void;
}

function ScheduledFilterButtons({
  scheduledFilter,
  onChange,
}: ScheduledFilterButtonsProps) {
  const options: { value: 'all' | 'scheduled' | 'unscheduled'; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'scheduled', label: 'Scheduled' },
    { value: 'unscheduled', label: 'Unscheduled' },
  ];

  return (
    <div className="inline-flex rounded-lg border border-slate-700 bg-slate-900/40 p-1">
      {options.map((opt) => (
        <button
          type="button"
          key={opt.value}
          onClick={() => onChange(opt.value)}
          className={`rounded-md px-3 py-1.5 text-sm transition-colors ${
            scheduledFilter === opt.value
              ? 'bg-primary-600/15 text-primary-200'
              : 'text-slate-400 hover:text-slate-200'
          }`}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}

// =============================================================================
// Main Page Component
// =============================================================================

function VisibilityMapPage() {
  const { scheduleId } = useParams();
  const currentId = parseInt(scheduleId ?? '0', 10);

  // Applied filter state for histogram (separate from AnalysisContext filters)
  const [appliedFilters, setAppliedFilters] = useState<FilterParams>(DEFAULT_FILTERS);

  // Get analysis context for priority filter (shared with histogram)
  const { state: analysisState } = useAnalysis();

  // Fetch visibility map data (includes blocks list)
  const { data: mapData, isLoading: mapLoading, error: mapError } = useVisibilityMap(currentId);

  // Build histogram query from applied filters + analysis context
  const histogramQuery = useMemo<VisibilityHistogramQuery>(() => {
    const query: VisibilityHistogramQuery = {};

    // Binning from local filter settings
    if (appliedFilters.useCustomDuration && appliedFilters.binDurationMinutes) {
      query.bin_duration_minutes = appliedFilters.binDurationMinutes;
    } else {
      query.num_bins = appliedFilters.numBins;
    }

    // Priority from filter settings (takes precedence) or analysis context
    const priorityMin = appliedFilters.priorityMin ?? analysisState.priorityFilter.min;
    const priorityMax = appliedFilters.priorityMax ?? analysisState.priorityFilter.max;

    if (priorityMin !== undefined) {
      query.priority_min = priorityMin;
    }
    if (priorityMax !== undefined) {
      query.priority_max = priorityMax;
    }

    if (analysisState.scheduledFilter === 'scheduled') {
      query.scheduled = true;
    } else if (analysisState.scheduledFilter === 'unscheduled') {
      query.scheduled = false;
    }

    // If blocks are selected, filter histogram to just those blocks
    if (analysisState.selectedBlockIds.size > 0 && analysisState.selectedBlockIds.size <= 100) {
      query.block_ids = Array.from(analysisState.selectedBlockIds);
    }

    return query;
  }, [
    appliedFilters,
    analysisState.priorityFilter,
    analysisState.scheduledFilter,
    analysisState.selectedBlockIds,
  ]);

  // Fetch histogram data
  const {
    data: histogramData,
    isLoading: histogramLoading,
    error: histogramError,
    refetch,
  } = useVisibilityHistogram(currentId, histogramQuery);

  // Stable callback for filter changes
  const handleFiltersChange = useCallback((params: FilterParams) => {
    setAppliedFilters(params);
  }, []);

  // Initial loading state
  if (mapLoading && !mapData) {
    return (
      <PageContainer>
        <div className="flex h-96 items-center justify-center">
          <LoadingSpinner size="lg" />
        </div>
      </PageContainer>
    );
  }

  // Error state
  const error = mapError || histogramError;
  if (error) {
    return (
      <PageContainer>
        <ErrorMessage
          title="Failed to load visibility map"
          message={(error as Error).message}
          onRetry={() => refetch()}
        />
      </PageContainer>
    );
  }

  // No data state
  if (!mapData) {
    return (
      <PageContainer>
        <ErrorMessage message="No data available" />
      </PageContainer>
    );
  }

  return (
    <VisibilityMapContent
      mapData={mapData}
      histogramData={histogramData}
      histogramLoading={histogramLoading}
      filters={appliedFilters}
      onFiltersChange={handleFiltersChange}
    />
  );
}

export default VisibilityMapPage;
