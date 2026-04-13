/**
 * CelestialSkyMap – wraps d3-celestial (loaded globally via /celestial.js) to
 * render an interactive all-sky Aitoff projection with the real Milky Way,
 * equatorial grid, and our observation targets colored by priority bin.
 */
import { useEffect, useRef, useState, useCallback } from 'react';
import type { LightweightBlock, PriorityBinInfo } from '@/api/types';
import { mjdToDate, isValidDate } from '@/constants/dates';

// ── Minimal type declaration for window.Celestial ─────────────────────────────
declare global {
  interface Window {
    d3?: {
      geo?: {
        projection?: unknown;
        zoom?: unknown;
      };
    };
    Celestial?: {
      container?: unknown;
      display: (config: object) => void;
      redraw: () => void;
      clear: () => void;
      add: (cfg: {
        type: string;
        callback?: (error: unknown, json: unknown) => void;
        redraw: () => void;
      }) => void;
      clip: (coords: number[]) => boolean;
      mapProjection: (coords: number[]) => [number, number];
      context: CanvasRenderingContext2D;
      setStyle: (style: {
        stroke?: string;
        fill?: string;
        width?: number;
        opacity?: number;
      }) => void;
      metrics: () => { width: number; height: number };
    };
  }
}

export interface CelestialSkyMapProps {
  blocks: LightweightBlock[];
  bins: PriorityBinInfo[];
  /** Container element ID – must be unique per page. */
  containerId?: string;
  showCoordinateGuide?: boolean;
}

// ── Tooltip helpers ────────────────────────────────────────────────────────────

interface DrawnPoint {
  cx: number;
  cy: number;
  block: LightweightBlock;
}

interface TooltipState {
  block: LightweightBlock;
  x: number;
  y: number;
}

/** Pixel distance (in canvas space) within which hovering snaps to a target. */
const HOVER_RADIUS = 12;

function formatMjd(mjd: number): string {
  if (!Number.isFinite(mjd)) return 'Unknown';
  const date = mjdToDate(mjd);
  return isValidDate(date) ? date.toISOString().replace('T', ' ').slice(0, 16) + ' UTC' : 'Unknown';
}

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0 && m > 0) return `${h}h ${m}m`;
  if (h > 0) return `${h}h`;
  return `${m}m`;
}

/**
 * Renders an interactive d3-celestial sky map.
 * The map is initialized once on mount; subsequent block/bin changes
 * trigger a lightweight redraw without reinitializing the projection.
 */
