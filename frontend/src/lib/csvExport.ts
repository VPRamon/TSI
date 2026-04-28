/**
 * Helpers for exporting tabular data as RFC 4180-style CSV.
 *
 * The companion {@link DownloadCsvButton} component (under
 * `src/components/charts/`) wires a styled button into ChartPanel
 * `headerActions` so panels can offer "Download CSV" alongside the
 * existing Plotly PNG/SVG buttons.
 */

import { sanitizeImageFilename } from './imageExport';

export type CsvCell = string | number | boolean | null | undefined;

/** Column descriptor for {@link toCsv}. */
export interface CsvColumn<Row> {
  /** Header label written on the first row. */
  header: string;
  /** Extracts a primitive cell from a row. */
  accessor: (row: Row) => CsvCell;
}

const NEEDS_QUOTE_RE = /[",\r\n]/;

/**
 * Quote a single CSV field per RFC 4180: wrap in `"…"` when it contains
 * a delimiter, quote, or newline; double any embedded quotes.
 */
export function escapeCsvField(value: CsvCell): string {
  if (value === null || value === undefined) return '';
  const str = typeof value === 'string' ? value : String(value);
  if (!NEEDS_QUOTE_RE.test(str)) return str;
  return `"${str.replace(/"/g, '""')}"`;
}

/**
 * Build a CSV string from rows + column descriptors. Always emits the
 * header row first. Uses CRLF line endings for spreadsheet compatibility.
 */
export function toCsv<Row>(rows: readonly Row[], columns: readonly CsvColumn<Row>[]): string {
  if (columns.length === 0) return '';
  const lines: string[] = [];
  lines.push(columns.map((c) => escapeCsvField(c.header)).join(','));
  for (const row of rows) {
    lines.push(columns.map((c) => escapeCsvField(c.accessor(row))).join(','));
  }
  return lines.join('\r\n');
}

/**
 * Trigger a browser download of a CSV string.  Filename is sanitised the
 * same way as image exports so a label like "Run inventory" becomes
 * `run-inventory.csv`.
 */
export function downloadCsv(label: string, csv: string): void {
  const filename = sanitizeImageFilename(label);
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename.endsWith('.csv') ? filename : `${filename}.csv`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

/**
 * Convenience: build the CSV and trigger the download in one step.
 */
export function exportRowsAsCsv<Row>(
  label: string,
  rows: readonly Row[],
  columns: readonly CsvColumn<Row>[],
): void {
  downloadCsv(label, toCsv(rows, columns));
}
