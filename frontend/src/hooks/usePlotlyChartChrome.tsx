/**
 * usePlotlyChartChrome — bundles the recurring chart-panel "chrome"
 * (download button, fullscreen toggle, '?' help popover) into a single
 * hook.
 *
 * Returns:
 *   - `config`       Plotly config patch that re-enables the modebar
 *                    with **only** the camera (download) button and a
 *                    custom fullscreen icon. All other Plotly tools are
 *                    removed so the bar can't overflow on small charts.
 *   - `onInitialized` Pass to <PlotlyChart onInitialized={...} /> so we
 *                    can capture the graph div for fullscreen + image
 *                    export.
 *   - `helpButton`   ReactNode to drop into ChartPanel `headerActions`
 *                    (omitted when no help content is provided).
 *   - `fullscreenButton` ReactNode that opens the overlay; identical
 *                    behaviour to the modebar icon, exposed in the
 *                    header for discoverability.
 *   - `fullscreenOverlay` Renders the overlay when fullscreen is open;
 *                    place it once next to the chart.
 */
import { useCallback, useMemo, useState, type ReactNode } from 'react';
import Plotly from 'plotly.js-dist-min';
import type { Config } from 'plotly.js-dist-min';
import HelpPopover, { type HelpContent } from '@/components/charts/HelpPopover';
import ChartFullscreenOverlay from '@/components/charts/ChartFullscreenOverlay';
import {
  downloadPngDataUrl,
  downloadSvgString,
  sanitizeImageFilename,
} from '@/lib/imageExport';

const HEADER_BTN_CLASS =
  'inline-flex h-7 items-center gap-1 rounded-md border border-slate-600 bg-slate-800/70 px-2 text-xs font-medium text-slate-300 transition-colors hover:bg-slate-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-900';

// Inline expand-arrows icon used for the modebar custom button and the
// header fullscreen affordance. Keeps the bundle free of an icon set.
const EXPAND_PATH =
  'M3 10V3h7v2H6.41L11 9.59 9.59 11 5 6.41V10H3zm18 4v7h-7v-2h3.59L13 14.41 14.41 13 19 17.59V14h2z';

export interface UsePlotlyChartChromeOptions {
  /** Used as the modebar download filename and fullscreen overlay heading. */
  label: string;
  /** Optional content for the '?' popover. */
  help?: HelpContent;
  /** Extra Plotly config keys to merge after our defaults. */
  configOverrides?: Partial<Config>;
}

export interface PlotlyChartChrome {
  config: Partial<Config>;
  onInitialized: (figure: unknown, graphDiv: HTMLElement) => void;
  helpButton: ReactNode | null;
  fullscreenButton: ReactNode;
  fullscreenOverlay: ReactNode;
  /** "Download PNG" button (header). */
  downloadPngButton: ReactNode;
  /** "Download SVG" button (header). */
  downloadSvgButton: ReactNode;
  /** Combined header actions (PNG + SVG + fullscreen + help) ready to drop in. */
  headerActions: ReactNode;
}

export function usePlotlyChartChrome(options: UsePlotlyChartChromeOptions): PlotlyChartChrome {
  const { label, help, configOverrides } = options;
  const [graphDiv, setGraphDiv] = useState<HTMLElement | null>(null);
  const [fullscreen, setFullscreen] = useState(false);

  const onInitialized = useCallback((_figure: unknown, gd: HTMLElement) => {
    setGraphDiv(gd);
  }, []);

  const openFullscreen = useCallback(() => {
    if (graphDiv) setFullscreen(true);
  }, [graphDiv]);

  const closeFullscreen = useCallback(() => setFullscreen(false), []);

  const filename = useMemo(() => sanitizeImageFilename(label), [label]);

  const config = useMemo<Partial<Config>>(() => {
    const base: Partial<Config> = {
      responsive: true,
      displaylogo: false,
      displayModeBar: 'hover',
      modeBarButtonsToRemove: [
        'zoom2d',
        'pan2d',
        'select2d',
        'lasso2d',
        'zoomIn2d',
        'zoomOut2d',
        'autoScale2d',
        'resetScale2d',
        'hoverClosestCartesian',
        'hoverCompareCartesian',
        'toggleSpikelines',
        'zoom3d',
        'pan3d',
        'orbitRotation',
        'tableRotation',
        'resetCameraDefault3d',
        'resetCameraLastSave3d',
        'hoverClosest3d',
        'zoomInGeo',
        'zoomOutGeo',
        'resetGeo',
        'hoverClosestGeo',
        'sendDataToCloud',
      ],
      modeBarButtonsToAdd: [
        {
          name: 'fullscreen',
          title: 'View fullscreen',
          icon: { width: 24, height: 24, path: EXPAND_PATH },
          click: () => openFullscreen(),
        },
      ],
      toImageButtonOptions: {
        format: 'png',
        filename,
        scale: 2,
      },
    };
    return { ...base, ...configOverrides };
  }, [filename, openFullscreen, configOverrides]);

  const downloadImage = useCallback(
    async (format: 'png' | 'svg') => {
      if (!graphDiv || typeof Plotly.toImage !== 'function') return;
      try {
        const dataUrl = await Plotly.toImage(graphDiv, {
          format,
          scale: format === 'png' ? 2 : 1,
          width: graphDiv.offsetWidth || 800,
          height: graphDiv.offsetHeight || 400,
        });
        if (format === 'png') {
          downloadPngDataUrl(dataUrl, filename);
          return;
        }
        // Plotly returns an SVG payload as `data:image/svg+xml,<encoded>`.
        const commaIdx = dataUrl.indexOf(',');
        const payload = commaIdx >= 0 ? dataUrl.slice(commaIdx + 1) : dataUrl;
        const svg = decodeURIComponent(payload);
        downloadSvgString(svg, filename);
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error(`[Plotly export ${format.toUpperCase()}]`, err);
      }
    },
    [graphDiv, filename],
  );

  const downloadPngButton = (
    <button
      type="button"
      onClick={() => void downloadImage('png')}
      disabled={!graphDiv}
      className={HEADER_BTN_CLASS}
      title="Download chart as PNG"
      aria-label={`Download ${label} as PNG`}
    >
      PNG
    </button>
  );

  const downloadSvgButton = (
    <button
      type="button"
      onClick={() => void downloadImage('svg')}
      disabled={!graphDiv}
      className={HEADER_BTN_CLASS}
      title="Download chart as SVG"
      aria-label={`Download ${label} as SVG`}
    >
      SVG
    </button>
  );

  const helpButton = help ? <HelpPopover content={help} ariaLabel={`Help: ${label}`} /> : null;

  const fullscreenButton = (
    <button
      type="button"
      onClick={openFullscreen}
      disabled={!graphDiv}
      className={HEADER_BTN_CLASS}
      title="View fullscreen"
      aria-label={`View ${label} fullscreen`}
    >
      <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true">
        <path fill="currentColor" d={EXPAND_PATH} />
      </svg>
      Fullscreen
    </button>
  );

  const fullscreenOverlay =
    fullscreen && graphDiv ? (
      <ChartFullscreenOverlay graphDiv={graphDiv} title={label} onClose={closeFullscreen} />
    ) : null;

  const headerActions = (
    <div className="flex items-center gap-2">
      {downloadPngButton}
      {downloadSvgButton}
      {fullscreenButton}
      {helpButton}
    </div>
  );

  return {
    config,
    onInitialized,
    helpButton,
    fullscreenButton,
    fullscreenOverlay,
    downloadPngButton,
    downloadSvgButton,
    headerActions,
  };
}

export default usePlotlyChartChrome;
