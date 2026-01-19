/**
 * MetricsGrid component - Responsive grid for MetricCard components.
 */
import { ReactNode } from 'react';

interface MetricsGridProps {
  children: ReactNode;
  /** Number of columns on large screens */
  columns?: 3 | 4 | 5;
  className?: string;
}

const columnClasses = {
  3: 'grid-cols-1 sm:grid-cols-2 lg:grid-cols-3',
  4: 'grid-cols-2 lg:grid-cols-4',
  5: 'grid-cols-2 sm:grid-cols-3 lg:grid-cols-5',
};

function MetricsGrid({ children, columns = 4, className = '' }: MetricsGridProps) {
  return (
    <div className={`grid gap-4 ${columnClasses[columns]} ${className}`}>
      {children}
    </div>
  );
}

export default MetricsGrid;
