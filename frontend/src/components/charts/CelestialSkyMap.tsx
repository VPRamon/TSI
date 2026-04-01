/**
 * CelestialSkyMap – wraps d3-celestial (loaded globally via /celestial.js) to
 * render an interactive all-sky Aitoff projection with the real Milky Way,
 * equatorial grid, and our observation targets colored by priority bin.
 */
import { useEffect, useRef } from 'react';
import type { LightweightBlock, PriorityBinInfo } from '@/api/types';

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
}: CelestialSkyMapProps) {
  const blocksRef = useRef(blocks);
  const binsRef = useRef(bins);
  const initializedRef = useRef(false);

  // Keep refs current on every render so redrawF always uses latest data.
  blocksRef.current = blocks;
  binsRef.current = bins;

  // ── Initialize the map once ─────────────────────────────────────────────
  useEffect(() => {
    const Celestial = window.Celestial;
    if (!Celestial) {
      console.error('[CelestialSkyMap] window.Celestial not found – make sure /celestial.js is loaded.');
      return;
    }
    if (
      typeof window.d3?.geo?.projection !== 'function' ||
      typeof window.d3?.geo?.zoom !== 'function'
    ) {
      console.error(
        '[CelestialSkyMap] d3-celestial dependencies missing – make sure d3 and d3.geo.projection load before /celestial.js.',
      );
      return;
    }

    // Draw our targets on every Celestial redraw event.
    const redrawF = () => {
      const ctx = Celestial.context;
      if (!ctx) return;
      const currentBlocks = blocksRef.current;
      const currentBins = binsRef.current;

      for (const block of currentBlocks) {
        // d3-celestial uses geographic [-180,180] longitude convention.
        const lon = block.target_ra_deg > 180
          ? block.target_ra_deg - 360
          : block.target_ra_deg;
        const coords = [lon, block.target_dec_deg];

        if (!Celestial.clip(coords)) continue;

        const [cx, cy] = Celestial.mapProjection(coords);
        const bin = currentBins.find(
          (b) => block.priority >= b.min_priority && block.priority <= b.max_priority,
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
      width: 0,             // fill parent
      projection: 'aitoff',
      transform: 'equatorial',
      center: null,         // RA=0h at center; user can pan/rotate interactively
      background: { fill: '#000000', opacity: 1, stroke: '#000000', width: 1.5 },
      stars: { show: false },
      dsos:  { show: false },
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
          lon: { pos: [''], fill: '#64748b', font: '10px Helvetica, Arial, sans-serif' },
          lat: { pos: [''], fill: '#64748b', font: '10px Helvetica, Arial, sans-serif' },
        },
        equatorial:    { show: true,  stroke: '#475569', width: 1.3, opacity: 0.7 },
        ecliptic:      { show: false },
        galactic:      { show: false },
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
  }, [containerId]);

  // ── Redraw whenever the filtered block set changes ──────────────────────
  useEffect(() => {
    if (!initializedRef.current) return;
    window.Celestial?.redraw();
  }, [blocks, bins]);

  return (
    <div
      id={containerId}
      className="w-full"
      style={{ minHeight: '500px' }}
    />
  );
}

export default CelestialSkyMap;
