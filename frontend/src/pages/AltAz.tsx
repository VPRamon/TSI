/**
 * AltAz page - Altitude/Azimuth analysis for observation targets.
 *
 * Uses siderust-js WASM to compute altitude curves for selected targets
 * as seen from a configurable observatory location over a custom UTC interval.
 */
import { useState, useMemo, useCallback, useEffect, useRef } from 'react';
import { useParams } from 'react-router-dom';
import { useSkyMap, usePlotlyTheme } from '@/hooks';
import { useSiderust } from '@/hooks/useSiderust';
import { loadSiderust, type SiderustModules } from '@/lib/siderust';
import {
  LoadingSpinner,
  ErrorMessage,
  PlotlyChart,
  PageHeader,
  PageContainer,
  ChartPanel,
  ToolbarRow,
} from '@/components';
import type { LightweightBlock } from '@/api/types';

// ─── Observatory presets ─────────────────────────────────────────────

interface ObservatoryPreset {
  label: string;
  lon: number;
  lat: number;
  height: number;
  /** Build an Observer using the canonical siderust factory. */
  factory: (siderust: SiderustModules['siderust']) => InstanceType<SiderustModules['siderust']['Observer']>;
}

const OBSERVATORIES: ObservatoryPreset[] = [
  { label: 'Roque de los Muchachos (ORM)', lon: -17.879, lat: 28.764, height: 2396, factory: (s) => s.Observer.roqueDeLasMuchachos() },
  { label: 'Paranal (VLT)', lon: -70.404, lat: -24.627, height: 2635, factory: (s) => s.Observer.elParanal() },
  { label: 'Mauna Kea', lon: -155.468, lat: 19.826, height: 4205, factory: (s) => s.Observer.maunaKea() },
  { label: 'La Silla', lon: -70.730, lat: -29.257, height: 2347, factory: (s) => s.Observer.laSilla() },
];

// ─── Computation helpers ─────────────────────────────────────────────

interface AltitudeCurve {
  blockId: string;
  blockName: string;
  priority: number;
  times: Date[];
  altitudes: number[];
  azimuths: number[];
}

const DEFAULT_PERIOD_HOURS = 24;
const TARGET_SAMPLE_INTERVAL_MINUTES = 10;

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

async function computeAltAzCurves(
  targets: LightweightBlock[],
  obsLon: number,
  obsLat: number,
  obsHeight: number,
  startDate: Date,
  endDate: Date,
  presetFactory?: ObservatoryPreset['factory'],
): Promise<AltitudeCurve[]> {
  const { siderust, tempoch, qtty } = await loadSiderust();

  const observer = presetFactory
    ? presetFactory(siderust)
    : new siderust.Observer(
        new qtty.Quantity(obsLon, 'Degree'),
        new qtty.Quantity(obsLat, 'Degree'),
        new qtty.Quantity(obsHeight, 'Meter'),
      );

  const windowDurationMs = endDate.getTime() - startDate.getTime();
  const windowDurationMinutes = windowDurationMs / 60000;
  const startMjd = tempoch.mjdFromDate(startDate);
  const sampleCount = Math.max(1, Math.ceil(windowDurationMinutes / TARGET_SAMPLE_INTERVAL_MINUTES));
  const stepDays = (windowDurationMs / 86400000) / sampleCount;

  const curves: AltitudeCurve[] = [];

  for (const target of targets) {
    const star = new siderust.Star(
      target.original_block_id,
      new qtty.Quantity(1, 'Parsec'),      // placeholder distance
      new qtty.Quantity(1, 'SolarMass'),   // placeholder mass
      new qtty.Quantity(1, 'NominalSolarRadius'), // placeholder radius
      new qtty.Quantity(1, 'SolarLuminosity'),     // placeholder luminosity
      new qtty.Quantity(target.target_ra_deg, 'Degree'),
      new qtty.Quantity(target.target_dec_deg, 'Degree'),
    );

    const times: Date[] = [];
    const altitudes: number[] = [];
    const azimuths: number[] = [];

    for (let i = 0; i <= sampleCount; i++) {
      const mjd = new tempoch.ModifiedJulianDate(startMjd + i * stepDays);
      const alt = siderust.starAltitudeAt(star, observer, mjd);
      const az = siderust.starAzimuthAt(star, observer, mjd);
      times.push(mjd.toDate());
      altitudes.push(alt.to('Degree').value);
      azimuths.push(az.to('Degree').value);
    }

    curves.push({
      blockId: target.original_block_id,
      blockName: target.block_name,
      priority: target.priority,
      times,
      altitudes,
      azimuths,
    });
  }

  return curves;
}