function CelestialSkyMap({
  blocks,
  bins,
  containerId = 'celestial-map',
  showCoordinateGuide = true,
}: CelestialSkyMapProps) {
  const blocksRef = useRef(blocks);
  const binsRef = useRef(bins);
  const initializedRef = useRef(false);
  const drawnPointsRef = useRef<DrawnPoint[]>([]);
  const [tooltip, setTooltip] = useState<TooltipState | null>(null);

  // Keep refs current on every render so redrawF always uses latest data.
  blocksRef.current = blocks;
  binsRef.current = bins;

  // ── Initialize the map once ─────────────────────────────────────────────
  useEffect(() => {
    const Celestial = window.Celestial;
    if (!Celestial) {
      console.error(
        '[CelestialSkyMap] window.Celestial not found – make sure /celestial.js is loaded.'
      );
      return;
    }
    if (
      typeof window.d3?.geo?.projection !== 'function' ||
      typeof window.d3?.geo?.zoom !== 'function'
    ) {
      console.error(
        '[CelestialSkyMap] d3-celestial dependencies missing – make sure d3 and d3.geo.projection load before /celestial.js.'
      );
      return;
    }

    // Draw our targets on every Celestial redraw event.
    const redrawF = () => {
      const ctx = Celestial.context;
      if (!ctx) return;
      const currentBlocks = blocksRef.current;
      const currentBins = binsRef.current;

      drawnPointsRef.current = [];

      for (const block of currentBlocks) {
        // d3-celestial uses geographic [-180,180] longitude convention.
        const lon = block.target_ra_deg > 180 ? block.target_ra_deg - 360 : block.target_ra_deg;
        const coords = [lon, block.target_dec_deg];

        if (!Celestial.clip(coords)) continue;

        const [cx, cy] = Celestial.mapProjection(coords);
        drawnPointsRef.current.push({ cx, cy, block });
        const bin = currentBins.find(
          (b) => block.priority >= b.min_priority && block.priority <= b.max_priority
        );
        const color = bin?.color ?? '#94a3b8';
        const isScheduled = block.scheduled_period !== null;
        const radius = isScheduled ? 3.5 : 2;

        ctx.beginPath();
        ctx.arc(cx, cy, radius, 0, 2 * Math.PI);
        ctx.closePath();

        if (isScheduled) {
          ctx.globalAlpha = 0.85;
          ctx.fillStyle = color;
          ctx.fill();
        }

        ctx.globalAlpha = isScheduled ? 0.9 : 0.45;
        ctx.strokeStyle = color;
        ctx.lineWidth = isScheduled ? 1.5 : 1;
        ctx.stroke();

        ctx.globalAlpha = 1;
      }
    };

    const celestialConfig = {
      width: 0, // fill parent
      projection: 'aitoff',
      transform: 'equatorial',
      center: null, // RA=0h at center; user can pan/rotate interactively
      background: { fill: '#000000', opacity: 1, stroke: '#000000', width: 1.5 },
      stars: { show: false },
      dsos: { show: false },
      planets: { show: false },
      constellations: { show: false, names: false, lines: false },
      mw: {
        show: true,
        style: { fill: '#ffffff', opacity: 0.15 },
      },
      lines: {
        graticule: {
          show: true,
          stroke: '#334155',
          width: 0.6,
          opacity: 0.8,
          // Keep RA labels on the equator, but pin Dec labels to a single
          // edge so the two coordinate guides do not collide at map center.
          lon: {
            pos: showCoordinateGuide ? ['center'] : [],
            fill: '#94a3b8',
            font: '10px Helvetica, Arial, sans-serif',
          },
          lat: {
            pos: showCoordinateGuide ? [-179.99] : [],
            fill: '#94a3b8',
            font: '10px Helvetica, Arial, sans-serif',
          },
        },
        equatorial: { show: true, stroke: '#475569', width: 1.3, opacity: 0.7 },
        ecliptic: { show: false },
        galactic: { show: false },
        supergalactic: { show: false },
      },
      // Keep celestial assets local so the Milky Way/background renders reliably.
      datapath: '/galacticmap/',
      interactive: true,
      form: false,
      controls: false,
      container: containerId,
    };

    try {
      // d3-celestial stores its DOM container globally. Reset it before init so
      // React StrictMode remounts do not keep writing into a detached node.
      Celestial.container = undefined;
      document.getElementById(containerId)?.replaceChildren();
      Celestial.clear();
      // callback is required by d3-celestial for type:'line' (no-op here – we only use redraw)
      Celestial.add({ type: 'line', callback: () => {}, redraw: redrawF });
      Celestial.display(celestialConfig);
      // Force an immediate draw: d3-celestial only calls redraw() inside async data-load
      // callbacks, so without this the canvas stays blank until network data arrives.
      Celestial.redraw();
      initializedRef.current = true;
    } catch (error) {
      console.error('[CelestialSkyMap] Failed to initialize sky map.', error);
      initializedRef.current = false;
    }

    return () => {
      initializedRef.current = false;
      Celestial.clear();
      Celestial.container = undefined;
      document.getElementById(containerId)?.replaceChildren();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [containerId, showCoordinateGuide]);

  // ── Redraw whenever the filtered block set changes ──────────────────────
  useEffect(() => {
    if (!initializedRef.current) return;
    window.Celestial?.redraw();
  }, [blocks, bins]);

  // ── Hover detection ─────────────────────────────────────────────────────
  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      const canvas = document.querySelector<HTMLCanvasElement>(`#${containerId} canvas`);
      if (!canvas) return;

      const rect = canvas.getBoundingClientRect();
      const scaleX = canvas.width / rect.width;
      const scaleY = canvas.height / rect.height;
      const mouseX = (e.clientX - rect.left) * scaleX;
      const mouseY = (e.clientY - rect.top) * scaleY;

      let closest: DrawnPoint | null = null;
      let minDist = HOVER_RADIUS * scaleX;

      for (const pt of drawnPointsRef.current) {
        const dist = Math.hypot(mouseX - pt.cx, mouseY - pt.cy);
        if (dist < minDist) {
          minDist = dist;
          closest = pt;
        }
      }

      if (closest) {
        const wrapperRect = e.currentTarget.getBoundingClientRect();
        setTooltip({
          block: closest.block,
          x: e.clientX - wrapperRect.left,
          y: e.clientY - wrapperRect.top,
        });
      } else {
        setTooltip(null);
      }
    },
    [containerId]
  );

  const handleMouseLeave = useCallback(() => {
    setTooltip(null);
  }, []);

  // Flip tooltip to the left when it's in the right half of the container
  const tooltipFlip = tooltip && tooltip.x > 400;

  return (
    <div className="relative w-full" onMouseMove={handleMouseMove} onMouseLeave={handleMouseLeave}>
      <div id={containerId} className="w-full" style={{ minHeight: '500px' }} />
      {tooltip && (
        <div
          className="pointer-events-none absolute z-10 max-w-xs rounded-lg border border-slate-600 bg-slate-900/95 p-3 text-xs shadow-xl backdrop-blur-sm"
          style={{
            left: tooltipFlip ? undefined : tooltip.x + 12,
            right: tooltipFlip ? `calc(100% - ${tooltip.x}px + 12px)` : undefined,
            top: tooltip.y + 12,
          }}
        >
          <p className="mb-1.5 truncate font-semibold text-slate-100">
            {tooltip.block.original_block_id}
          </p>
          {tooltip.block.block_name && (
            <p className="mb-1.5 truncate text-xs text-slate-400">{tooltip.block.block_name}</p>
          )}
          <div className="space-y-1 text-slate-300">
            <div className="flex justify-between gap-4">
              <span className="text-slate-400">Priority</span>
              <span>
                {tooltip.block.priority.toFixed(2)}
                <span className="ml-1 text-slate-500">({tooltip.block.priority_bin})</span>
              </span>
            </div>
            <div className="flex justify-between gap-4">
              <span className="text-slate-400">Duration</span>
              <span>{formatDuration(tooltip.block.requested_duration_seconds)}</span>
            </div>
            <div className="flex justify-between gap-4">
              <span className="text-slate-400">RA / Dec</span>
              <span>
                {tooltip.block.target_ra_deg.toFixed(2)}° /{' '}
                {tooltip.block.target_dec_deg.toFixed(2)}°
              </span>
            </div>
            {tooltip.block.scheduled_period ? (
              <>
                <div className="mt-1.5 flex items-center gap-1.5 border-t border-slate-700 pt-1.5">
                  <span className="h-1.5 w-1.5 rounded-full bg-emerald-400" />
                  <span className="font-medium text-emerald-400">Scheduled</span>
                </div>
                <div className="flex justify-between gap-4">
                  <span className="text-slate-400">Start</span>
                  <span>{formatMjd(tooltip.block.scheduled_period.start)}</span>
                </div>
                <div className="flex justify-between gap-4">
                  <span className="text-slate-400">End</span>
                  <span>{formatMjd(tooltip.block.scheduled_period.stop)}</span>
                </div>
              </>
            ) : (
              <div className="mt-1.5 flex items-center gap-1.5 border-t border-slate-700 pt-1.5">
                <span className="h-1.5 w-1.5 rounded-full bg-slate-500" />
                <span className="text-slate-500">Unscheduled</span>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

export default CelestialSkyMap;
