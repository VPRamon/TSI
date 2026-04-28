/**
 * Pure aggregation primitives used by `aggregations.worker.ts`.
 *
 * These functions are deliberately data-only (no DOM, no React) so they can
 * be:
 *   - executed inside a Web Worker via comlink (see `aggregations.worker.ts`),
 *   - imported directly from unit tests (no worker plumbing required),
 *   - or invoked synchronously on the main thread when input sizes are too
 *     small to justify a worker round-trip.
 *
 * Inputs and outputs are restricted to structured-cloneable values (plain
 * objects, arrays, Maps) so the same code path works in either context.
 */

// ---------------------------------------------------------------------------
// computeBlockStatusMap — used by ScheduleComparisonTables.BlockStatusTable
// ---------------------------------------------------------------------------

export interface BlockStatusInputBlock {
  original_block_id: string;
  priority: number;
  requested_hours: number;
  scheduled: boolean;
  scheduled_start_mjd: number | null | undefined;
}

export interface BlockStatusInputSchedule {
  id: number;
  blocks: BlockStatusInputBlock[];
}

export interface BlockStatusEntry {
  original_block_id: string;
  maxPriority: number;
  maxRequestedHours: number;
  perSchedule: Record<
    number,
    {
      scheduled: boolean;
      start_mjd: number | null | undefined;
      requested_hours: number;
    }
  >;
}

/**
 * Build the per-(block × schedule) map for the visible page slice.
 *
 * @param schedules  list of schedules to aggregate over
 * @param pageBlockIds  the original_block_ids that should appear in the result
 */
export function computeBlockStatusMap(
  schedules: BlockStatusInputSchedule[],
  pageBlockIds: string[]
): Map<string, BlockStatusEntry> {
  const ids = new Set(pageBlockIds);
  const map = new Map<string, BlockStatusEntry>();
  if (ids.size === 0) return map;

  for (const schedule of schedules) {
    for (const block of schedule.blocks) {
      const key = block.original_block_id;
      if (!ids.has(key)) continue;

      let entry = map.get(key);
      if (!entry) {
        entry = {
          original_block_id: key,
          maxPriority: block.priority,
          maxRequestedHours: block.requested_hours,
          perSchedule: {},
        };
        map.set(key, entry);
      }

      entry.maxPriority = Math.max(entry.maxPriority, block.priority);
      entry.maxRequestedHours = Math.max(entry.maxRequestedHours, block.requested_hours);
      entry.perSchedule[schedule.id] = {
        scheduled: block.scheduled,
        start_mjd: block.scheduled_start_mjd,
        requested_hours: block.requested_hours,
      };
    }
  }

  return map;
}

// ---------------------------------------------------------------------------
// sortFilterBlocks — used by BlocksTable
// ---------------------------------------------------------------------------

export interface SortFilterBlock {
  scheduling_block_id: number;
  original_block_id: string;
  block_name?: string;
  priority: number;
  scheduled: boolean;
  total_visibility_hours?: number;
  requested_hours?: number;
  num_visibility_periods?: number;
  // Allow callers to round-trip extra fields without losing data.
  [extra: string]: unknown;
}

export type SortFilterField =
  | 'priority'
  | 'scheduled'
  | 'total_visibility_hours'
  | 'requested_hours';

export type SortDirection = 'asc' | 'desc';

export interface SortFilterSpec {
  field: SortFilterField;
  direction: SortDirection;
}

