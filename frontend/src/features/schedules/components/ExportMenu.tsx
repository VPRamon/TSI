/**
 * ExportMenu - Dropdown menu for exporting analysis data.
 *
 * Features:
 * - Export filtered blocks as CSV or JSON
 * - Export selected blocks only
 * - Copy block IDs to clipboard
 * - Copy shareable permalink
 */
import { useState, useRef, useEffect, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { useAppStore } from '@/store/appStore';
import { useBlockSelection } from '../context/AnalysisContext';
import {
  exportBlocksToCSV,
  exportBlocksToJSON,
  exportBlockIds,
  copyBlockIdsToClipboard,
  copyPermalinkToClipboard,
  generateExportFilename,
  type ExportableBlock,
  type ExportMetadata,
} from '../utils/export';

interface ExportMenuProps<T extends ExportableBlock> {
  /** All blocks currently visible/filtered */
  blocks: T[];
  /** Total blocks before filtering (for metadata) */
  totalBlocks?: number;
  /** Columns to include in CSV export */
  columns: string[];
  /** Additional CSS classes */
  className?: string;
}

export function ExportMenu<T extends ExportableBlock>({
  blocks,
  totalBlocks,
  columns,
  className = '',
}: ExportMenuProps<T>) {
  const [isOpen, setIsOpen] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const { scheduleId } = useParams();
  const currentId = parseInt(scheduleId ?? '0', 10);
  const { selectedSchedule } = useAppStore();
  const { selectedBlockIds } = useBlockSelection();

  // Close menu when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Get selected blocks
  const selectedBlocks = blocks.filter((b) => selectedBlockIds.has(b.scheduling_block_id));
  const hasSelection = selectedBlocks.length > 0;

  // Build export metadata
  const buildMetadata = useCallback(
    (exportedBlocks: T[]): ExportMetadata => ({
      exportedAt: new Date().toISOString(),
      scheduleId: currentId,
      scheduleName: selectedSchedule?.schedule_name,
      filters: {
        selectedBlockCount: selectedBlockIds.size || undefined,
      },
      totalBlocks: totalBlocks ?? blocks.length,
      exportedBlocks: exportedBlocks.length,
    }),
    [currentId, selectedSchedule, selectedBlockIds.size, totalBlocks, blocks.length]
  );

  // Export handlers
  const handleExportCSV = useCallback(
    (blocksToExport: T[]) => {
      const filename = generateExportFilename(currentId, hasSelection ? 'selection' : 'filtered', 'csv');
      exportBlocksToCSV(blocksToExport, filename, columns);
      setIsOpen(false);
    },
    [currentId, hasSelection, columns]
  );

  const handleExportJSON = useCallback(
    (blocksToExport: T[]) => {
      const filename = generateExportFilename(currentId, hasSelection ? 'selection' : 'filtered', 'json');
      const metadata = buildMetadata(blocksToExport);
      exportBlocksToJSON(blocksToExport, metadata, filename);
      setIsOpen(false);
    },
    [currentId, hasSelection, buildMetadata]
  );

  const handleExportBlockIds = useCallback(
    (blocksToExport: T[]) => {
      const filename = generateExportFilename(currentId, hasSelection ? 'selection' : 'filtered', 'txt');
      exportBlockIds(blocksToExport, filename, 'txt');
      setIsOpen(false);
    },
    [currentId, hasSelection]
  );

  const handleCopyBlockIds = useCallback(
    async (blocksToExport: T[]) => {
      await copyBlockIdsToClipboard(blocksToExport);
      setCopied('ids');
      setTimeout(() => setCopied(null), 2000);
    },
    []
  );

  const handleCopyPermalink = useCallback(async () => {
    await copyPermalinkToClipboard();
    setCopied('link');
    setTimeout(() => setCopied(null), 2000);
  }, []);

  return (
    <div ref={menuRef} className={`relative ${className}`}>
      {/* Trigger button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-1.5 rounded-lg border border-slate-600 bg-slate-700/50 px-3 py-1.5 text-sm text-slate-300 hover:bg-slate-700 hover:text-white"
      >
        <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
          />
        </svg>
        Export
        <svg
          className={`h-3 w-3 transition-transform ${isOpen ? 'rotate-180' : ''}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {/* Dropdown menu */}
      {isOpen && (
        <div className="absolute right-0 top-full z-50 mt-1 w-56 rounded-lg border border-slate-600 bg-slate-800 py-1 shadow-lg">
          {/* Export all filtered */}
          <div className="border-b border-slate-700 px-3 py-1.5 text-xs font-medium uppercase tracking-wider text-slate-400">
            Export Filtered ({blocks.length})
          </div>
          <button
            onClick={() => handleExportCSV(blocks)}
            className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700"
          >
            <span className="text-slate-400">📊</span>
            Download as CSV
          </button>
          <button
            onClick={() => handleExportJSON(blocks)}
            className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700"
          >
            <span className="text-slate-400">📋</span>
            Download as JSON
          </button>
          <button
            onClick={() => handleExportBlockIds(blocks)}
            className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700"
          >
            <span className="text-slate-400">📝</span>
            Export Block IDs
          </button>

          {/* Export selection (if any) */}
          {hasSelection && (
            <>
              <div className="border-b border-t border-slate-700 px-3 py-1.5 text-xs font-medium uppercase tracking-wider text-slate-400">
                Export Selection ({selectedBlocks.length})
              </div>
              <button
                onClick={() => handleExportCSV(selectedBlocks)}
                className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700"
              >
                <span className="text-slate-400">📊</span>
                Selection as CSV
              </button>
              <button
                onClick={() => handleExportJSON(selectedBlocks)}
                className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700"
              >
                <span className="text-slate-400">📋</span>
                Selection as JSON
              </button>
            </>
          )}

          {/* Copy actions */}
          <div className="border-b border-t border-slate-700 px-3 py-1.5 text-xs font-medium uppercase tracking-wider text-slate-400">
            Quick Actions
          </div>
          <button
            onClick={() => handleCopyBlockIds(hasSelection ? selectedBlocks : blocks)}
            className="flex w-full items-center justify-between px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700"
          >
            <span className="flex items-center gap-2">
              <span className="text-slate-400">📋</span>
              Copy Block IDs
            </span>
            {copied === 'ids' && (
              <span className="text-xs text-emerald-400">Copied!</span>
            )}
          </button>
          <button
            onClick={handleCopyPermalink}
            className="flex w-full items-center justify-between px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700"
          >
            <span className="flex items-center gap-2">
              <span className="text-slate-400">🔗</span>
              Copy Permalink
            </span>
            {copied === 'link' && (
              <span className="text-xs text-emerald-400">Copied!</span>
            )}
          </button>
        </div>
      )}
    </div>
  );
}
