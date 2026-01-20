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
  
  const containerClasses = variant === 'overlay' 
    ? `${baseClasses} ${overlayClasses} ${className}`
    : `${baseClasses} ${className}`;

  return (
    <div className={containerClasses}>
      <h4 className="mb-2 text-xs font-medium uppercase tracking-wide text-slate-400">
        Priority Bins
      </h4>
      <div className="flex flex-col gap-1.5">
        {bins.map((bin) => (
          <div key={bin.label} className="flex items-center gap-2">
            <div
              className="h-2.5 w-2.5 flex-shrink-0 rounded-sm"
              style={{ backgroundColor: bin.color }}
              aria-hidden="true"
            />
            <span className="text-xs text-slate-300 whitespace-nowrap">
              {bin.label}: {bin.min_priority.toFixed(1)} – {bin.max_priority.toFixed(1)}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
});

export default PriorityBinsLegend;
