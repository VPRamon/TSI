/**
 * Reusable Plotly chart wrapper component.
 * Provides consistent styling and responsive behavior.
 * Memoized to prevent unnecessary re-renders.
 */
import { lazy, memo, Suspense, useMemo, type ComponentType } from 'react';
import createPlotlyComponent from 'react-plotly.js/factory';
import type { Data, Layout, Config, PlotMouseEvent, PlotSelectionEvent } from 'plotly.js-dist-min';
import { sanitizeImageFilename } from '@/lib/imageExport';
import { loadPlotly, type PlotlyBundle } from './plotlyRegistry';

interface InternalPlotProps {
  data: Data[];
  layout: Partial<Layout>;
  config: Partial<Config>;
  style: React.CSSProperties;
  useResizeHandler: boolean;
  onInitialized?: (figure: unknown, graphDiv: HTMLElement) => void;
  onSelected?: (event: PlotSelectionEvent | undefined) => void;
  onDeselect?: () => void;
  onClick?: (event: PlotMouseEvent) => void;
}

const lazyComponentCache: Partial<Record<PlotlyBundle, ComponentType<InternalPlotProps>>> = {};

function getLazyPlot(bundle: PlotlyBundle): ComponentType<InternalPlotProps> {
  const cached = lazyComponentCache[bundle];
  if (cached) return cached;
  const Component = lazy(async () => {
    const Plotly = await loadPlotly(bundle);
    const Plot = createPlotlyComponent(Plotly) as ComponentType<InternalPlotProps>;
    return { default: Plot };
  });
  lazyComponentCache[bundle] = Component;
  return Component;
}

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
   * Which Plotly bundle to load. Defaults to `'basic'` (lighter).
   * Use `'full'` for charts that need 3-D, mapbox/geo, or other
   * trace types missing from the basic build.
   */
  bundle?: PlotlyBundle;
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
  bundle = 'basic',
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

  const Plot = getLazyPlot(bundle);
  // Sized placeholder so the layout doesn't shift while the Plotly
  // bundle is fetched on first mount. Reuses neutral chart background
  // styling already in the surrounding ChartPanel.
  const fallback = (
    <div
      className="h-full w-full animate-pulse rounded-md bg-slate-800/40"
      aria-hidden="true"
    />
  );

  return (
    <div className={`w-full ${className}`} style={{ height }} role="img" aria-label={ariaLabel}>
      <Suspense fallback={fallback}>
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
      </Suspense>
    </div>
  );
});

export default PlotlyChart;
