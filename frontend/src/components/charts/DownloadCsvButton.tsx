/**
 * Pre-styled "Download CSV" button matching the chart-panel header
 * action chrome (PNG/SVG/Fullscreen).  Clicks call {@link exportRowsAsCsv}
 * with the supplied label, rows, and columns.
 */
import type { ReactNode } from 'react';
import { exportRowsAsCsv, type CsvColumn } from '@/lib/csvExport';

const HEADER_BTN_CLASS =
  'inline-flex h-7 items-center gap-1 rounded-md border border-slate-600 bg-slate-800/70 px-2 text-xs font-medium text-slate-300 transition-colors hover:bg-slate-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-900 disabled:cursor-not-allowed disabled:opacity-50';

export interface DownloadCsvButtonProps<Row> {
  /** Used to derive the CSV filename and aria label. */
  label: string;
  /** Rows to export — typically already filtered to what the user sees. */
  rows: readonly Row[];
  /** Column extractors. */
  columns: readonly CsvColumn<Row>[];
  /** Optional override for button text. Defaults to "CSV". */
  children?: ReactNode;
  /** Disable the button (e.g. while data is still loading). */
  disabled?: boolean;
}

export default function DownloadCsvButton<Row>({
  label,
  rows,
  columns,
  children,
  disabled,
}: DownloadCsvButtonProps<Row>) {
  const isEmpty = rows.length === 0 || columns.length === 0;
  return (
    <button
      type="button"
      onClick={() => exportRowsAsCsv(label, rows, columns)}
      className={HEADER_BTN_CLASS}
      disabled={disabled || isEmpty}
      title={isEmpty ? 'Nothing to export yet' : `Download ${label} as CSV`}
      aria-label={`Download ${label} as CSV`}
    >
      {children ?? 'CSV'}
    </button>
  );
}
