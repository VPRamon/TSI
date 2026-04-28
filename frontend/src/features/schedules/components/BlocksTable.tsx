/**
 * BlocksTable - Sortable, filterable table for block-level inspection.
 *
 * Features:
 * - Column sorting (priority, visibility, requested hours, scheduled status)
 * - Row selection (integrates with AnalysisContext)
 * - Click-through to details drawer
 * - Compact display with key metrics
 *
 * Used in VisibilityMap, Insights, and other drill-down views.
 */
import { useState, useMemo, useCallback, memo, useEffect, useDeferredValue } from 'react';
import { useBlockSelection } from '../context/AnalysisContext';
import { useAsyncMemo } from '@/hooks/useAsyncMemo';
import { sortFilterBlocks as sortFilterBlocksSync } from '@/workers/aggregations';
import { getAggregationsClient } from '@/workers/aggregationsClient';

const FILTERING_HINT_DELAY_MS = 200;

/** Below this row count we skip the worker round-trip and compute inline. */
const WORKER_BLOCKS_THRESHOLD = 200;

// Generic block interface - pages provide their specific block type
export interface TableBlock {
  scheduling_block_id: number;
  original_block_id: string;
  block_name?: string;
  priority: number;
  scheduled: boolean;
  total_visibility_hours?: number;
  requested_hours?: number;
  num_visibility_periods?: number;
}

type SortField = 'priority' | 'scheduled' | 'total_visibility_hours' | 'requested_hours';
type SortDirection = 'asc' | 'desc';

interface BlocksTableProps<T extends TableBlock> {
  /** Array of blocks to display */
  blocks: T[];
  /** Loading state */
  isLoading?: boolean;
  /** Page size for pagination (default 100). When total rows fit in a single page no pagination footer is shown. */
  maxRows?: number;
  /** Callback when a block row is clicked */
  onBlockClick?: (block: T) => void;
  /** Additional columns to render */
  renderExtraColumns?: (block: T) => React.ReactNode;
  /** Extra column headers */
  extraColumnHeaders?: { label: string; className?: string }[];
  /** Title for the table section */
  title?: string;
  /** Whether to show selection checkboxes */
  showSelection?: boolean;
  /** Compact mode (smaller rows) */
  compact?: boolean;
}

