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
import { useState, useCallback, useMemo, memo } from 'react';
import { useParams } from 'react-router-dom';
import { useVisibilityMap, useVisibilityHistogram } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  PageContainer,
  PageHeader,
  MetricsGrid,
  MetricCard,
} from '@/components';
import {
  FilterSettings,
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
  const schedulingRate = mapData.total_count > 0
    ? ((mapData.scheduled_count / mapData.total_count) * 100).toFixed(1)
    : '0';

  return (
    <>
      <PageHeader
        title="Visibility Analysis"
        description="Target visibility over the observation period with block-level drill-down"
      />
      <MetricsGrid>
        <MetricCard label="Total Blocks" value={mapData.total_count} icon="📊" />
        <MetricCard
          label="Scheduled"
          value={`${mapData.scheduled_count} (${schedulingRate}%)`}
          icon="✅"
        />
        <MetricCard
          label="Priority Range"
          value={`${mapData.priority_min.toFixed(1)} – ${mapData.priority_max.toFixed(1)}`}
          icon="⭐"
        />
        {filteredCount !== mapData.total_count && (
          <MetricCard label="Filtered" value={filteredCount} icon="🔍" />
        )}
        {selectionCount > 0 && (
          <MetricCard label="Selected" value={selectionCount} icon="✓" />
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
  onFiltersChange: (params: FilterParams) => void;
}

const VisibilityMapContent = memo(function VisibilityMapContent({
  mapData,
  histogramData,
  histogramLoading,
  onFiltersChange,
}: VisibilityMapContentProps) {
  const { state, setActiveBlock, selectionCount } = useAnalysis();
  const [activeBlock, setActiveBlockLocal] = useState<VisibilityBlock | null>(null);

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

    // If blocks are selected, show only selected (or all if none selected)
    if (state.selectedBlockIds.size > 0) {
      result = result.filter((b) => state.selectedBlockIds.has(b.scheduling_block_id));
    }

    return result;
  }, [blocks, state.priorityFilter, state.scheduledFilter, state.selectedBlockIds]);

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

  return (
    <PageContainer>
      {/* Summary metrics */}
      <SummaryMetrics
        mapData={mapData}
        filteredCount={filteredBlocks.length}
        selectionCount={selectionCount}
      />

      {/* Main content: histogram + controls */}
      <div className="flex flex-col gap-6 lg:flex-row">
        {/* Left panel: Filters */}
        <aside className="shrink-0 lg:w-72">
          <div className="space-y-4">
            {/* Histogram controls */}
            <div className="rounded-lg border border-slate-700 bg-slate-800/50 p-4">
              <FilterSettings
                defaultParams={DEFAULT_FILTERS}
                mapPriorityMin={mapData.priority_min}
                mapPriorityMax={mapData.priority_max}
                onParamsChange={onFiltersChange}
              />
            </div>

            {/* Quick filter: scheduled status */}
            <div className="rounded-lg border border-slate-700 bg-slate-800/50 p-4">
              <h3 className="mb-3 text-sm font-medium text-slate-200">Block Status</h3>
              <ScheduledFilterButtons />
            </div>
          </div>
        </aside>

        {/* Right panel: Histogram */}
        <div className="min-w-0 flex-1">
          <OpportunitiesHistogram
            histogramData={histogramData}
            isLoading={histogramLoading}
          />
        </div>
      </div>

      {/* Blocks table for drill-down */}
      <div className="mt-6">
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-lg font-semibold text-white">Observation Blocks</h2>
          <ExportMenu
            blocks={filteredBlocks}
            totalBlocks={mapData.total_count}
            columns={['scheduling_block_id', 'original_block_id', 'priority', 'scheduled', 'num_visibility_periods']}
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

// =============================================================================
// Scheduled Filter Buttons (uses AnalysisContext)
// =============================================================================

function ScheduledFilterButtons() {
  const { state, setScheduledFilter } = useAnalysis();

  const options: { value: 'all' | 'scheduled' | 'unscheduled'; label: string }[] = [
    { value: 'all', label: 'All' },
    { value: 'scheduled', label: 'Scheduled' },
    { value: 'unscheduled', label: 'Unscheduled' },
  ];

  return (
    <div className="flex gap-1">
      {options.map((opt) => (
        <button
          key={opt.value}
          onClick={() => setScheduledFilter(opt.value)}
          className={`flex-1 rounded-md px-3 py-1.5 text-xs font-medium transition-colors ${
            state.scheduledFilter === opt.value
              ? 'bg-primary-600 text-white'
              : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
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
  const {
    data: mapData,
    isLoading: mapLoading,
    error: mapError,
  } = useVisibilityMap(currentId);

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
      query.priority_min = Math.floor(priorityMin);
    }
    if (priorityMax !== undefined) {
      query.priority_max = Math.ceil(priorityMax);
    }

    // If blocks are selected, filter histogram to just those blocks
    if (analysisState.selectedBlockIds.size > 0 && analysisState.selectedBlockIds.size <= 100) {
      query.block_ids = Array.from(analysisState.selectedBlockIds);
    }

    return query;
  }, [appliedFilters, analysisState.priorityFilter, analysisState.selectedBlockIds]);

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
      onFiltersChange={handleFiltersChange}
    />
  );
}

export default VisibilityMapPage;
