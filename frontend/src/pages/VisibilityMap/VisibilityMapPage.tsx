/**
 * VisibilityMap page - Histogram of target visibility over observation period.
 * 
 * This page displays a visibility histogram showing when observation targets
 * are visible during the scheduling period. Users can filter by priority range
 * and adjust histogram binning to analyze visibility patterns.
 * 
 * ARCHITECTURE:
 * - Thin page component that assembles shared/feature components
 * - FilterSettings: uncontrolled component with debounced updates (prevents blink)
 * - OpportunitiesHistogram: memoized with useDeferredValue for smooth updates
 * - InfoCards: inline memoized section (page-specific layout only)
 * 
 * RENDER ISOLATION:
 * - FilterSettings owns its state completely (no sync from parent)
 * - InfoCards depends only on mapData (stable after initial load)
 * - Histogram updates are isolated via memoization boundaries
 */
import { useState, useCallback, useMemo, memo } from 'react';
import { useParams } from 'react-router-dom';
import { useVisibilityMap, useVisibilityHistogram, useRemountDetector, useRenderCounter } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  PageContainer,
  PageHeader,
  MetricsGrid,
  MetricCard,
} from '@/components';
import { FilterSettings, OpportunitiesHistogram, type FilterParams } from '@/features/schedules';

// Types for histogram query
interface HistogramQuery {
  num_bins?: number;
  bin_duration_minutes?: number;
  priority_min?: number;
  priority_max?: number;
}

// Default filter values - stable reference outside component
const DEFAULT_FILTERS: FilterParams = {
  numBins: 50,
  binDurationMinutes: undefined,
  priorityMin: undefined,
  priorityMax: undefined,
  useCustomDuration: false,
};

// Map data type (inferred from API response)
interface MapData {
  total_count: number;
  scheduled_count: number;
  priority_min: number;
  priority_max: number;
}

// Histogram data type
interface HistogramBin {
  bin_start_unix: number;
  bin_end_unix: number;
  visible_count: number;
}

/**
 * InfoCards - Page header and summary metrics section.
 * Inline component - only depends on mapData which is stable after initial load.
 */
interface InfoCardsProps {
  totalCount: number;
  scheduledCount: number;
  priorityMin: number;
  priorityMax: number;
}

const InfoCards = memo(function InfoCards({
  totalCount,
  scheduledCount,
  priorityMin,
  priorityMax,
}: InfoCardsProps) {
  useRemountDetector('InfoCards');
  useRenderCounter('InfoCards');

  return (
    <>
      <PageHeader
        title="Visibility Map"
        description={`Target visibility over the observation period (${totalCount} blocks)`}
      />
      <MetricsGrid>
        <MetricCard label="Total Blocks" value={totalCount} icon="📊" />
        <MetricCard label="Scheduled" value={scheduledCount} icon="✅" />
        <MetricCard
          label="Priority Range"
          value={`${priorityMin.toFixed(1)} - ${priorityMax.toFixed(1)}`}
          icon="⭐"
        />
      </MetricsGrid>
    </>
  );
});

/**
 * Content component - renders the stable layout.
 * No data fetching - receives stable props and renders children directly in JSX.
 */
interface VisibilityMapContentProps {
  mapData: MapData;
  histogramData: HistogramBin[] | undefined;
  histogramLoading: boolean;
  onFiltersChange: (params: FilterParams) => void;
}

const VisibilityMapContent = memo(function VisibilityMapContent({
  mapData,
  histogramData,
  histogramLoading,
  onFiltersChange,
}: VisibilityMapContentProps) {
  useRemountDetector('VisibilityMapContent');
  useRenderCounter('VisibilityMapContent');

  return (
    <PageContainer>
      {/* Info cards - stable, only depends on mapData which rarely changes */}
      <InfoCards
        totalCount={mapData.total_count}
        scheduledCount={mapData.scheduled_count}
        priorityMin={mapData.priority_min}
        priorityMax={mapData.priority_max}
      />

      {/* Split layout: controls left, chart right */}
      <div className="flex flex-col gap-6 lg:flex-row">
        {/* Controls panel - fixed width on desktop */}
        <aside className="shrink-0 lg:w-72">
          <div className="rounded-lg border border-slate-700 bg-slate-800/50 p-4">
            <FilterSettings
              defaultParams={DEFAULT_FILTERS}
              mapPriorityMin={mapData.priority_min}
              mapPriorityMax={mapData.priority_max}
              onParamsChange={onFiltersChange}
            />
          </div>
        </aside>
        
        {/* Chart area - flexible width with overflow prevention */}
        <div className="min-w-0 flex-1">
          <OpportunitiesHistogram
            histogramData={histogramData}
            isLoading={histogramLoading}
          />
        </div>
      </div>
    </PageContainer>
  );
});

/**
 * Main page component - handles data fetching and state management.
 * Loading/error states are handled here to keep content component stable.
 */
function VisibilityMapPage() {
  useRemountDetector('VisibilityMapPage');
  useRenderCounter('VisibilityMapPage');

  const { scheduleId } = useParams();
  const currentId = parseInt(scheduleId ?? '0', 10);

  // Applied filter state - updated by FilterSettings via debounced callback
  const [appliedFilters, setAppliedFilters] = useState<FilterParams>(DEFAULT_FILTERS);

  // Fetch visibility map data (metadata)
  const { 
    data: mapData, 
    isLoading: mapLoading, 
    error: mapError 
  } = useVisibilityMap(currentId);

  // Build histogram query from applied filters
  const histogramQuery = useMemo<HistogramQuery>(() => {
    const query: HistogramQuery = {};

    if (appliedFilters.useCustomDuration && appliedFilters.binDurationMinutes) {
      query.bin_duration_minutes = appliedFilters.binDurationMinutes;
    } else {
      query.num_bins = appliedFilters.numBins;
    }

    if (appliedFilters.priorityMin !== undefined) {
      query.priority_min = Math.floor(appliedFilters.priorityMin);
    }
    if (appliedFilters.priorityMax !== undefined) {
      query.priority_max = Math.ceil(appliedFilters.priorityMax);
    }

    return query;
  }, [appliedFilters]);

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

  // Handle error retry
  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  // Initial loading state (no mapData yet)
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
          onRetry={handleRetry}
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

  // Render content - mapData is stable after initial load
  // histogramData changes cause re-render but ONLY of OpportunitiesHistogram
  // because VisibilityMapContent is memoized and InfoCards/FilterSettings
  // don't depend on histogramData
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
