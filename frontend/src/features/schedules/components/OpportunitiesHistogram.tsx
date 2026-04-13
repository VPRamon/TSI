/**
 * OpportunitiesHistogram - Displays the visibility histogram chart.
 *
 * This page uses a lightweight React-rendered histogram instead of Plotly.
 * The selection flow updates the dataset frequently, and a simple bar chart
 * is more stable than the previous Plotly re-render path.
 */
import { memo, useCallback, useMemo } from 'react';
import { ChartPanel } from '@/components';
import type { HistogramBin } from '../hooks/useHistogramData';
import { downloadCanvasAsPng } from '@/lib/imageExport';

const EXPORT_WIDTH = 1600;
const EXPORT_HEIGHT = 900;
const EXPORT_PADDING = {
  top: 72,
  right: 56,
  bottom: 144,
  left: 112,
};
const DOWNLOAD_BUTTON_CLASS =
  'rounded-md border border-slate-600 bg-slate-800/70 px-3 py-1.5 text-xs font-medium text-slate-300 transition-colors hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-800 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:bg-slate-800/70';

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
      new Set([
        0,
        Math.floor((bins.length - 1) / 3),
        Math.floor(((bins.length - 1) * 2) / 3),
        bins.length - 1,
      ])
    );

    return indexes.map((index) => ({
      index,
      label: formatHistogramTick(bins[index].bin_start_unix),
    }));
  }, [bins]);

  const handleDownload = useCallback(() => {
    if (bins.length === 0) return;

    const canvas = document.createElement('canvas');
    canvas.width = EXPORT_WIDTH;
    canvas.height = EXPORT_HEIGHT;

    const context = canvas.getContext('2d');
    if (!context) return;

    const chartLeft = EXPORT_PADDING.left;
    const chartRight = EXPORT_WIDTH - EXPORT_PADDING.right;
    const chartTop = EXPORT_PADDING.top;
    const chartBottom = EXPORT_HEIGHT - EXPORT_PADDING.bottom;
    const chartWidth = chartRight - chartLeft;
    const chartHeight = chartBottom - chartTop;
    const tickLabelFormatter = new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
      timeZone: 'UTC',
    });

    context.fillStyle = '#0f172a';
    context.fillRect(0, 0, EXPORT_WIDTH, EXPORT_HEIGHT);

    context.fillStyle = 'rgba(15, 23, 42, 0.82)';
    context.fillRect(chartLeft, chartTop, chartWidth, chartHeight);

    context.strokeStyle = 'rgba(51, 65, 85, 0.8)';
    context.lineWidth = 1;
    for (const tick of yAxisTicks) {
      const y = chartTop + chartHeight * (1 - tick.ratio);
      context.beginPath();
      context.moveTo(chartLeft, y);
      context.lineTo(chartRight, y);
      context.stroke();
    }

    context.font = '24px system-ui, sans-serif';
    context.fillStyle = '#e2e8f0';
    context.fillText('Visibility Histogram', chartLeft, 36);

    context.font = '18px system-ui, sans-serif';
    context.fillStyle = '#94a3b8';
    context.fillText('Observation Period (UTC)', chartLeft, EXPORT_HEIGHT - 36);

    context.save();
    context.translate(28, chartBottom);
    context.rotate(-Math.PI / 2);
    context.fillText('Number of Visible Blocks', 0, 0);
    context.restore();

    context.font = '18px system-ui, sans-serif';
    context.textAlign = 'right';
    context.textBaseline = 'middle';
    for (const tick of yAxisTicks) {
      const y = chartTop + chartHeight * (1 - tick.ratio);
      context.fillStyle = '#64748b';
      context.fillText(String(tick.value), chartLeft - 16, y);
    }

    const gap = 2;
    const totalGapWidth = gap * Math.max(bins.length - 1, 0);
    const barWidth = Math.max((chartWidth - totalGapWidth) / bins.length, 1);

    context.textAlign = 'center';
    context.textBaseline = 'top';
    bins.forEach((bin, index) => {
      const x = chartLeft + index * (barWidth + gap);
      const barHeight = (bin.visible_count / maxVisibleCount) * chartHeight;
      const y = chartBottom - barHeight;
      const opacity = 0.35 + (bin.visible_count / maxVisibleCount) * 0.65;

      context.fillStyle = `rgba(56, 189, 248, ${opacity.toFixed(3)})`;
      context.fillRect(x, y, barWidth, Math.max(barHeight, bin.visible_count > 0 ? 4 : 0));

      context.strokeStyle = 'rgba(186, 230, 253, 0.18)';
      context.lineWidth = 1;
      context.strokeRect(x, y, barWidth, Math.max(barHeight, bin.visible_count > 0 ? 4 : 0));
    });

    for (const tick of xAxisTicks) {
      const x = chartLeft + tick.index * (barWidth + gap) + barWidth / 2;
      context.fillStyle = '#64748b';
      context.fillText(tick.label, x, chartBottom + 20);
    }

    context.font = '16px system-ui, sans-serif';
    context.fillStyle = '#94a3b8';
    context.textAlign = 'left';
    context.textBaseline = 'bottom';
    const rangeLabel = `${tickLabelFormatter.format(new Date(bins[0].bin_start_unix * 1000))} - ${tickLabelFormatter.format(new Date(bins[bins.length - 1].bin_end_unix * 1000))}`;
    context.fillText(rangeLabel, chartLeft, chartTop - 18);

    downloadCanvasAsPng(canvas, 'visibility-histogram');
  }, [bins, maxVisibleCount, xAxisTicks, yAxisTicks]);

  return (
    <ChartPanel
      title="Visibility Histogram"
      headerActions={
        <button
          type="button"
          onClick={handleDownload}
          disabled={bins.length === 0}
          className={DOWNLOAD_BUTTON_CLASS}
        >
          Download PNG
        </button>
      }
    >
      <div
        className={`transition-opacity duration-150 ${isLoading ? 'opacity-70' : 'opacity-100'}`}
      >
        {bins.length === 0 ? (
          <div className="flex h-[550px] items-center justify-center rounded-lg border border-dashed border-slate-700 text-sm text-slate-400">
            {isLoading
              ? 'Loading visibility histogram...'
              : 'No visibility data for the current filters'}
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
                        className="group relative flex h-full flex-1 items-end"
                        data-testid="visibility-histogram-column"
                        title={`${formatHistogramRange(bin.bin_start_unix, bin.bin_end_unix)}\n${bin.visible_count} visible block${bin.visible_count === 1 ? '' : 's'}`}
                      >
                        <div
                          className="w-full rounded-t-sm border border-sky-300/20 bg-sky-400/80 transition-[filter,transform] duration-150 group-hover:-translate-y-0.5 group-hover:brightness-110"
                          style={{
                            height: `${heightPercent}%`,
                            minHeight: bin.visible_count > 0 ? '2px' : undefined,
                            opacity: intensity,
                          }}
                          data-testid="visibility-histogram-bar"
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
