/**
 * ToolbarRow component - Horizontal row of compact controls.
 * Wraps on smaller screens.
 */
import { ReactNode } from 'react';

interface ToolbarRowProps {
  children: ReactNode;
  className?: string;
}

function ToolbarRow({ children, className = '' }: ToolbarRowProps) {
  return (
    <div
      className={`flex flex-wrap items-end gap-4 rounded-lg border border-slate-700 bg-slate-800/50 p-4 ${className}`}
    >
      {children}
    </div>
  );
}

export default ToolbarRow;
