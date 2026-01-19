/**
 * SplitPane component - Controls on left, main content on right (desktop).
 * Stacks vertically on mobile.
 */
import { ReactNode } from 'react';

interface SplitPaneProps {
  /** Left/top panel content (controls) */
  controls: ReactNode;
  /** Right/bottom panel content (main visualization) */
  children: ReactNode;
  /** Width of controls panel on desktop */
  controlsWidth?: 'sm' | 'md' | 'lg';
  /** Optional className for the container */
  className?: string;
}

const widthClasses = {
  sm: 'lg:w-64',
  md: 'lg:w-80',
  lg: 'lg:w-96',
};

function SplitPane({ controls, children, controlsWidth = 'md', className = '' }: SplitPaneProps) {
  return (
    <div className={`flex flex-col gap-6 lg:flex-row ${className}`}>
      {/* Controls panel */}
      <aside
        className={`shrink-0 ${widthClasses[controlsWidth]}`}
      >
        <div className="rounded-lg border border-slate-700 bg-slate-800/50 p-4">
          {controls}
        </div>
      </aside>
      
      {/* Main content area */}
      <div className="min-w-0 flex-1">
        {children}
      </div>
    </div>
  );
}

export default SplitPane;
