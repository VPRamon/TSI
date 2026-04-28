/**
 * Reusable Plotly chart wrapper component.
 * Provides consistent styling and responsive behavior.
 * Memoized to prevent unnecessary re-renders.
 */
import { memo, useMemo } from 'react';
import createPlotlyComponent from 'react-plotly.js/factory';
import Plotly from 'plotly.js-dist-min';
import type { Data, Layout, Config, PlotMouseEvent, PlotSelectionEvent } from 'plotly.js-dist-min';
import { sanitizeImageFilename } from '@/lib/imageExport';

const Plot = createPlotlyComponent(Plotly);

export interface PlotlyChartProps {
  /** Chart data traces */
  data: Data[];
  /** Chart layout configuration */
  layout: Partial<Layout>;
  /** Chart config options */
  config?: Partial<Config>;
  /** Chart height (default: 400px) */
  height?: string;
  /** Additional CSS class */
  className?: string;
  /** Accessible label for the chart */
  ariaLabel?: string;
  /**
   * Called once after Plotly initialises the chart.
   * Receives the raw figure object and the underlying DOM element.
   * Use with usePlotlyDownload to wire up the header download button.
   */
  onInitialized?: (figure: unknown, graphDiv: HTMLElement) => void;
  /** Plotly box/lasso selection event. */
  onSelected?: (event: PlotSelectionEvent | undefined) => void;
  /** Fired when the current selection is cleared (e.g. double-click). */
  onDeselect?: () => void;
  /** Plotly point click event. */
  onClick?: (event: PlotMouseEvent) => void;
}

/**
 * Reusable Plotly chart component with consistent styling.
 * Memoized to prevent expensive re-renders when parent updates.
 *
 * @example
 * <PlotlyChart
 *   data={chartData}
 *   layout={layout}
 *   height="500px"
 *   ariaLabel="Sky map showing scheduled observations"
 * />
 */
const PlotlyChart = memo(function PlotlyChart({
  data,
  layout,
  config = { responsive: true },
  height = '400px',
  className = '',
  ariaLabel,
  onInitialized,
  onSelected,
  onDeselect,
  onClick,
}: PlotlyChartProps) {
  const exportFilename = useMemo(() => {
    const titleValue = layout.title;
    const title =
      typeof titleValue === 'string'
        ? titleValue
        : titleValue &&
            typeof titleValue === 'object' &&
            'text' in titleValue &&
            typeof titleValue.text === 'string'
          ? titleValue.text
          : ariaLabel;

    return sanitizeImageFilename(title ?? 'tsi-plot');
  }, [ariaLabel, layout.title]);

  const mergedConfig = useMemo(
    (): Partial<Config> => ({
      responsive: true,
      displaylogo: false,
      ...config,
      toImageButtonOptions: {
        format: 'png',
        filename: exportFilename,
        scale: 2,
        ...config.toImageButtonOptions,
      },
    }),
    [config, exportFilename]
  );

  return (
    <div className={`w-full ${className}`} style={{ height }} role="img" aria-label={ariaLabel}>
      <Plot
        data={data}
        layout={layout}
        config={mergedConfig}
        style={{ width: '100%', height: '100%' }}
        useResizeHandler
        onInitialized={onInitialized}
        onSelected={onSelected}
        onDeselect={onDeselect}
        onClick={onClick}
      />
    </div>
  );
});

export default PlotlyChart;
