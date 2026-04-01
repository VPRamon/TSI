/**
 * ChartPanel component - Flat panel for charts with optional title.
 * Less elevation than Card for cleaner chart presentation.
 */
import { ReactNode, memo } from 'react';

interface ChartPanelProps {
  title?: string;
  headerActions?: ReactNode;
  children: ReactNode;
  className?: string;
}

const ChartPanel = memo(function ChartPanel({
  title,
  headerActions,
  children,
  className = '',
}: ChartPanelProps) {
  return (
    <div className={`rounded-lg border border-slate-700 bg-slate-800/30 ${className}`}>
      {(title || headerActions) && (
        <div className="flex items-center justify-between gap-3 border-b border-slate-700/50 px-4 py-3">
          {title ? <h3 className="text-sm font-medium text-slate-300">{title}</h3> : <div />}
          {headerActions ? <div className="shrink-0">{headerActions}</div> : null}
        </div>
      )}
      <div className="p-4">{children}</div>
    </div>
  );
});

export default ChartPanel;
