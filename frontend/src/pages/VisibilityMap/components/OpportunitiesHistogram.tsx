/**
 * OpportunitiesHistogram - Displays the visibility histogram chart.
 * 
 * Memoized component that only re-renders when histogram data changes.
 * Uses useDeferredValue for smooth transitions during rapid updates.
 */
import { memo, useMemo, useDeferredValue } from 'react';
import { PlotlyChart, ChartPanel } from '@/components';
import { usePlotlyTheme } from '@/hooks/usePlotlyTheme';
import { useHistogramData, type HistogramBin } from '../hooks/useHistogramData';
import { useRemountDetector, useRenderCounter } from '../hooks/useRemountDetector';

interface OpportunitiesHistogramProps {
  histogramData: HistogramBin[] | undefined;
  isLoading?: boolean;
}

/**
 * OpportunitiesHistogram renders the visibility histogram.
 * Uses deferred value to keep UI responsive during rapid filter changes.
 */
const OpportunitiesHistogram = memo(function OpportunitiesHistogram({
  histogramData,
  isLoading = false,
}: OpportunitiesHistogramProps) {
  // DEV: Remount/render detection
  useRemountDetector('OpportunitiesHistogram');
  useRenderCounter('OpportunitiesHistogram');

  // Defer the histogram data to keep controls responsive during updates
  const deferredHistogramData = useDeferredValue(histogramData);
  
  // Transform data using memoized hook
  const histogramTrace = useHistogramData(deferredHistogramData);
  
  // Get theme (stable reference from hook)
  const plotlyTheme = usePlotlyTheme();

  // Memoized layout - only recalculates when theme changes
  const plotlyLayout = useMemo(
    () => ({
      ...plotlyTheme,
      xaxis: {
        title: { text: 'Observation Period (UTC)' },
        showgrid: true,
        gridcolor: 'rgba(100, 100, 100, 0.3)',
        type: 'date' as const,
      },
      yaxis: {
        title: { text: 'Number of Visible Blocks' },
        showgrid: true,
        gridcolor: 'rgba(100, 100, 100, 0.3)',
      },
      bargap: 0,
      height: 550,
      hovermode: 'x unified' as const,
      showlegend: false,
    }),
    [plotlyTheme]
  );

  // Memoize chart data array to prevent new array reference each render
  const chartData = useMemo(() => [histogramTrace], [histogramTrace]);

  // Memoize config to prevent reference changes
  const chartConfig = useMemo(
    () => ({ displayModeBar: true, responsive: true }),
    []
  );

  // Show subtle loading indicator during deferred updates
  const isPending = histogramData !== deferredHistogramData;

  return (
    <ChartPanel title="Visibility Histogram">
      <div className={`transition-opacity duration-150 ${isPending || isLoading ? 'opacity-70' : 'opacity-100'}`}>
        <PlotlyChart
          data={chartData}
          layout={plotlyLayout}
          config={chartConfig}
        />
      </div>
    </ChartPanel>
  );
});

export default OpportunitiesHistogram;
