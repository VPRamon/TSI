/**
 * AltAz page - Altitude/Azimuth analysis for observation targets.
 *
 * Requests altitude/azimuth curves from the backend for selected targets
 * as seen from a configurable observatory location over a custom UTC interval.
 */
import { useState, useMemo, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { useSkyMap, usePlotlyTheme, usePlotlyDownload, useAltAz } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  PlotlyChart,
  PageHeader,
  PageContainer,
  ChartPanel,
  ToolbarRow,
} from '@/components';
import type { AltAzRequest, AltAzTargetRequest } from '@/api/types';

// ─── Observatory presets ─────────────────────────────────────────────

interface ObservatoryPreset {
  label: string;
  lon: number;
  lat: number;
  height: number;
}

const OBSERVATORIES: ObservatoryPreset[] = [
  {
    label: 'Roque de los Muchachos (ORM)',
    lon: -17.879,
    lat: 28.764,
    height: 2396,
  },
  {
    label: 'Paranal (VLT)',
    lon: -70.404,
    lat: -24.627,
    height: 2635,
  },
  {
    label: 'Mauna Kea',
    lon: -155.468,
    lat: 19.826,
    height: 4205,
  },
  {
    label: 'La Silla',
    lon: -70.73,
    lat: -29.257,
    height: 2347,
  },
];

// ─── Computation helpers ─────────────────────────────────────────────

/** Minimal target fields required for Alt/Az computation. */
interface TargetForComputation {
  original_block_id: string;
  block_name: string;
  priority: number;
  target_ra_deg: number;
  target_dec_deg: number;
}

/** Unique sky position deduped from the block list. */
interface UniqueTarget extends TargetForComputation {
  /** Stable key derived from rounded RA/Dec — used for selection tracking. */
  key: string;
  /** Number of scheduling blocks that share this sky position. */
  blockCount: number;
}

/** Round coordinate to 4 dp (~0.36 arcsec) for deduplication. */
const raDecKey = (ra: number, dec: number) => `${ra.toFixed(4)}_${dec.toFixed(4)}`;

const DEFAULT_PERIOD_HOURS = 24;
const TARGET_SAMPLE_INTERVAL_MINUTES = 10;

const mjdToDate = (mjd: number): Date => new Date((mjd - 40587) * 86400000);

const dateToMjd = (date: Date): number => date.getTime() / 86400000 + 40587;

function formatDateTimeLocalUtc(date: Date): string {
  const year = date.getUTCFullYear();
  const month = String(date.getUTCMonth() + 1).padStart(2, '0');
  const day = String(date.getUTCDate()).padStart(2, '0');
  const hours = String(date.getUTCHours()).padStart(2, '0');
  const minutes = String(date.getUTCMinutes()).padStart(2, '0');
  return `${year}-${month}-${day}T${hours}:${minutes}`;
}

function parseDateTimeLocalUtc(value: string): Date {
  return new Date(`${value}:00Z`);
}

// ─── Target selector ─────────────────────────────────────────────────

const MAX_DISPLAY = 200;
const MAX_TARGETS = 10;

// Stable colors for altitude traces
const TRACE_COLORS = [
  '#3b82f6',
  '#ef4444',
  '#22c55e',
  '#f59e0b',
  '#8b5cf6',
  '#ec4899',
  '#06b6d4',
  '#f97316',
  '#14b8a6',
  '#a855f7',
];

// ─── Main component ──────────────────────────────────────────────────