// ─── Target selector ─────────────────────────────────────────────────

const MAX_DISPLAY = 200;
const MAX_TARGETS = 10;

// Stable colors for altitude traces
const TRACE_COLORS = [
  '#3b82f6', '#ef4444', '#22c55e', '#f59e0b', '#8b5cf6',
  '#ec4899', '#06b6d4', '#f97316', '#14b8a6', '#a855f7',
];

// ─── Main component ──────────────────────────────────────────────────

function AltAz() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data: skyData, isLoading: skyLoading, error: skyError } = useSkyMap(id);
  const { status: wasmStatus, error: wasmError } = useSiderust();

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
    [useCustom, customLon, customLat, customHeight, presetIndex],
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
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [search, setSearch] = useState('');

  // Computation state
  const [curves, setCurves] = useState<AltitudeCurve[]>([]);
  const [computing, setComputing] = useState(false);

  const startDate = useMemo(() => parseDateTimeLocalUtc(startTimeStr), [startTimeStr]);
  const endDate = useMemo(() => parseDateTimeLocalUtc(endTimeStr), [endTimeStr]);
  const hasValidWindow = startDate.getTime() < endDate.getTime();

  // Filter blocks for display
  const filteredBlocks = useMemo(() => {
    if (!skyData) return [];
    let blocks = skyData.blocks;
    if (search) {
      const q = search.toLowerCase();
      blocks = blocks.filter(
        (b) =>
          b.original_block_id.toLowerCase().includes(q) ||
          b.block_name.toLowerCase().includes(q),
      );
    }
    return blocks.slice(0, MAX_DISPLAY);
  }, [skyData, search]);

  const toggleTarget = useCallback((blockId: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(blockId)) {
        next.delete(blockId);
      } else if (next.size < MAX_TARGETS) {
        next.add(blockId);
      }
      return next;
    });
  }, []);

  const clearSelection = useCallback(() => setSelectedIds(new Set()), []);

  // Auto-compute altitude/azimuth curves on any input change
  const computeIdRef = useRef(0);
  useEffect(() => {
    if (!skyData || selectedIds.size === 0 || wasmStatus !== 'ready' || !hasValidWindow) {
      setCurves([]);
      return;
    }
    const id = ++computeIdRef.current;
    setComputing(true);
    const targets = skyData.blocks.filter((b) => selectedIds.has(b.original_block_id));
    const factory = !useCustom ? OBSERVATORIES[presetIndex].factory : undefined;
    computeAltAzCurves(
      targets, observatory.lon, observatory.lat, observatory.height, startDate, endDate, factory,
    ).then((result) => {
      if (id === computeIdRef.current) setCurves(result);
    }).finally(() => {
      if (id === computeIdRef.current) setComputing(false);
    });
  }, [skyData, selectedIds, wasmStatus, observatory, startDate, endDate, hasValidWindow, useCustom, presetIndex]);

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

  // Extend altitude layout with horizon line
  const altitudeLayout = useMemo(() => ({
    ...altLayout,
    shapes: [
      {
        type: 'line' as const,
        xref: 'paper' as const,
        yref: 'y' as const,
        x0: 0, x1: 1,
        y0: 0, y1: 0,
        line: { color: '#ef4444', width: 1, dash: 'dash' as const },
      },
    ],
  }), [altLayout]);

  // Build Plotly traces
  const altTraces: Plotly.Data[] = useMemo(() =>
    curves.map((curve, i) => {
      const label = curve.blockName ? `${curve.blockName} (${curve.blockId})` : curve.blockId;
      return {
        type: 'scatter' as const,
        mode: 'lines' as const,
        name: `${label} p=${curve.priority.toFixed(1)}`,
        x: curve.times,
        y: curve.altitudes,
        line: { color: TRACE_COLORS[i % TRACE_COLORS.length], width: 2 },
        hovertemplate: `<b>${label}</b><br>Alt: %{y:.1f}°<br>%{x|%H:%M UTC}<extra></extra>`,
      };
    }),
  [curves]);

  const azTraces: Plotly.Data[] = useMemo(() =>
    curves.map((curve, i) => {
      const label = curve.blockName ? `${curve.blockName} (${curve.blockId})` : curve.blockId;
      return {
        type: 'scatter' as const,
        mode: 'lines' as const,
        name: `${label} p=${curve.priority.toFixed(1)}`,
        x: curve.times,
        y: curve.azimuths,
        line: { color: TRACE_COLORS[i % TRACE_COLORS.length], width: 2 },
        hovertemplate: `<b>${label}</b><br>Az: %{y:.1f}°<br>%{x|%H:%M UTC}<extra></extra>`,
      };
    }),
  [curves]);

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
              <option key={obs.label} value={String(i)}>{obs.label}</option>
            ))}
            <option value="custom">Custom Location</option>
          </select>
          <p className="mt-1.5 text-xs text-slate-500">
            {observatory.lat.toFixed(3)}° N, {observatory.lon.toFixed(3)}° E, {observatory.height} m
          </p>
        </div>

        <div>
          <label className="mb-1.5 block text-xs font-medium text-slate-400">Start Time (UTC)</label>
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

        {(!hasValidWindow || wasmStatus === 'error') && (
          <div className="sm:col-span-2 lg:col-span-3">
            {!hasValidWindow && (
              <p className="text-xs text-red-400">End time must be later than start time.</p>
            )}
            {wasmStatus === 'error' && (
              <p className="text-xs text-red-400">WASM error: {wasmError}</p>
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

      <ChartPanel title={`Targets (${selectedIds.size}/${MAX_TARGETS})`}>
        <div className="flex flex-col gap-3">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <input
              type="text"
              placeholder="Search targets..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="min-w-[240px] flex-1 rounded-md border border-slate-600 bg-slate-700 px-3 py-1.5 text-sm text-white placeholder-slate-500 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            />
            {selectedIds.size > 0 && (
              <button
                onClick={clearSelection}
                className="rounded px-2 py-1 text-xs text-slate-400 hover:bg-slate-700 hover:text-slate-200"
              >
                Clear Selection
              </button>
            )}
          </div>
          <div className="scrollbar-thin max-h-48 overflow-y-auto rounded border border-slate-700 bg-slate-800/50">
            {filteredBlocks.map((block) => {
              const isSelected = selectedIds.has(block.original_block_id);
              return (
                <button
                  key={block.original_block_id}
                  onClick={() => toggleTarget(block.original_block_id)}
                  className={`flex w-full items-start justify-between gap-2 px-3 py-2 text-left text-xs transition-colors ${
                    isSelected
                      ? 'bg-primary-600/20 text-primary-300'
                      : 'text-slate-300 hover:bg-slate-700/50'
                  }`}
                >
                  <span className="min-w-0 flex-1">
                    <span className="block truncate font-medium">{block.original_block_id}</span>
                    {block.block_name && (
                      <span className="block truncate text-slate-400">{block.block_name}</span>
                    )}
                    <span className="block text-slate-500">
                      RA {block.target_ra_deg.toFixed(2)}° / Dec {block.target_dec_deg.toFixed(2)}°
                    </span>
                  </span>
                  <span className="shrink-0 text-slate-500">p={block.priority.toFixed(1)}</span>
                </button>
              );
            })}
            {filteredBlocks.length === 0 && (
              <p className="px-3 py-2 text-xs text-slate-500">No targets found</p>
            )}
          </div>
        </div>
      </ChartPanel>

      <div className="flex flex-col gap-6">
        <ChartPanel title="Altitude vs Time">
          {curves.length > 0 ? (
            <PlotlyChart
              data={altTraces}
              layout={altitudeLayout}
              config={config}
              height="500px"
            />
          ) : (
            <div className="flex h-[500px] items-center justify-center text-slate-500">
              {computing ? (
                <LoadingSpinner size="lg" />
              ) : (
                <p className="text-center text-sm">
                  Select targets and click <strong>Compute Alt/Az</strong> to generate the plots
                </p>
              )}
            </div>
          )}
        </ChartPanel>
        <ChartPanel title="Azimuth vs Time">
          {curves.length > 0 ? (
            <PlotlyChart
              data={azTraces}
              layout={azLayout}
              config={config}
              height="500px"
            />
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
