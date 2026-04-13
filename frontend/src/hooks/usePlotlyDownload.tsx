/**
 * Hook for adding a "Download PNG" button to Plotly chart panels.
 *
 * Returns an `onInitialized` callback (to wire into PlotlyChart) and a
 * pre-styled `downloadButton` ReactNode to place in a ChartPanel header.
 * Uses Plotly.toImage for high-DPI PNG export (2× scale).
 */
import { useState, useCallback, type ReactNode } from 'react';
import type Plotly from 'plotly.js';
import { sanitizeImageFilename, downloadPngDataUrl } from '@/lib/imageExport';

// Plotly is loaded at runtime by react-plotly.js; access via window to avoid bundling it.
const getPlotly = (): typeof Plotly | undefined =>
  (window as Window & { Plotly?: typeof Plotly }).Plotly;

/** Matches the secondary action button style used across panel headers. */
const BTN_CLASS =
  'rounded-md border border-slate-600 bg-slate-800/70 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-800';

export interface UsePlotlyDownloadResult {
  /** Pass to PlotlyChart's onInitialized prop to register the chart element. */
  onInitialized: (figure: unknown, graphDiv: HTMLElement) => void;
  /** Pre-styled "Download PNG" button to place in a ChartPanel headerActions. */
  downloadButton: ReactNode;
}

/**
 * @param label - Used to derive the PNG filename (spaces → dashes, lowercased).
 */
export function usePlotlyDownload(label: string): UsePlotlyDownloadResult {
  const [graphDiv, setGraphDiv] = useState<HTMLElement | null>(null);

  const onInitialized = useCallback((_figure: unknown, gd: HTMLElement) => {
    setGraphDiv(gd);
  }, []);

  const filename = sanitizeImageFilename(label);

  const handleDownload = useCallback(async () => {
    if (!graphDiv) return;
    const plotly = getPlotly();
    if (!plotly) return;
    try {
      const dataUrl = await plotly.toImage(graphDiv, {
        format: 'png',
        scale: 2,
        width: graphDiv.offsetWidth || 800,
        height: graphDiv.offsetHeight || 400,
      });
      downloadPngDataUrl(dataUrl, filename);
    } catch {
      // Silently ignore export errors (e.g. browser security restrictions).
    }
  }, [graphDiv, filename]);

  const downloadButton = (
    <button type="button" onClick={handleDownload} className={BTN_CLASS} disabled={!graphDiv}>
      Download PNG
    </button>
  );

  return { onInitialized, downloadButton };
}
