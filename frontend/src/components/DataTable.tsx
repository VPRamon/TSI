/**
 * Accessible data table component.
 * Provides proper semantics, ARIA attributes, and keyboard navigation.
 */
import { ReactNode, memo } from 'react';

export interface TableColumn<T> {
  /** Column header text */
  header: string;
  /** Key or accessor for the column data */
  accessor: keyof T | ((row: T) => ReactNode);
  /** Column alignment */
  align?: 'left' | 'center' | 'right';
  /** Column width class (Tailwind) */
  width?: string;
  /** Screen reader only header (visually hidden) */
  srOnly?: boolean;
}

export interface DataTableProps<T> {
  /** Table data rows */
  data: T[];
  /** Column definitions */
  columns: TableColumn<T>[];
  /** Row key accessor */
  keyAccessor: (row: T, index: number) => string | number;
  /** Accessible table caption (describes table purpose) */
  caption: string;
  /** Whether caption is visually hidden */
  captionHidden?: boolean;
  /** Empty state message */
  emptyMessage?: string;
  /** Maximum rows to display */
  maxRows?: number;
  /** Show "and X more" text when truncated */
  showMoreText?: boolean;
}

const alignmentClasses = {
  left: 'text-left',
  center: 'text-center',
  right: 'text-right',
};

function DataTableInner<T>({
  data,
  columns,
  keyAccessor,
  caption,
  captionHidden = false,
  emptyMessage = 'No data available',
  maxRows,
  showMoreText = true,
}: DataTableProps<T>) {
  const displayData = maxRows ? data.slice(0, maxRows) : data;
  const hasMore = maxRows && data.length > maxRows;

  if (data.length === 0) {
    return (
      <div className="py-8 text-center text-slate-400" role="status" aria-live="polite">
        {emptyMessage}
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm" role="table">
        <caption className={captionHidden ? 'sr-only' : 'mb-2 text-left text-slate-400'}>
          {caption}
        </caption>
        <thead>
          <tr className="border-b border-slate-700">
            {columns.map((col, index) => (
              <th
                key={index}
                scope="col"
                className={`px-4 py-3 font-medium text-slate-400 ${alignmentClasses[col.align ?? 'left']} ${col.width ?? ''} ${col.srOnly ? 'sr-only' : ''}`}
              >
                {col.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {displayData.map((row, rowIndex) => (
            <tr
              key={keyAccessor(row, rowIndex)}
              className="border-b border-slate-700/50 transition-colors hover:bg-slate-800/50"
            >
              {columns.map((col, colIndex) => {
                const value =
                  typeof col.accessor === 'function'
                    ? col.accessor(row)
                    : (row[col.accessor] as ReactNode);
                return (
                  <td
                    key={colIndex}
                    className={`px-4 py-3 ${alignmentClasses[col.align ?? 'left']} ${colIndex === 0 ? 'text-white' : 'text-slate-300'}`}
                  >
                    {value}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
      {hasMore && showMoreText && (
        <p className="mt-4 text-center text-sm text-slate-400" aria-live="polite">
          ... and {data.length - maxRows!} more items
        </p>
      )}
    </div>
  );
}

/**
 * Memoized data table for performance.
 */
export const DataTable = memo(DataTableInner) as typeof DataTableInner;

export default DataTable;
