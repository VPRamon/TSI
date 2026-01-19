/**
 * BlockDetailsDrawer - Side panel showing detailed info for a selected block.
 *
 * Features:
 * - Slides in from right
 * - Shows all available block metrics
 * - Actions: navigate to related views, add to selection
 * - Keyboard accessible (Escape to close)
 */
import { useEffect, useCallback, memo } from 'react';
import { useBlockSelection } from '../context/AnalysisContext';
import type { TableBlock } from './BlocksTable';

interface BlockDetailsDrawerProps<T extends TableBlock> {
  /** Block to display details for, or null to hide drawer */
  block: T | null;
  /** Close handler */
  onClose: () => void;
  /** Additional details to render */
  renderExtraDetails?: (block: T) => React.ReactNode;
  /** Optional scheduled time info */
  scheduledTime?: { start: number; end: number } | null;
}

function BlockDetailsDrawerInner<T extends TableBlock>({
  block,
  onClose,
  renderExtraDetails,
  scheduledTime,
}: BlockDetailsDrawerProps<T>) {
  const { isSelected, addToSelection, removeFromSelection } = useBlockSelection();

  // Close on Escape
  useEffect(() => {
    if (!block) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [block, onClose]);

  // Toggle selection for this block
  const handleToggleSelection = useCallback(() => {
    if (!block) return;
    const id = block.scheduling_block_id;
    if (isSelected(id)) {
      removeFromSelection([id]);
    } else {
      addToSelection([id]);
    }
  }, [block, isSelected, addToSelection, removeFromSelection]);

  // Format unix timestamp to readable date
  const formatTime = (unix: number) => {
    return new Date(unix * 1000).toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  if (!block) return null;

  const selected = isSelected(block.scheduling_block_id);

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-40 bg-slate-900/60 backdrop-blur-sm"
        onClick={onClose}
        aria-hidden="true"
      />

      {/* Drawer */}
      <aside
        className="fixed inset-y-0 right-0 z-50 flex w-full max-w-md flex-col border-l border-slate-700 bg-slate-800 shadow-xl"
        role="dialog"
        aria-modal="true"
        aria-labelledby="drawer-title"
      >
        {/* Header */}
        <div className="flex items-center justify-between border-b border-slate-700 px-4 py-3">
          <div>
            <h2 id="drawer-title" className="text-lg font-semibold text-white">
              Block Details
            </h2>
            <p className="text-sm text-slate-400">
              #{block.scheduling_block_id} • {block.original_block_id}
            </p>
          </div>
          <button
            onClick={onClose}
            className="rounded-lg p-2 text-slate-400 hover:bg-slate-700 hover:text-white"
            aria-label="Close details"
          >
            <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {/* Status badge */}
          <div className="mb-6">
            {block.scheduled ? (
              <span className="inline-flex items-center gap-2 rounded-full bg-emerald-500/10 px-3 py-1.5 text-sm font-medium text-emerald-400">
                <span className="h-2 w-2 rounded-full bg-emerald-400" />
                Scheduled
              </span>
            ) : (
              <span className="inline-flex items-center gap-2 rounded-full bg-slate-500/10 px-3 py-1.5 text-sm font-medium text-slate-400">
                <span className="h-2 w-2 rounded-full bg-slate-500" />
                Not Scheduled
              </span>
            )}
          </div>

          {/* Key metrics grid */}
          <div className="mb-6 grid grid-cols-2 gap-4">
            <MetricBox
              label="Priority"
              value={block.priority.toFixed(2)}
              highlight={block.priority >= 8 ? 'high' : block.priority >= 5 ? 'medium' : 'low'}
            />
            {block.total_visibility_hours !== undefined && (
              <MetricBox
                label="Total Visibility"
                value={`${block.total_visibility_hours.toFixed(1)}h`}
              />
            )}
            {block.requested_hours !== undefined && (
              <MetricBox label="Requested Duration" value={`${block.requested_hours.toFixed(1)}h`} />
            )}
            {block.num_visibility_periods !== undefined && (
              <MetricBox label="Visibility Windows" value={block.num_visibility_periods.toString()} />
            )}
          </div>

          {/* Scheduled time */}
          {scheduledTime && (
            <div className="mb-6 rounded-lg border border-slate-700 bg-slate-900/50 p-4">
              <h3 className="mb-2 text-xs font-medium uppercase tracking-wider text-slate-400">
                Scheduled Time
              </h3>
              <p className="text-sm text-white">{formatTime(scheduledTime.start)}</p>
              <p className="text-xs text-slate-400">to</p>
              <p className="text-sm text-white">{formatTime(scheduledTime.end)}</p>
            </div>
          )}

          {/* Extra details from parent */}
          {renderExtraDetails?.(block)}

          {/* Actions */}
          <div className="mt-6 space-y-2">
            <button
              onClick={handleToggleSelection}
              className={`flex w-full items-center justify-center gap-2 rounded-lg border px-4 py-2 text-sm font-medium transition-colors ${
                selected
                  ? 'border-primary-500 bg-primary-500/10 text-primary-400 hover:bg-primary-500/20'
                  : 'border-slate-600 bg-slate-700/50 text-slate-300 hover:bg-slate-700'
              }`}
            >
              {selected ? (
                <>
                  <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                    <path
                      fillRule="evenodd"
                      d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                      clipRule="evenodd"
                    />
                  </svg>
                  Selected
                </>
              ) : (
                <>
                  <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M12 6v6m0 0v6m0-6h6m-6 0H6"
                    />
                  </svg>
                  Add to Selection
                </>
              )}
            </button>
          </div>
        </div>
      </aside>
    </>
  );
}

// Metric display box
interface MetricBoxProps {
  label: string;
  value: string;
  highlight?: 'high' | 'medium' | 'low';
}

function MetricBox({ label, value, highlight }: MetricBoxProps) {
  const highlightClass =
    highlight === 'high'
      ? 'text-red-400'
      : highlight === 'medium'
        ? 'text-amber-400'
        : 'text-white';

  return (
    <div className="rounded-lg border border-slate-700 bg-slate-900/50 p-3">
      <p className="text-xs text-slate-400">{label}</p>
      <p className={`mt-0.5 text-lg font-semibold ${highlightClass}`}>{value}</p>
    </div>
  );
}

export const BlockDetailsDrawer = memo(BlockDetailsDrawerInner) as typeof BlockDetailsDrawerInner;
