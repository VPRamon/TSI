/**
 * ChartPanel component - Flat panel for charts with optional title.
 * Less elevation than Card for cleaner chart presentation.
 */
import { ReactNode, memo } from 'react';

interface ChartPanelProps {
  title?: string;
  children: ReactNode;
  className?: string;
}

const ChartPanel = memo(function ChartPanel({ title, children, className = '' }: ChartPanelProps) {
  return (
    <div className={`rounded-lg border border-slate-700 bg-slate-800/30 ${className}`}>
      {title && (
        <div className="border-b border-slate-700/50 px-4 py-3">
          <h3 className="text-sm font-medium text-slate-300">{title}</h3>
        </div>
      )}
      <div className="p-4">{children}</div>
    </div>
  );
});

export default ChartPanel;
