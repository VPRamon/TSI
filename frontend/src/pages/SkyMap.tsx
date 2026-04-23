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
import { mjdToDate, dateToMjd, isValidDate } from '@/constants/dates';
import { downloadCanvasAsPng } from '@/lib/imageExport';

const SKY_MAP_CONTAINER_ID = 'sky-map-canvas';
const SECONDARY_ACTION_BUTTON_CLASS =
  'rounded-md border border-slate-600 bg-slate-800/70 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-800';

// ─── MJD conversion utilities ───────────────────────────────────────

function mjdToUtc(mjd: number | null | undefined): string | null {
  if (!Number.isFinite(mjd)) return null;
  const finiteMjd = mjd as number;
  const date = mjdToDate(finiteMjd);
  return isValidDate(date) ? date.toISOString() : null;
}

function utcToMjd(utcString: string): number | null {
  if (!utcString) return null;
  // datetime-local values lack a timezone suffix; treat them as UTC
  const utc =
    /[Zz]$/.test(utcString) || /[+-]\d{2}:?\d{2}$/.test(utcString) ? utcString : utcString + 'Z';
  const date = new Date(utc);
  return isValidDate(date) ? dateToMjd(date) : null;
}

function toDatetimeLocal(utcIso: string | null): string {
  if (!utcIso) return '';
  return utcIso.slice(0, 19);
}

function createDefaultFilters(
  priorityMin: number,
  priorityMax: number,
  scheduledTimeMin: number | null,
  scheduledTimeMax: number | null
): SkyMapFilterState {
  return {
    showScheduled: true,
    showUnscheduled: true,
    scheduledBeginUtc: toDatetimeLocal(mjdToUtc(scheduledTimeMin)),
    scheduledEndUtc: toDatetimeLocal(mjdToUtc(scheduledTimeMax)),
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
      (filters.scheduledBeginUtc || filters.scheduledEndUtc)
    ) {
      const { start, stop } = block.scheduled_period;
      const filterBeginMjd = utcToMjd(filters.scheduledBeginUtc);
      const filterEndMjd = utcToMjd(filters.scheduledEndUtc);
      if (filterBeginMjd !== null && stop < filterBeginMjd) return false;
      if (filterEndMjd !== null && start > filterEndMjd) return false;
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
      data.scheduled_time_max
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

  const [showPath, setShowPath] = useState(false);

  const pathBlocks = useMemo(
    () =>
      filteredBlocks.scheduled
        .slice()
        .sort((a, b) => (a.scheduled_period!.start - b.scheduled_period!.start)),
    [filteredBlocks.scheduled]
  );

  const handleReset = useCallback(() => {
    if (data) {
      setFilters(
        createDefaultFilters(
          data.priority_min,
          data.priority_max,
          data.scheduled_time_min,
          data.scheduled_time_max
        )
      );
    }
  }, [data]);

  const handleDownloadSkyMap = useCallback(() => {
    const canvas = document.querySelector<HTMLCanvasElement>(`#${SKY_MAP_CONTAINER_ID} canvas`);
    if (!canvas) return;
    downloadCanvasAsPng(canvas, 'sky-map');
  }, []);

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
                ? 'The requested schedule does not exist. It may have been deleted or the server may have restarted.'
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
    min: mjdToUtc(data.scheduled_time_min),
    max: mjdToUtc(data.scheduled_time_max),
  };

  return (
    <PageContainer>
      <PageHeader
        title="Sky Map"
        description="All-sky Aitoff projection with Milky Way, observation targets, and RA/Dec graticule labels"
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

      <div className="flex flex-col gap-4">
        <SkyMapFilters
          filters={activeFilters}
          onChange={setFilters}
          scheduledTimeRange={scheduledTimeRange}
          priorityRange={{ min: data.priority_min, max: data.priority_max }}
          bins={data.priority_bins}
          onReset={handleReset}
        />

        <ChartPanel
          title="Celestial Coordinates (Aitoff)"
          headerActions={
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={() => setShowPath((prev) => !prev)}
                className={[
                  SECONDARY_ACTION_BUTTON_CLASS,
                  showPath
                    ? 'border-amber-500 bg-amber-900/40 text-amber-300 hover:bg-amber-800/50'
                    : '',
                ]
                  .filter(Boolean)
                  .join(' ')}
                aria-pressed={showPath}
                title={showPath ? 'Hide observation path' : 'Show observation path'}
              >
                Display Path
              </button>
              <button
                type="button"
                onClick={handleDownloadSkyMap}
                className={SECONDARY_ACTION_BUTTON_CLASS}
              >
                Download PNG
              </button>
            </div>
          }
        >
          <CelestialSkyMap
            blocks={filteredBlocks.all}
            bins={data.priority_bins}
            containerId={SKY_MAP_CONTAINER_ID}
            showCoordinateGuide
            showPath={showPath}
            pathBlocks={pathBlocks}
          />
        </ChartPanel>
      </div>
    </PageContainer>
  );
}

export default SkyMap;
