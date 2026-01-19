/**
 * Export utilities for schedule analysis data.
 *
 * Provides CSV and JSON export for block data, filtered views, and selections.
 * Designed for reproducibility: exports include filter state metadata.
 */

// =============================================================================
// Types
// =============================================================================

export interface ExportMetadata {
  exportedAt: string;
  scheduleId: number;
  scheduleName?: string;
  filters?: {
    priorityMin?: number;
    priorityMax?: number;
    scheduledFilter?: string;
    selectedBlockCount?: number;
  };
  totalBlocks: number;
  exportedBlocks: number;
}

// Base block type for export (minimal required fields)
export interface ExportableBlock {
  scheduling_block_id: number;
  original_block_id: string;
  priority: number;
  scheduled: boolean;
}

// =============================================================================
// CSV Export
// =============================================================================

/**
 * Convert array of objects to CSV string.
 */
function toCSV<T>(data: T[], columns: string[]): string {
  if (data.length === 0) return '';

  // Header row
  const header = columns.join(',');

  // Data rows
  const rows = data.map((row) => {
    return columns
      .map((col) => {
        const value = (row as Record<string, unknown>)[col];
        if (value === null || value === undefined) return '';
        if (typeof value === 'string') {
          // Escape quotes and wrap in quotes if contains comma/quote/newline
          if (value.includes(',') || value.includes('"') || value.includes('\n')) {
            return `"${value.replace(/"/g, '""')}"`;
          }
          return value;
        }
        if (typeof value === 'boolean') return value ? 'true' : 'false';
        return String(value);
      })
      .join(',');
  });

  return [header, ...rows].join('\n');
}

/**
 * Export blocks to CSV and trigger download.
 */
export function exportBlocksToCSV<T extends ExportableBlock>(
  blocks: T[],
  filename: string,
  columns: string[]
): void {
  const csv = toCSV(blocks, columns);
  downloadFile(csv, filename, 'text/csv');
}

// =============================================================================
// JSON Export
// =============================================================================

/**
 * Export blocks to JSON with metadata and trigger download.
 */
export function exportBlocksToJSON<T extends ExportableBlock>(
  blocks: T[],
  metadata: ExportMetadata,
  filename: string
): void {
  const exportData = {
    metadata,
    blocks,
  };
  const json = JSON.stringify(exportData, null, 2);
  downloadFile(json, filename, 'application/json');
}

// =============================================================================
// Block IDs Export (for use with other tools)
// =============================================================================

/**
 * Export just block IDs as a simple list (useful for piping to other analysis tools).
 */
export function exportBlockIds(
  blocks: ExportableBlock[],
  filename: string,
  format: 'csv' | 'json' | 'txt' = 'txt'
): void {
  const ids = blocks.map((b) => b.scheduling_block_id);

  if (format === 'json') {
    downloadFile(JSON.stringify(ids), filename, 'application/json');
  } else if (format === 'csv') {
    downloadFile(ids.join(','), filename, 'text/csv');
  } else {
    downloadFile(ids.join('\n'), filename, 'text/plain');
  }
}

// =============================================================================
// Clipboard
// =============================================================================

/**
 * Copy block IDs to clipboard.
 */
export async function copyBlockIdsToClipboard(blocks: ExportableBlock[]): Promise<void> {
  const ids = blocks.map((b) => b.scheduling_block_id).join(', ');
  await navigator.clipboard.writeText(ids);
}

/**
 * Copy current URL (for sharing analysis state via URL params).
 */
export async function copyPermalinkToClipboard(): Promise<void> {
  await navigator.clipboard.writeText(window.location.href);
}

// =============================================================================
// Download Helper
// =============================================================================

function downloadFile(content: string, filename: string, mimeType: string): void {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);

  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();

  // Cleanup
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

// =============================================================================
// Generate Filename
// =============================================================================

/**
 * Generate a descriptive filename for exports.
 */
export function generateExportFilename(
  scheduleId: number,
  type: 'blocks' | 'selection' | 'filtered',
  extension: 'csv' | 'json' | 'txt'
): string {
  const date = new Date().toISOString().split('T')[0];
  return `schedule_${scheduleId}_${type}_${date}.${extension}`;
}
