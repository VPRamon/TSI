/**
 * Helpers for the environments feature: structure formatting,
 * file ingestion for bulk import, and small list utilities.
 */
import type { BulkImportItem, EnvironmentInfo, EnvironmentStructure } from '@/api/types';

const MJD_TO_UNIX_OFFSET_DAYS = 40_587;

function mjdToDate(mjd: number): Date {
  return new Date((mjd - MJD_TO_UNIX_OFFSET_DAYS) * 86_400_000);
}

function formatLat(latDeg: number): string {
  return `${Math.abs(latDeg).toFixed(2)}° ${latDeg >= 0 ? 'N' : 'S'}`;
}

function formatLon(lonDeg: number): string {
  return `${Math.abs(lonDeg).toFixed(2)}° ${lonDeg >= 0 ? 'E' : 'W'}`;
}

function formatDateRange(startMjd: number, endMjd: number): string {
  const fmt = (d: Date) =>
    d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' });
  return `${fmt(mjdToDate(startMjd))} – ${fmt(mjdToDate(endMjd))}`;
}

/**
 * Render an environment structure as a single-line summary suitable for
 * the environment card. Returns a friendly placeholder when the
 * environment has not been seeded with any schedule yet.
 */
export function formatStructureSummary(structure: EnvironmentStructure | null): string {
  if (!structure) {
    return 'No structure yet — upload the first schedule to seed it.';
  }
  const location = `${formatLat(structure.lat_deg)}, ${formatLon(structure.lon_deg)}`;
  const period = formatDateRange(structure.period_start_mjd, structure.period_end_mjd);
  return `${location} · ${period}`;
}

/** Read a `File` as text and JSON-parse it. Throws on either step. */
export async function validateScheduleJsonFile(file: File): Promise<BulkImportItem> {
  if (!file.name.toLowerCase().endsWith('.json')) {
    throw new Error(`${file.name}: only .json files are accepted`);
  }
  const text = await readFileAsText(file);
  let schedule_json: unknown;
  try {
    schedule_json = JSON.parse(text);
  } catch (err) {
    const reason = err instanceof Error ? err.message : 'invalid JSON';
    throw new Error(`${file.name}: ${reason}`);
  }
  return {
    name: file.name.replace(/\.json$/i, ''),
    schedule_json,
  };
}

function readFileAsText(file: File): Promise<string> {
  if (typeof file.text === 'function') {
    return file.text();
  }
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(reader.error ?? new Error('failed to read file'));
    reader.onload = () => resolve(typeof reader.result === 'string' ? reader.result : '');
    reader.readAsText(file);
  });
}

/** A single ingestion entry: a schedule file, optional paired trace, and the resulting item or error. */
export interface PreparedUploadEntry {
  schedule: File;
  trace?: File;
  item: BulkImportItem | null;
  error?: string;
}

/** Result of partitioning + parsing a drop. */
export interface PreparedUploadResult {
  entries: PreparedUploadEntry[];
  /** Trace `.jsonl` files dropped without any matching `.json` schedule. */
  orphanTraces: File[];
}

/**
 * Recover the "schedule stem" from a trace filename.
 *
 * Strips the trailing `.jsonl`, then a trailing `.<word>_trace` segment if
 * present (e.g. `e1-k1-b1.est_trace.jsonl` → `e1-k1-b1`). Without the
 * `_trace` suffix the bare stem is returned unchanged.
 */
function traceStem(name: string): string {
  const noExt = name.replace(/\.jsonl$/i, '');
  return noExt.replace(/\.[A-Za-z0-9]+_trace$/i, '');
}

/**
 * Partition dropped files into schedule + optional trace pairs, parsing
 * each schedule's JSON eagerly and reading the trace text into the
 * resulting `BulkImportItem.algorithm_trace_jsonl`.
 *
 * Pairing rule: a trace `<stem>.<algo>_trace.jsonl` (or any `<stem>.jsonl`)
 * is matched against a schedule whose name (sans `.json`) equals `<stem>`.
 * Unmatched traces are returned in `orphanTraces`. Files that are neither
 * `.json` nor `.jsonl` produce an entry with `error` set.
 */
