/**
 * Sky Map page – d3-celestial all-sky projection of observation targets.
 *
 * Renders an Aitoff equal-area all-sky map with the real Milky Way,
 * equatorial grid, and observation targets colored by priority bin.
 * Powered by d3-celestial (loaded globally via /celestial.js).
 */
import { useState, useMemo, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useSkyMap } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  Icon,
  MetricCard,
  CelestialSkyMap,
  PageHeader,
  PageContainer,
  MetricsGrid,
  ChartPanel,
  SkyMapFilters,
} from '@/components';
import type { SkyMapFilterState } from '@/components';
import type { LightweightBlock } from '@/api/types';

// ─── MJD conversion utilities ───────────────────────────────────────

const MJD_EPOCH = 2400000.5;
const UNIX_EPOCH_JD = 2440587.5;

function mjdToUtc(mjd: number): string {
  const jd = mjd + MJD_EPOCH;
  const unixMs = (jd - UNIX_EPOCH_JD) * 86400000;
  return new Date(unixMs).toISOString();
}

function utcToMjd(utcString: string): number {
  if (!utcString) return 0;
  const unixMs = new Date(utcString).getTime();
  const jd = unixMs / 86400000 + UNIX_EPOCH_JD;
  return jd - MJD_EPOCH;
}

function toDatetimeLocal(utcIso: string): string {
  if (!utcIso) return '';
  return utcIso.slice(0, 16);
}

function createDefaultFilters(
  priorityMin: number,
  priorityMax: number,
  scheduledTimeMin: number | null,
  scheduledTimeMax: number | null,
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

function filterBlocks(blocks: LightweightBlock[], filters: SkyMapFilterState): LightweightBlock[] {
  return blocks.filter((block) => {
    const isScheduled = block.scheduled_period !== null;
    if (isScheduled && !filters.showScheduled) return false;
    if (!isScheduled && !filters.showUnscheduled) return false;
    if (block.priority < filters.priorityMin || block.priority > filters.priorityMax) return false;
    if (
      isScheduled &&
      block.scheduled_period &&
      filters.scheduledBeginUtc &&
      filters.scheduledEndUtc
    ) {
      const { start, stop } = block.scheduled_period;
      const filterBeginMjd = utcToMjd(filters.scheduledBeginUtc);
      const filterEndMjd = utcToMjd(filters.scheduledEndUtc);
      if (stop < filterBeginMjd || start > filterEndMjd) return false;
    }
    return true;
  });
}

// ─── Component ──────────────────────────────────────────────────────

function SkyMap() {
  const { scheduleId } = useParams();
  const navigate = useNavigate();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useSkyMap(id);

  const [filters, setFilters] = useState<SkyMapFilterState | null>(null);

  const activeFilters = useMemo(() => {
    if (!data) return null;
    if (filters) return filters;
    return createDefaultFilters(
      data.priority_min,
      data.priority_max,
      data.scheduled_time_min,
      data.scheduled_time_max,
    );
  }, [data, filters]);

  const filteredBlocks = useMemo(() => {
    if (!data || !activeFilters) return { all: [], scheduled: [], unscheduled: [] };
    const filtered = filterBlocks(data.blocks, activeFilters);
    return {
      all: filtered,
      scheduled: filtered.filter((b) => b.scheduled_period !== null),
      unscheduled: filtered.filter((b) => b.scheduled_period === null),
    };
  }, [data, activeFilters]);

  const handleReset = useCallback(() => {
    if (data) {
      setFilters(
        createDefaultFilters(
          data.priority_min,
          data.priority_max,
          data.scheduled_time_min,
          data.scheduled_time_max,
        ),
      );
    }
  }, [data]);

  // ── Loading / error / empty states ──────────────────────────────

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

  const { all: allFiltered, scheduled } = filteredBlocks;
  const totalFiltered = allFiltered.length;
  const schedulingRate =
    totalFiltered > 0 ? ((scheduled.length / totalFiltered) * 100).toFixed(1) : '0';

  const scheduledTimeRange = {
    min: data.scheduled_time_min ? mjdToUtc(data.scheduled_time_min) : null,
    max: data.scheduled_time_max ? mjdToUtc(data.scheduled_time_max) : null,
  };

  return (
    <PageContainer>
      <PageHeader
        title="Sky Map"
        description="All-sky Aitoff projection with Milky Way and observation targets"
      />

      <MetricsGrid>
        <MetricCard
          label="Displayed Blocks"
          value={`${totalFiltered} / ${data.total_count}`}
          icon={<Icon name="target" />}
        />
        <MetricCard
          label="Scheduled"
          value={scheduled.length}
          icon={<Icon name="check-circle" />}
        />
        <MetricCard
          label="Scheduling Rate"
          value={`${schedulingRate}%`}
          icon={<Icon name="chart-bar" />}
        />
        <MetricCard
          label="Priority Range"
          value={`${data.priority_min.toFixed(1)} - ${data.priority_max.toFixed(1)}`}
          icon={<Icon name="star" />}
        />
      </MetricsGrid>

      <div className="grid gap-4 lg:grid-cols-[280px_1fr]">
        <div className="order-2 lg:order-1">
          <SkyMapFilters
            filters={activeFilters}
            onChange={setFilters}
            scheduledTimeRange={scheduledTimeRange}
            priorityRange={{ min: data.priority_min, max: data.priority_max }}
            onReset={handleReset}
          />
        </div>

        <div className="order-1 flex flex-col gap-4 lg:order-2">
          <ChartPanel title="Celestial Coordinates (Aitoff)">
            <CelestialSkyMap blocks={filteredBlocks.all} bins={data.priority_bins} />
          </ChartPanel>
        </div>
      </div>
    </PageContainer>
  );
}

export default SkyMap;