function AltAz() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data: skyData, isLoading: skyLoading, error: skyError } = useSkyMap(id);

  // Observatory selection
  const [presetIndex, setPresetIndex] = useState(0);
  const [useCustom, setUseCustom] = useState(false);
  const [customLon, setCustomLon] = useState(-17.879);
  const [customLat, setCustomLat] = useState(28.764);
  const [customHeight, setCustomHeight] = useState(2396);

  const observatory = useMemo(
    () =>
      useCustom
        ? { lon: customLon, lat: customLat, height: customHeight }
        : OBSERVATORIES[presetIndex],
    [useCustom, customLon, customLat, customHeight, presetIndex]
  );

  const [startTimeStr, setStartTimeStr] = useState(() => {
    const start = new Date();
    start.setUTCHours(12, 0, 0, 0);
    return formatDateTimeLocalUtc(start);
  });
  const [endTimeStr, setEndTimeStr] = useState(() => {
    const end = new Date();
    end.setUTCHours(12 + DEFAULT_PERIOD_HOURS, 0, 0, 0);
    return formatDateTimeLocalUtc(end);
  });

  // Target selection
  const [selectedKeys, setSelectedKeys] = useState<Set<string>>(new Set());
  const [search, setSearch] = useState('');

  const startDate = useMemo(() => parseDateTimeLocalUtc(startTimeStr), [startTimeStr]);
  const endDate = useMemo(() => parseDateTimeLocalUtc(endTimeStr), [endTimeStr]);
  const hasValidWindow = startDate.getTime() < endDate.getTime();

  // Deduplicate blocks by sky position (unique RA/Dec)
  const uniqueTargets = useMemo((): UniqueTarget[] => {
    if (!skyData) return [];
    const map = new Map<string, UniqueTarget>();
    for (const b of skyData.blocks) {
      const key = raDecKey(b.target_ra_deg, b.target_dec_deg);
      const existing = map.get(key);
      if (!existing || b.priority > existing.priority) {
        map.set(key, {
          key,
          original_block_id: b.original_block_id,
          block_name: b.block_name,
          priority: b.priority,
          target_ra_deg: b.target_ra_deg,
          target_dec_deg: b.target_dec_deg,
          blockCount: existing?.blockCount ?? 1,
        });
      } else {
        existing.blockCount += 1;
      }
    }
    return Array.from(map.values()).sort((a, b) => b.priority - a.priority);
  }, [skyData]);

  // Filter unique targets by search term
  const filteredTargets = useMemo(() => {
    if (!search) return uniqueTargets.slice(0, MAX_DISPLAY);
    const q = search.toLowerCase();
    return uniqueTargets
      .filter(
        (t) =>
          t.original_block_id.toLowerCase().includes(q) || t.block_name.toLowerCase().includes(q)
      )
      .slice(0, MAX_DISPLAY);
  }, [uniqueTargets, search]);

  const selectedTargets = useMemo(
    () => uniqueTargets.filter((target) => selectedKeys.has(target.key)),
    [uniqueTargets, selectedKeys]
  );

  const altAzRequest = useMemo<AltAzRequest | undefined>(() => {
    if (!hasValidWindow || selectedTargets.length === 0) return undefined;

    return {
      observatory: {
        lon_deg: observatory.lon,
        lat_deg: observatory.lat,
        height: observatory.height,
      },
      start_mjd: dateToMjd(startDate),
      end_mjd: dateToMjd(endDate),
      targets: selectedTargets.map(
        (target): AltAzTargetRequest => ({
          original_block_id: target.original_block_id,
          block_name: target.block_name,
          priority: target.priority,
          target_ra_deg: target.target_ra_deg,
          target_dec_deg: target.target_dec_deg,
        })
      ),
    };
  }, [hasValidWindow, selectedTargets, observatory, startDate, endDate]);

  const {
    data: altAzData,
    isFetching: altAzFetching,
    error: altAzError,
  } = useAltAz(id, altAzRequest);

  const sampleTimes = useMemo(
    () => altAzData?.sample_times_mjd.map(mjdToDate) ?? [],
    [altAzData]
  );
  const curves = altAzData?.curves ?? [];
  const computing = altAzFetching;

  const toggleTarget = useCallback((key: string) => {
    setSelectedKeys((prev) => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else if (next.size < MAX_TARGETS) {
        next.add(key);
      }
      return next;
    });
  }, []);

  const clearSelection = useCallback(() => setSelectedKeys(new Set()), []);

  // Chart theme
  const { layout: altLayout, config } = usePlotlyTheme({
    title: 'Target Altitude',
    xAxis: { title: 'UTC Time', type: 'date' },
    yAxis: { title: 'Altitude (degrees)', range: [-10, 90] },
  });

  const { layout: azLayout } = usePlotlyTheme({
    title: 'Target Azimuth',
    xAxis: { title: 'UTC Time', type: 'date' },
    yAxis: { title: 'Azimuth (degrees)', range: [0, 360] },
  });

  const { onInitialized: onAltInit, downloadButton: altDownload } =
    usePlotlyDownload('Altitude vs Time');
  const { onInitialized: onAzInit, downloadButton: azDownload } =
    usePlotlyDownload('Azimuth vs Time');

  // Extend altitude layout with horizon line
  const altitudeLayout = useMemo(
    () => ({
      ...altLayout,
      shapes: [
        {
          type: 'line' as const,
          xref: 'paper' as const,
          yref: 'y' as const,
          x0: 0,
          x1: 1,
          y0: 0,
          y1: 0,
          line: { color: '#ef4444', width: 1, dash: 'dash' as const },
        },
      ],
    }),
    [altLayout]
  );

  // Build Plotly traces
  const altTraces: Plotly.Data[] = useMemo(
    () =>
      curves.map((curve, i) => {
        const label = curve.block_name
          ? `${curve.block_name} (${curve.original_block_id})`
          : curve.original_block_id;
        return {
          type: 'scatter' as const,
          mode: 'lines' as const,
          name: `${label} p=${curve.priority.toFixed(1)}`,
          x: sampleTimes,
          y: curve.altitudes_deg,
          line: { color: TRACE_COLORS[i % TRACE_COLORS.length], width: 2 },
          hovertemplate: `<b>${label}</b><br>Alt: %{y:.1f}°<br>%{x|%H:%M UTC}<extra></extra>`,
        };
      }),
    [curves, sampleTimes]
  );

  const azTraces: Plotly.Data[] = useMemo(
    () =>
      curves.map((curve, i) => {
        const label = curve.block_name
          ? `${curve.block_name} (${curve.original_block_id})`
          : curve.original_block_id;
        return {
          type: 'scatter' as const,
          mode: 'lines' as const,
          name: `${label} p=${curve.priority.toFixed(1)}`,
          x: sampleTimes,
          y: curve.azimuths_deg,
          line: { color: TRACE_COLORS[i % TRACE_COLORS.length], width: 2 },
          hovertemplate: `<b>${label}</b><br>Az: %{y:.1f}°<br>%{x|%H:%M UTC}<extra></extra>`,
        };
      }),
    [curves, sampleTimes]
  );

  // Loading states
  if (skyLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (skyError) {
    return <ErrorMessage title="Failed to load targets" message={(skyError as Error).message} />;
  }

  if (!skyData) {
    return <ErrorMessage message="No data available" />;
  }

  return (
    <PageContainer>
      <PageHeader
        title="Altitude / Azimuth"
        description="Compute target altitude and azimuth from an observatory over a customizable time window"
      />

      <ToolbarRow className="!grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 lg:items-start">
        <div>
          <label className="mb-1.5 block text-xs font-medium text-slate-400">Observatory</label>
          <select
            value={useCustom ? 'custom' : String(presetIndex)}
            onChange={(e) => {
              if (e.target.value === 'custom') {
                setUseCustom(true);
              } else {
                setUseCustom(false);
                setPresetIndex(parseInt(e.target.value, 10));
              }
            }}
            className="w-full rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          >
            {OBSERVATORIES.map((obs, i) => (
              <option key={obs.label} value={String(i)}>
                {obs.label}
              </option>
            ))}
            <option value="custom">Custom Location</option>
          </select>
          <p className="mt-1.5 text-xs text-slate-500">
            {observatory.lat.toFixed(3)}° N, {observatory.lon.toFixed(3)}° E, {observatory.height} m
          </p>
        </div>

        <div>
          <label className="mb-1.5 block text-xs font-medium text-slate-400">
            Start Time (UTC)
          </label>
          <input
            type="datetime-local"
            value={startTimeStr}
            onChange={(e) => setStartTimeStr(e.target.value)}
            className="w-full rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
        </div>

        <div>
          <label className="mb-1.5 block text-xs font-medium text-slate-400">End Time (UTC)</label>
          <input
            type="datetime-local"
            value={endTimeStr}
            onChange={(e) => setEndTimeStr(e.target.value)}
            className="w-full rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
          <p className="mt-1.5 text-xs text-slate-500">Default range is 24 hours.</p>
        </div>

        {(!hasValidWindow || (altAzError && altAzRequest)) && (
          <div className="sm:col-span-2 lg:col-span-3">
            {!hasValidWindow && (
              <p className="text-xs text-red-400">End time must be later than start time.</p>
            )}
            {altAzError && altAzRequest && (
              <p className="text-xs text-red-400">
                Alt/Az error: {altAzError instanceof Error ? altAzError.message : String(altAzError)}
              </p>
            )}
          </div>
        )}
      </ToolbarRow>

      {useCustom && (
        <ToolbarRow className="!grid grid-cols-1 gap-4 sm:grid-cols-3">
          <div>
            <label className="mb-1 block text-xs text-slate-400">Lon (°)</label>
            <input
              type="number"
              step="0.001"
              value={customLon}
              onChange={(e) => setCustomLon(parseFloat(e.target.value) || 0)}
              className="w-full rounded border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none"
            />
          </div>
          <div>
            <label className="mb-1 block text-xs text-slate-400">Lat (°)</label>
            <input
              type="number"
              step="0.001"
              value={customLat}
              onChange={(e) => setCustomLat(parseFloat(e.target.value) || 0)}
              className="w-full rounded border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none"
            />
          </div>
          <div>
            <label className="mb-1 block text-xs text-slate-400">Height (m)</label>
            <input
              type="number"
              step="1"
              value={customHeight}
              onChange={(e) => setCustomHeight(parseFloat(e.target.value) || 0)}
              className="w-full rounded border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none"
            />
          </div>
        </ToolbarRow>
      )}

      <ChartPanel title={`Targets (${selectedKeys.size}/${MAX_TARGETS})`}>
        <div className="flex flex-col gap-3">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <input
              type="text"
              placeholder="Search targets..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="min-w-[240px] flex-1 rounded-md border border-slate-600 bg-slate-700 px-3 py-1.5 text-sm text-white placeholder-slate-500 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            />
            {selectedKeys.size > 0 && (
              <button
                onClick={clearSelection}
                className="rounded px-2 py-1 text-xs text-slate-400 hover:bg-slate-700 hover:text-slate-200"
              >
                Clear Selection
              </button>
            )}
          </div>
          <div className="scrollbar-thin max-h-48 overflow-y-auto rounded border border-slate-700 bg-slate-800/50">
            {filteredTargets.map((target) => {
              const isSelected = selectedKeys.has(target.key);
              return (
                <button
                  key={target.key}
                  onClick={() => toggleTarget(target.key)}
                  className={`flex w-full items-start justify-between gap-2 px-3 py-2 text-left text-xs transition-colors ${
                    isSelected
                      ? 'bg-primary-600/20 text-primary-300'
                      : 'text-slate-300 hover:bg-slate-700/50'
                  }`}
                >
                  <span className="min-w-0 flex-1">
                    <span className="block truncate font-medium">{target.original_block_id}</span>
                    {target.block_name && (
                      <span className="block truncate text-slate-400">{target.block_name}</span>
                    )}
                    <span className="block text-slate-500">
                      RA {target.target_ra_deg.toFixed(2)}° / Dec {target.target_dec_deg.toFixed(2)}
                      °
                      {target.blockCount > 1 && (
                        <span className="ml-1.5 text-slate-600">({target.blockCount} blocks)</span>
                      )}
                    </span>
                  </span>
                  <span className="shrink-0 text-slate-500">p={target.priority.toFixed(1)}</span>
                </button>
              );
            })}
            {filteredTargets.length === 0 && (
              <p className="px-3 py-2 text-xs text-slate-500">No targets found</p>
            )}
          </div>
        </div>
      </ChartPanel>

      <div className="flex flex-col gap-6">
        <ChartPanel
          title="Altitude vs Time"
          headerActions={curves.length > 0 ? altDownload : undefined}
        >
          {curves.length > 0 ? (
            <PlotlyChart
              data={altTraces}
              layout={altitudeLayout}
              config={config}
              height="500px"
              onInitialized={onAltInit}
            />
          ) : altAzError && altAzRequest ? (
            <div className="flex h-[500px] items-center justify-center text-red-400">
              Alt/Az computation failed.
            </div>
          ) : (
            <div className="flex h-[500px] items-center justify-center text-slate-500">
              {computing ? (
                <LoadingSpinner size="lg" />
              ) : (
                <p className="text-center text-sm">
                  Select targets to generate the plots.
                </p>
              )}
            </div>
          )}
        </ChartPanel>
        <ChartPanel
          title="Azimuth vs Time"
          headerActions={curves.length > 0 ? azDownload : undefined}
        >
          {curves.length > 0 ? (
            <PlotlyChart
              data={azTraces}
              layout={azLayout}
              config={config}
              height="500px"
              onInitialized={onAzInit}
            />
          ) : altAzError && altAzRequest ? (
            <div className="flex h-[500px] items-center justify-center text-red-400">
              Alt/Az computation failed.
            </div>
          ) : (
            <div className="flex h-[500px] items-center justify-center text-slate-500">
              {computing ? (
                <LoadingSpinner size="lg" />
              ) : (
                <p className="text-center text-sm">
                  Azimuth curves will appear here after computation.
                </p>
              )}
            </div>
          )}
        </ChartPanel>
      </div>
    </PageContainer>
  );
}

export default AltAz;