export async function prepareUploadFiles(files: File[]): Promise<PreparedUploadResult> {
  const schedules: File[] = [];
  const traces: File[] = [];
  const invalid: File[] = [];

  for (const file of files) {
    const lower = file.name.toLowerCase();
    if (lower.endsWith('.jsonl')) {
      traces.push(file);
    } else if (lower.endsWith('.json')) {
      schedules.push(file);
    } else {
      invalid.push(file);
    }
  }

  const traceByStem = new Map<string, File>();
  for (const t of traces) {
    traceByStem.set(traceStem(t.name), t);
  }

  const entries: PreparedUploadEntry[] = [];

  for (const file of invalid) {
    entries.push({
      schedule: file,
      item: null,
      error: `${file.name}: only .json/.jsonl files are accepted`,
    });
  }

  for (const schedule of schedules) {
    const stem = schedule.name.replace(/\.json$/i, '');
    const trace = traceByStem.get(stem);
    if (trace) traceByStem.delete(stem);

    const text = await readFileAsText(schedule);
    let schedule_json: unknown;
    try {
      schedule_json = JSON.parse(text);
    } catch (err) {
      const reason = err instanceof Error ? err.message : 'invalid JSON';
      entries.push({
        schedule,
        trace,
        item: null,
        error: `${schedule.name}: ${reason}`,
      });
      continue;
    }

    const algorithm_trace_jsonl = trace ? await readFileAsText(trace) : undefined;
    entries.push({
      schedule,
      trace,
      item: {
        name: stem,
        schedule_json,
        ...(algorithm_trace_jsonl !== undefined ? { algorithm_trace_jsonl } : {}),
      },
    });
  }

  const orphanTraces = Array.from(traceByStem.values());

  return { entries, orphanTraces };
}

/** Sort environments most-recently-created first. Stable on equal timestamps. */
export function sortEnvironmentsByRecency(envs: EnvironmentInfo[]): EnvironmentInfo[] {
  return [...envs].sort((a, b) => {
    const ta = Date.parse(a.created_at);
    const tb = Date.parse(b.created_at);
    if (Number.isNaN(ta) && Number.isNaN(tb)) return 0;
    if (Number.isNaN(ta)) return 1;
    if (Number.isNaN(tb)) return -1;
    return tb - ta;
  });
}

/** Format an RFC3339 timestamp for display, falling back to the raw value. */
export function formatCreatedAt(iso: string): string {
  const ms = Date.parse(iso);
  if (Number.isNaN(ms)) return iso;
  return new Date(ms).toLocaleString('en-GB', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

/**
 * Default chunk size for the bulk-import progress UI.
 *
 * The backend handler now parallelises items inside a single request, so
 * sending one item per HTTP call (the previous behaviour) wastes the
 * server-side concurrency. Sending the entire batch in one shot loses
 * progress feedback. As a compromise we group items into modest chunks
 * — each chunk is one HTTP round-trip whose progress increments the bar.
 */
export const BULK_IMPORT_CHUNK_SIZE = 10;

export interface BulkImportChunkProgress {
  /** 1-based index of the chunk currently being uploaded. */
  chunkIndex: number;
  /** Total number of chunks in the submission. */
  chunkCount: number;
  /** First item name in the in-flight chunk (for inline display). */
  currentName: string;
  /** Number of items in the in-flight chunk. */
  chunkSize: number;
}

export function chunkBulkImportItems<T>(items: T[], chunkSize = BULK_IMPORT_CHUNK_SIZE): T[][] {
  const size = Math.max(1, Math.floor(chunkSize));
  const chunks: T[][] = [];
  for (let i = 0; i < items.length; i += size) {
    chunks.push(items.slice(i, i + size));
  }
  return chunks;
}

/**
 * Default number of bulk-import chunks the dialog keeps in flight to the
 * backend at the same time. The backend itself still parallelises items
 * inside each chunk (`BULK_IMPORT_CONCURRENCY`), so effective parallelism
 * is roughly `BULK_IMPORT_PARALLEL_CHUNKS × BULK_IMPORT_CONCURRENCY`.
 *
 * Kept conservative (3) so even small DB pools can absorb us; the dialog
 * UI accepts an override for tests/perf experiments.
 */
export const BULK_IMPORT_PARALLEL_CHUNKS = 3;

/**
 * Run an async task over each item in `items` with at most `limit`
 * tasks in flight at once. Resolves to results in input order. Failures
 * surface via the per-item Result wrapper so a single rejection cannot
 * abort the rest of the batch.
 */
export async function runWithConcurrency<T, R>(
  items: T[],
  limit: number,
  task: (item: T, index: number) => Promise<R>,
  onSettled?: (index: number) => void,
): Promise<Array<{ ok: true; value: R } | { ok: false; error: unknown }>> {
  const cap = Math.max(1, Math.floor(limit));
  const results: Array<{ ok: true; value: R } | { ok: false; error: unknown }> = new Array(
    items.length,
  );
  let cursor = 0;

  async function worker() {
    for (;;) {
      const idx = cursor++;
      if (idx >= items.length) return;
      try {
        const value = await task(items[idx], idx);
        results[idx] = { ok: true, value };
      } catch (error) {
        results[idx] = { ok: false, error };
      } finally {
        onSettled?.(idx);
      }
    }
  }

  const workers = Array.from({ length: Math.min(cap, items.length) }, () => worker());
  await Promise.all(workers);
  return results;
}
