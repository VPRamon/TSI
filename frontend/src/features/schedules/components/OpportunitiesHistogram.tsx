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
import { useRemountDetector, useRenderCounter } from '@/hooks/useRemountDetector';

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

  // Use consistent theme via shared hook
  const { layout, config } = usePlotlyTheme({
    title: 'Visibility Histogram',
    xAxis: { title: 'Observation Period (UTC)', type: 'date' },
    yAxis: { title: 'Number of Visible Blocks' },
    showLegend: false,
  });

  // Extend themed layout with histogram-specific settings
  const plotlyLayout = useMemo(
    () => ({
      ...layout,
      bargap: 0,
      hovermode: 'x unified' as const,
    }),
    [layout]
  );

  // Memoize chart data array to prevent new array reference each render
  const chartData = useMemo(() => [histogramTrace], [histogramTrace]);

  // Show subtle loading indicator during deferred updates
  const isPending = histogramData !== deferredHistogramData;

  return (
    <ChartPanel title="Visibility Histogram">
      <div className={`transition-opacity duration-150 ${isPending || isLoading ? 'opacity-70' : 'opacity-100'}`}>
        <PlotlyChart
          data={chartData}
          layout={plotlyLayout}
          config={config}
          height="550px"
        />
      </div>
    </ChartPanel>
  );
});

export default OpportunitiesHistogram;
