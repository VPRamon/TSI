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
import { useState, useMemo, useCallback, memo } from 'react';
import { useBlockSelection } from '../context/AnalysisContext';

// Generic block interface - pages provide their specific block type
export interface TableBlock {
  scheduling_block_id: number;
  original_block_id: string;
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
  /** Maximum rows to display (pagination not implemented, just truncation) */
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

  const { selectedBlockIds, selectBlocks, addToSelection, removeFromSelection, isSelected } =
    useBlockSelection();

  // Filter and sort blocks
  const processedBlocks = useMemo(() => {
    let result = [...blocks];

    // Filter by search term (block ID or original ID)
    if (filter) {
      const lowerFilter = filter.toLowerCase();
      result = result.filter(
        (b) =>
          b.scheduling_block_id.toString().includes(lowerFilter) ||
          b.original_block_id.toLowerCase().includes(lowerFilter)
      );
    }

    // Sort
    result.sort((a, b) => {
      let aVal: number | boolean;
      let bVal: number | boolean;

      switch (sortField) {
        case 'priority':
          aVal = a.priority;
          bVal = b.priority;
          break;
        case 'scheduled':
          aVal = a.scheduled ? 1 : 0;
          bVal = b.scheduled ? 1 : 0;
          break;
        case 'total_visibility_hours':
          aVal = a.total_visibility_hours ?? 0;
          bVal = b.total_visibility_hours ?? 0;
          break;
        case 'requested_hours':
          aVal = a.requested_hours ?? 0;
          bVal = b.requested_hours ?? 0;
          break;
        default:
          return 0;
      }

      const diff = typeof aVal === 'number' && typeof bVal === 'number' ? aVal - bVal : 0;
      return sortDirection === 'asc' ? diff : -diff;
    });

    // Limit rows
    return result.slice(0, maxRows);
  }, [blocks, filter, sortField, sortDirection, maxRows]);

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
            {processedBlocks.length}
            {processedBlocks.length !== blocks.length && ` of ${blocks.length}`} blocks
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
            className="w-40 rounded border border-slate-600 bg-slate-700 px-2 py-1 text-xs text-white placeholder-slate-400 focus:border-primary-500 focus:outline-none"
          />
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
                <td
                  colSpan={10}
                  className="px-4 py-8 text-center text-sm text-slate-400"
                >
                  {filter ? 'No blocks match your filter' : 'No blocks available'}
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

      {/* Footer with truncation notice */}
      {blocks.length > maxRows && (
        <div className="border-t border-slate-700 px-4 py-2 text-xs text-slate-400">
          Showing first {maxRows} of {blocks.length} blocks. Use filters to narrow results.
        </div>
      )}
    </div>
  );
}

// Memoize with generic support
export const BlocksTable = memo(BlocksTableInner) as typeof BlocksTableInner;
