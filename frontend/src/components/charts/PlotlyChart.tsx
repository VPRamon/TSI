/**
 * Reusable Plotly chart wrapper component.
 * Provides consistent styling and responsive behavior.
 * Memoized to prevent unnecessary re-renders.
 */
import { memo } from 'react';
import Plot from 'react-plotly.js';
import type { Data, Layout, Config } from 'plotly.js';

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
}: PlotlyChartProps) {
  return (
    <div
      className={`w-full ${className}`}
      style={{ height }}
      role="img"
      aria-label={ariaLabel}
    >
      <Plot
        data={data}
        layout={layout}
        config={config}
        style={{ width: '100%', height: '100%' }}
        useResizeHandler
      />
    </div>
  );
});

export default PlotlyChart;
