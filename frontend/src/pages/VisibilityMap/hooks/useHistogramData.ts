/**
 * useHistogramData - Memoized hook for histogram data transformation.
 * Isolates expensive computations and ensures stable references.
 */
import { useMemo } from 'react';
import type { Data } from 'plotly.js';

export interface HistogramBin {
  bin_start_unix: number;
  bin_end_unix: number;
  visible_count: number;
}

export interface HistogramTraceData {
  x: Date[];
  y: number[];
  width: number[];
  type: 'bar';
  name: string;
  marker: {
    color: number[];
    colorscale: string;
    colorbar: {
      title: string;
      titlefont: { color: string };
      tickfont: { color: string };
      bgcolor: string;
      bordercolor: string;
      borderwidth: number;
      outlinecolor: string;
      outlinewidth: number;
    };
    line: { width: number; color: string };
  };
  hovertemplate: string;
}

/**
 * Transforms raw histogram bin data into Plotly-compatible trace data.
 * Memoized to prevent recalculation unless histogram data changes.
 */
export function useHistogramData(histogramData: HistogramBin[] | undefined): Data {
  return useMemo(() => {
    if (!histogramData || histogramData.length === 0) {
      return {
        x: [],
        y: [],
        type: 'bar' as const,
        name: 'Visible Targets',
      };
    }

    const binStarts = histogramData.map((bin) => new Date(bin.bin_start_unix * 1000));
    const binCounts = histogramData.map((bin) => bin.visible_count);
    const binWidths = histogramData.map(
      (bin) => (bin.bin_end_unix - bin.bin_start_unix) * 1000
    );

    return {
      x: binStarts,
      y: binCounts,
      width: binWidths,
      type: 'bar' as const,
      name: 'Visible Targets',
      marker: {
        color: binCounts,
        colorscale: 'Viridis',
        colorbar: {
          title: 'Number of<br>Visible Blocks',
          titlefont: { color: '#e2e8f0' },
          tickfont: { color: '#cbd5e1' },
          bgcolor: '#1e293b',
          bordercolor: '#334155',
          borderwidth: 1,
          outlinecolor: '#334155',
          outlinewidth: 1,
        },
        line: { width: 0.5, color: 'rgba(255, 255, 255, 0.15)' },
      },
      hovertemplate: '<b>%{y} visible blocks</b><br>Time: %{x|%Y-%m-%d %H:%M}<br><extra></extra>',
    };
  }, [histogramData]);
}