export function sortFilterBlocks<T extends SortFilterBlock>(
  blocks: readonly T[],
  filter: string,
  sort: SortFilterSpec
): T[] {
  let result: T[] = blocks.slice();

  if (filter) {
    const lower = filter.toLowerCase();
    result = result.filter(
      (b) =>
        b.scheduling_block_id.toString().includes(lower) ||
        b.original_block_id.toLowerCase().includes(lower) ||
        (b.block_name?.toLowerCase().includes(lower) ?? false)
    );
  }

  result.sort((a, b) => {
    let aVal: number;
    let bVal: number;
    switch (sort.field) {
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
    const diff = aVal - bVal;
    return sort.direction === 'asc' ? diff : -diff;
  });

  return result;
}

// ---------------------------------------------------------------------------
// binHeatmap — generic 2D histogram bucketing
// ---------------------------------------------------------------------------

export interface HeatmapPoint {
  x: number;
  y: number;
  /** Optional weight, defaults to 1. */
  w?: number;
}

export interface HeatmapResult {
  /** Lower edges of x bins (length = xBins). */
  xEdges: number[];
  /** Lower edges of y bins (length = yBins). */
  yEdges: number[];
  /** Row-major counts/weights, indexed [yBin * xBins + xBin]. */
  counts: number[];
  xBins: number;
  yBins: number;
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
}

export function binHeatmap(
  points: readonly HeatmapPoint[],
  xBins: number,
  yBins: number
): HeatmapResult {
  const xb = Math.max(1, Math.floor(xBins));
  const yb = Math.max(1, Math.floor(yBins));

  if (points.length === 0) {
    return {
      xEdges: Array.from({ length: xb }, (_, i) => i),
      yEdges: Array.from({ length: yb }, (_, i) => i),
      counts: new Array(xb * yb).fill(0),
      xBins: xb,
      yBins: yb,
      xMin: 0,
      xMax: xb,
      yMin: 0,
      yMax: yb,
    };
  }

  let xMin = Infinity;
  let xMax = -Infinity;
  let yMin = Infinity;
  let yMax = -Infinity;
  for (const p of points) {
    if (!Number.isFinite(p.x) || !Number.isFinite(p.y)) continue;
    if (p.x < xMin) xMin = p.x;
    if (p.x > xMax) xMax = p.x;
    if (p.y < yMin) yMin = p.y;
    if (p.y > yMax) yMax = p.y;
  }

  if (!Number.isFinite(xMin)) {
    xMin = 0;
    xMax = xb;
  }
  if (!Number.isFinite(yMin)) {
    yMin = 0;
    yMax = yb;
  }
  // Guard against degenerate axes (all points identical on one axis).
  if (xMin === xMax) xMax = xMin + 1;
  if (yMin === yMax) yMax = yMin + 1;

  const xStep = (xMax - xMin) / xb;
  const yStep = (yMax - yMin) / yb;

  const counts = new Array(xb * yb).fill(0);
  for (const p of points) {
    if (!Number.isFinite(p.x) || !Number.isFinite(p.y)) continue;
    let xi = Math.floor((p.x - xMin) / xStep);
    let yi = Math.floor((p.y - yMin) / yStep);
    if (xi === xb) xi -= 1;
    if (yi === yb) yi -= 1;
    if (xi < 0 || xi >= xb || yi < 0 || yi >= yb) continue;
    counts[yi * xb + xi] += p.w ?? 1;
  }

  const xEdges = Array.from({ length: xb }, (_, i) => xMin + i * xStep);
  const yEdges = Array.from({ length: yb }, (_, i) => yMin + i * yStep);

  return { xEdges, yEdges, counts, xBins: xb, yBins: yb, xMin, xMax, yMin, yMax };
}

// ---------------------------------------------------------------------------
// computeParetoFront — Pareto dominance flags for N-dimensional points
// ---------------------------------------------------------------------------

export type ObjectiveDirection = 'min' | 'max';

/**
 * For each input point, return whether some other point dominates it on the
 * given objectives.  Position-stable: the i-th output flag corresponds to the
 * i-th input row.
 *
 * A point `a` dominates `b` when it is no worse on every objective and
 * strictly better on at least one.
 */
export function computeParetoFront(
  values: readonly (readonly number[])[],
  directions: readonly ObjectiveDirection[]
): boolean[] {
  const n = values.length;
  const k = directions.length;
  const dominated = new Array<boolean>(n).fill(false);

  for (let i = 0; i < n; i += 1) {
    const a = values[i];
    for (let j = 0; j < n; j += 1) {
      if (j === i) continue;
      const b = values[j];
      // Does b dominate a?
      let strictlyBetter = false;
      let worseOnAny = false;
      for (let d = 0; d < k; d += 1) {
        const dir = directions[d];
        const av = a[d];
        const bv = b[d];
        if (dir === 'max') {
          if (bv < av) {
            worseOnAny = true;
            break;
          }
          if (bv > av) strictlyBetter = true;
        } else {
          if (bv > av) {
            worseOnAny = true;
            break;
          }
          if (bv < av) strictlyBetter = true;
        }
      }
      if (!worseOnAny && strictlyBetter) {
        dominated[i] = true;
        break;
      }
    }
  }

  return dominated;
}
