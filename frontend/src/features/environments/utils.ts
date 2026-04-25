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
