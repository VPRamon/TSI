/**
 * OpportunitiesHistogram - Displays the visibility histogram chart.
 *
 * This page uses a lightweight React-rendered histogram instead of Plotly.
 * The selection flow updates the dataset frequently, and a simple bar chart
 * is more stable than the previous Plotly re-render path.
 */
import { memo, useMemo } from 'react';
import { ChartPanel } from '@/components';
import type { HistogramBin } from '../hooks/useHistogramData';

interface OpportunitiesHistogramProps {
  histogramData: HistogramBin[] | undefined;
  isLoading?: boolean;
}

const OpportunitiesHistogram = memo(function OpportunitiesHistogram({
  histogramData,
  isLoading = false,
}: OpportunitiesHistogramProps) {
  const bins = useMemo(() => histogramData ?? [], [histogramData]);

  const maxVisibleCount = useMemo(() => {
    if (bins.length === 0) {
      return 1;
    }
    return Math.max(...bins.map((bin) => bin.visible_count), 1);
  }, [bins]);

  const yAxisTicks = useMemo(() => {
    return [1, 0.75, 0.5, 0.25, 0].map((ratio) => ({
      ratio,
      value: Math.round(maxVisibleCount * ratio),
    }));
  }, [maxVisibleCount]);

  const xAxisTicks = useMemo(() => {
    if (bins.length === 0) {
      return [];
    }

    const indexes = Array.from(
      new Set([0, Math.floor((bins.length - 1) / 3), Math.floor(((bins.length - 1) * 2) / 3), bins.length - 1])
    );

    return indexes.map((index) => ({
      index,
      label: formatHistogramTick(bins[index].bin_start_unix),
    }));
  }, [bins]);

  return (
    <ChartPanel title="Visibility Histogram">
      <div
        className={`transition-opacity duration-150 ${isLoading ? 'opacity-70' : 'opacity-100'}`}
      >
        {bins.length === 0 ? (
          <div className="flex h-[550px] items-center justify-center rounded-lg border border-dashed border-slate-700 text-sm text-slate-400">
            {isLoading ? 'Loading visibility histogram...' : 'No visibility data for the current filters'}
          </div>
        ) : (
          <div className="grid h-[550px] grid-cols-[3rem_minmax(0,1fr)] gap-4">
            <div className="relative hidden h-full sm:block">
              {yAxisTicks.map((tick) => (
                <div
                  key={`${tick.ratio}-${tick.value}`}
                  className="absolute left-0 right-0 flex -translate-y-1/2 items-center justify-end"
                  style={{ top: `${tick.ratio * 100}%` }}
                >
                  <span className="text-xs tabular-nums text-slate-500">{tick.value}</span>
                </div>
              ))}
            </div>

            <div className="flex min-w-0 flex-col">
              <div className="relative flex-1 overflow-hidden rounded-lg border border-slate-700 bg-slate-950/35 px-2 pb-2 pt-4">
                {yAxisTicks.map((tick) => (
                  <div
                    key={`grid-${tick.ratio}`}
                    className="pointer-events-none absolute inset-x-0 border-t border-slate-700/50"
                    style={{ top: `${tick.ratio * 100}%` }}
                  />
                ))}

                <div className="flex h-full items-end gap-[2px]">
                  {bins.map((bin) => {
                    const heightPercent = (bin.visible_count / maxVisibleCount) * 100;
                    const intensity = 0.35 + (bin.visible_count / maxVisibleCount) * 0.65;

                    return (
                      <div
                        key={`${bin.bin_start_unix}-${bin.bin_end_unix}`}
                        className="group relative flex-1"
                        title={`${formatHistogramRange(bin.bin_start_unix, bin.bin_end_unix)}\n${bin.visible_count} visible block${bin.visible_count === 1 ? '' : 's'}`}
                      >
                        <div
                          className="w-full rounded-t-sm border border-sky-300/20 bg-sky-400/80 transition-[filter,transform] duration-150 group-hover:-translate-y-0.5 group-hover:brightness-110"
                          style={{
                            height: `${Math.max(heightPercent, 1)}%`,
                            opacity: intensity,
                          }}
                        />
                      </div>
                    );
                  })}
                </div>
              </div>

              <div className="mt-3 flex items-start justify-between gap-2 text-xs text-slate-500">
                {xAxisTicks.map((tick) => (
                  <span key={`${tick.index}-${tick.label}`} className="min-w-0 flex-1 text-center">
                    {tick.label}
                  </span>
                ))}
              </div>
              <div className="mt-2 flex items-center justify-between text-xs text-slate-500">
                <span>Observation Period (UTC)</span>
                <span>Number of Visible Blocks</span>
              </div>
            </div>
          </div>
        )}
      </div>
    </ChartPanel>
  );
});

function formatHistogramTick(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
    timeZone: 'UTC',
  });
}

function formatHistogramRange(startUnix: number, endUnix: number): string {
  const formatter = new Intl.DateTimeFormat('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
    timeZone: 'UTC',
  });

  return `${formatter.format(new Date(startUnix * 1000))} - ${formatter.format(new Date(endUnix * 1000))}`;
}

export default OpportunitiesHistogram;
