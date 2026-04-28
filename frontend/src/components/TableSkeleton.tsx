/**
 * TableSkeleton — animated placeholder rows for tables whose data is still
 * loading. Use as a drop-in below a table heading or inside an empty
 * `DataTable` body.
 */
import { memo } from 'react';

export interface TableSkeletonProps {
  /** Number of skeleton rows to render. */
  rows?: number;
  /** Number of skeleton cells per row. */
  columns?: number;
  /** Optional extra className applied to the root grid. */
  className?: string;
  /** Accessible label; consumed by screen readers via aria-busy + aria-label. */
  ariaLabel?: string;
}

const TableSkeleton = memo(function TableSkeleton({
  rows = 5,
  columns = 4,
  className = '',
  ariaLabel = 'Loading table data',
}: TableSkeletonProps) {
  return (
    <div
      role="status"
      aria-busy="true"
      aria-label={ariaLabel}
      className={`flex flex-col gap-2 ${className}`}
    >
      {Array.from({ length: rows }).map((_, r) => (
        <div key={r} className="flex gap-3">
          {Array.from({ length: columns }).map((_, c) => (
            <div
              key={c}
              className="h-4 flex-1 animate-pulse rounded bg-slate-700/60"
              style={{ animationDelay: `${(r * columns + c) * 40}ms` }}
            />
          ))}
        </div>
      ))}
    </div>
  );
});

export default TableSkeleton;
