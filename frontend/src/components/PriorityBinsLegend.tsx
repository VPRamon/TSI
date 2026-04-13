/**
 * PriorityBinsLegend - Compact legend for priority bins.
 * Can be rendered alongside or overlaid on charts.
 */
import { memo } from 'react';
import type { PriorityBinInfo } from '@/api/types';

interface PriorityBinsLegendProps {
  bins: PriorityBinInfo[];
  /** Render as overlay (absolute positioned) or inline */
  variant?: 'inline' | 'overlay';
  className?: string;
}

const PriorityBinsLegend = memo(function PriorityBinsLegend({
  bins,
  variant = 'inline',
  className = '',
}: PriorityBinsLegendProps) {
  const baseClasses = 'rounded-lg border border-slate-700 bg-slate-800/90 p-3';
  const overlayClasses = 'absolute top-3 right-3 z-10 backdrop-blur-sm';

  const containerClasses =
    variant === 'overlay'
      ? `${baseClasses} ${overlayClasses} ${className}`
      : `${baseClasses} ${className}`;

  return (
    <div className={containerClasses}>
      <p className="mb-1.5 text-xs font-medium uppercase tracking-wide text-slate-400">Priority</p>
      <div className="flex flex-col gap-1">
        {bins.map((bin) => (
          <div key={bin.label} className="flex items-center gap-1.5">
            <div
              className="h-2 w-2 flex-shrink-0 rounded-sm"
              style={{ backgroundColor: bin.color }}
              aria-hidden="true"
            />
            <span className="whitespace-nowrap text-xs text-slate-300">{bin.label}</span>
          </div>
        ))}
      </div>
    </div>
  );
});

export default PriorityBinsLegend;