function BlocksTableInner<T extends TableBlock>({
  blocks,
  isLoading = false,
  maxRows = 100,
  onBlockClick,
  renderExtraColumns,
  extraColumnHeaders = [],
  title = 'Blocks',
  showSelection = true,
  compact = false,
}: BlocksTableProps<T>) {
  const [sortField, setSortField] = useState<SortField>('priority');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');
  const [filter, setFilter] = useState('');
  // Defer the value used by the heavy filter/sort pipeline so the controlled
  // input stays responsive while large block sets re-filter in the background.
  const deferredFilter = useDeferredValue(filter);
  const isFilteringPending = filter !== deferredFilter;
  const [showFilteringHint, setShowFilteringHint] = useState(false);
  const [page, setPage] = useState(0);

  useEffect(() => {
    if (!isFilteringPending) {
      setShowFilteringHint(false);
      return;
    }
    const handle = setTimeout(() => setShowFilteringHint(true), FILTERING_HINT_DELAY_MS);
    return () => clearTimeout(handle);
  }, [isFilteringPending]);

  const { selectedBlockIds, selectBlocks, addToSelection, removeFromSelection, isSelected } =
    useBlockSelection();

  // Filter and sort the full block set (sorting happens BEFORE pagination so
  // page slices reflect the chosen sort order across the whole dataset).
  // For small inputs we compute synchronously to avoid the postMessage hop;
  // for larger ones we offload to the shared aggregations worker and keep
  // showing the previous result until the new one resolves.
  const useWorker = blocks.length > WORKER_BLOCKS_THRESHOLD;

  const inlineSorted = useMemo(
    () =>
      sortFilterBlocksSync(blocks, deferredFilter, {
        field: sortField,
        direction: sortDirection,
      }),
    [blocks, deferredFilter, sortField, sortDirection]
  );

  const { value: sortedBlocks } = useAsyncMemo<T[]>(
    () => {
      if (!useWorker) return inlineSorted;
      const client = getAggregationsClient();
      if (!client) return inlineSorted;
      return client.sortFilterBlocks(blocks, deferredFilter, {
        field: sortField,
        direction: sortDirection,
      }) as Promise<T[]>;
    },
    [useWorker, blocks, deferredFilter, sortField, sortDirection, inlineSorted],
    inlineSorted
  );

  const totalRows = sortedBlocks.length;
  const pageSize = Math.max(1, maxRows);
  const totalPages = Math.max(1, Math.ceil(totalRows / pageSize));
  const safePage = Math.min(page, totalPages - 1);

  // Reset to first page when the filter, sort, or underlying data changes so
  // the user is not left on an out-of-range page.
  useEffect(() => {
    setPage(0);
  }, [deferredFilter, sortField, sortDirection, blocks, pageSize]);

  const processedBlocks = useMemo(() => {
    if (totalRows <= pageSize) return sortedBlocks;
    const start = safePage * pageSize;
    return sortedBlocks.slice(start, start + pageSize);
  }, [sortedBlocks, safePage, pageSize, totalRows]);

  const showPagination = totalRows > pageSize;
  const pageStart = totalRows === 0 ? 0 : safePage * pageSize + 1;
  const pageEnd = Math.min(totalRows, (safePage + 1) * pageSize);

  // Handle column header click for sorting
  const handleSort = useCallback((field: SortField) => {
    setSortField((prev) => {
      if (prev === field) {
        setSortDirection((d) => (d === 'asc' ? 'desc' : 'asc'));
        return prev;
      }
      setSortDirection('desc');
      return field;
    });
  }, []);

  // Handle select all visible
  const handleSelectAll = useCallback(() => {
    const allIds = processedBlocks.map((b) => b.scheduling_block_id);
    const allSelected = allIds.every((id) => selectedBlockIds.has(id));

    if (allSelected) {
      // Deselect all visible
      removeFromSelection(allIds);
    } else {
      // Select all visible
      addToSelection(allIds);
    }
  }, [processedBlocks, selectedBlockIds, addToSelection, removeFromSelection]);

  // Handle row selection toggle
  const handleRowSelect = useCallback(
    (block: T, event: React.MouseEvent) => {
      event.stopPropagation();
      const id = block.scheduling_block_id;
      if (isSelected(id)) {
        removeFromSelection([id]);
      } else {
        if (event.shiftKey) {
          addToSelection([id]);
        } else {
          selectBlocks([id]);
        }
      }
    },
    [isSelected, removeFromSelection, addToSelection, selectBlocks]
  );

  // Handle row click (for details)
  const handleRowClick = useCallback(
    (block: T) => {
      onBlockClick?.(block);
    },
    [onBlockClick]
  );

  const rowPadding = compact ? 'py-1.5 px-2' : 'py-2.5 px-3';
  const allVisibleSelected =
    processedBlocks.length > 0 &&
    processedBlocks.every((b) => selectedBlockIds.has(b.scheduling_block_id));

  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) {
      return <span className="ml-1 text-slate-600">↕</span>;
    }
    return <span className="ml-1 text-primary-400">{sortDirection === 'asc' ? '↑' : '↓'}</span>;
  };

  if (isLoading) {
    return (
      <div className="rounded-lg border border-slate-700 bg-slate-800/50 p-8 text-center">
        <div className="inline-block h-6 w-6 animate-spin rounded-full border-2 border-slate-600 border-t-primary-500" />
        <p className="mt-2 text-sm text-slate-400">Loading blocks...</p>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-slate-700 bg-slate-800/50">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-slate-700 px-4 py-3">
        <div className="flex items-center gap-3">
          <h3 className="text-sm font-medium text-white">{title}</h3>
          <span className="text-xs text-slate-400">
            {totalRows}
            {totalRows !== blocks.length && ` of ${blocks.length}`} blocks
          </span>
          {selectedBlockIds.size > 0 && (
            <span className="rounded-full bg-primary-600/20 px-2 py-0.5 text-xs text-primary-400">
              {selectedBlockIds.size} selected
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          {/* Search filter */}
          <input
            type="text"
            placeholder="Filter by ID..."
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            aria-busy={isFilteringPending}
            className="w-40 rounded border border-slate-600 bg-slate-700 px-2 py-1 text-xs text-white placeholder-slate-400 focus:border-primary-500 focus:outline-none"
          />
          {showFilteringHint && (
            <span className="text-xs text-slate-400" role="status" aria-live="polite">
              Filtering…
            </span>
          )}
          {selectedBlockIds.size > 0 && (
            <button
              onClick={() => selectBlocks([])}
              className="text-xs text-slate-400 hover:text-white"
            >
              Clear selection
            </button>
          )}
        </div>
      </div>

      {/* Table */}
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-slate-700 text-left text-xs uppercase tracking-wider text-slate-400">
              {showSelection && (
                <th className={`${rowPadding} w-8`}>
                  <input
                    type="checkbox"
                    checked={allVisibleSelected}
                    onChange={handleSelectAll}
                    className="rounded border-slate-500 bg-slate-700 text-primary-600 focus:ring-primary-500"
                  />
                </th>
              )}
              <th className={rowPadding}>Block ID</th>
              <th
                className={`${rowPadding} cursor-pointer hover:text-white`}
                onClick={() => handleSort('priority')}
              >
                Priority
                <SortIcon field="priority" />
              </th>
              <th
                className={`${rowPadding} cursor-pointer hover:text-white`}
                onClick={() => handleSort('scheduled')}
              >
                Status
                <SortIcon field="scheduled" />
              </th>
              {blocks[0]?.total_visibility_hours !== undefined && (
                <th
                  className={`${rowPadding} cursor-pointer hover:text-white`}
                  onClick={() => handleSort('total_visibility_hours')}
                >
                  Visibility (h)
                  <SortIcon field="total_visibility_hours" />
                </th>
              )}
              {blocks[0]?.requested_hours !== undefined && (
                <th
                  className={`${rowPadding} cursor-pointer hover:text-white`}
                  onClick={() => handleSort('requested_hours')}
                >
                  Requested (h)
                  <SortIcon field="requested_hours" />
                </th>
              )}
              {extraColumnHeaders.map((header, i) => (
                <th key={i} className={`${rowPadding} ${header.className ?? ''}`}>
                  {header.label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-700/50">
            {processedBlocks.length === 0 ? (
              <tr>
                <td colSpan={10} className="px-4 py-8 text-center text-sm text-slate-400">
                  {deferredFilter ? 'No blocks match your filter' : 'No blocks available'}
                </td>
              </tr>
            ) : (
              processedBlocks.map((block) => {
                const selected = isSelected(block.scheduling_block_id);
                return (
                  <tr
                    key={block.scheduling_block_id}
                    onClick={() => handleRowClick(block)}
                    className={`cursor-pointer transition-colors ${
                      selected
                        ? 'bg-primary-600/10 hover:bg-primary-600/20'
                        : 'hover:bg-slate-700/30'
                    }`}
                  >
                    {showSelection && (
                      <td className={rowPadding}>
                        <input
                          type="checkbox"
                          checked={selected}
                          onClick={(e) => handleRowSelect(block, e)}
                          onChange={() => {}} // Controlled by onClick
                          className="rounded border-slate-500 bg-slate-700 text-primary-600 focus:ring-primary-500"
                        />
                      </td>
                    )}
                    <td className={`${rowPadding} font-mono text-xs`}>
                      <span className="text-slate-500">#{block.scheduling_block_id}</span>
                      <span className="ml-2 text-slate-300">{block.original_block_id}</span>
                      {block.block_name && (
                        <span className="ml-1 block text-slate-400">{block.block_name}</span>
                      )}
                    </td>
                    <td className={rowPadding}>
                      <span
                        className={`font-medium ${
                          block.priority >= 8
                            ? 'text-red-400'
                            : block.priority >= 5
                              ? 'text-amber-400'
                              : 'text-slate-300'
                        }`}
                      >
                        {block.priority.toFixed(1)}
                      </span>
                    </td>
                    <td className={rowPadding}>
                      {block.scheduled ? (
                        <span className="inline-flex items-center gap-1 text-emerald-400">
                          <span className="h-1.5 w-1.5 rounded-full bg-emerald-400" />
                          Scheduled
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 text-slate-400">
                          <span className="h-1.5 w-1.5 rounded-full bg-slate-500" />
                          Unscheduled
                        </span>
                      )}
                    </td>
                    {block.total_visibility_hours !== undefined && (
                      <td className={`${rowPadding} text-slate-300`}>
                        {block.total_visibility_hours.toFixed(1)}
                      </td>
                    )}
                    {block.requested_hours !== undefined && (
                      <td className={`${rowPadding} text-slate-300`}>
                        {block.requested_hours.toFixed(1)}
                      </td>
                    )}
                    {renderExtraColumns?.(block)}
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination footer */}
      {showPagination && (
        <div className="flex items-center justify-between border-t border-slate-700 px-4 py-2 text-xs text-slate-400">
          <span>
            Showing {pageStart}–{pageEnd} of {totalRows}
          </span>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setPage((p) => Math.max(0, p - 1))}
              disabled={safePage === 0}
              className="rounded border border-slate-600 px-2 py-1 text-xs text-slate-300 hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-40"
            >
              Prev
            </button>
            <span className="tabular-nums">
              Page {safePage + 1} / {totalPages}
            </span>
            <button
              type="button"
              onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
              disabled={safePage >= totalPages - 1}
              className="rounded border border-slate-600 px-2 py-1 text-xs text-slate-300 hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-40"
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

// Memoize with generic support
export const BlocksTable = memo(BlocksTableInner) as typeof BlocksTableInner;
