/**
 * Cumulative priority coverage helpers for the Compare page.
 *
 * All derivations operate on the existing compare payload — no backend
 * changes are needed or made.
 */
import type { CompareBlock } from '@/api/types';

export interface CoveragePoint {
  rank: number;
  original_block_id: string;
  block_name: string;
  priority: number;
  current_scheduled: boolean;
  comparison_scheduled: boolean;
  current_increment: number;
  comparison_increment: number;
  current_cumulative: number;
  comparison_cumulative: number;
}

export interface DisagreementRow {
  original_block_id: string;
  block_name: string;
  priority: number;
  current_scheduled: boolean;
  comparison_scheduled: boolean;
  current_start_mjd: number | null;
  current_stop_mjd: number | null;
  comparison_start_mjd: number | null;
  comparison_stop_mjd: number | null;
}

export interface CoverageSummary {
  matched_task_count: number;
  total_matched_priority: number;
  lead_after_top10: number;
  final_delta: number;
}

function buildMap(blocks: CompareBlock[]): Map<string, CompareBlock> {
  const map = new Map<string, CompareBlock>();
  for (const b of blocks) {
    if (b.original_block_id) map.set(b.original_block_id, b);
  }
  return map;
}

/**
 * Derive ranked cumulative coverage series for matched tasks only.
 *
 * Ordering: descending max(current.priority, comparison.priority), then
 * original_block_id ascending for stable ties.
 */
export function deriveCoverageSeries(
  currentBlocks: CompareBlock[],
  comparisonBlocks: CompareBlock[],
  commonIds: string[],
): CoveragePoint[] {
  const curMap = buildMap(currentBlocks);
  const cmpMap = buildMap(comparisonBlocks);

  const ranked = [...commonIds].sort((a, b) => {
    const pa = Math.max(curMap.get(a)?.priority ?? 0, cmpMap.get(a)?.priority ?? 0);
    const pb = Math.max(curMap.get(b)?.priority ?? 0, cmpMap.get(b)?.priority ?? 0);
    if (pb !== pa) return pb - pa;
    return a < b ? -1 : a > b ? 1 : 0;
  });

  const points: CoveragePoint[] = [];
  let curCum = 0;
  let cmpCum = 0;

  for (let i = 0; i < ranked.length; i++) {
    const id = ranked[i];
    const cur = curMap.get(id);
    const cmp = cmpMap.get(id);

    const priority = Math.max(cur?.priority ?? 0, cmp?.priority ?? 0);
    const curSched = cur?.scheduled ?? false;
    const cmpSched = cmp?.scheduled ?? false;

    const curInc = curSched ? (cur?.priority ?? 0) : 0;
    const cmpInc = cmpSched ? (cmp?.priority ?? 0) : 0;

    curCum += curInc;
    cmpCum += cmpInc;

    points.push({
      rank: i + 1,
      original_block_id: id,
      block_name: cur?.block_name ?? cmp?.block_name ?? '',
      priority,
      current_scheduled: curSched,
      comparison_scheduled: cmpSched,
      current_increment: curInc,
      comparison_increment: cmpInc,
      current_cumulative: curCum,
      comparison_cumulative: cmpCum,
    });
  }

  return points;
}

export function deriveSummary(points: CoveragePoint[]): CoverageSummary {
  const n = points.length;
  const totalPriority = points.reduce((s, p) => s + p.priority, 0);

  const top10 = Math.min(10, n);
  const leadAfterTop10 =
    top10 > 0
      ? points[top10 - 1].comparison_cumulative - points[top10 - 1].current_cumulative
      : 0;

  const finalDelta =
    n > 0
      ? points[n - 1].comparison_cumulative - points[n - 1].current_cumulative
      : 0;

  return {
    matched_task_count: n,
    total_matched_priority: totalPriority,
    lead_after_top10: leadAfterTop10,
    final_delta: finalDelta,
  };
}

export function deriveDisagreements(
  currentBlocks: CompareBlock[],
  comparisonBlocks: CompareBlock[],
  commonIds: string[],
): DisagreementRow[] {
  const curMap = buildMap(currentBlocks);
  const cmpMap = buildMap(comparisonBlocks);

  const rows: DisagreementRow[] = [];

  for (const id of commonIds) {
    const cur = curMap.get(id);
    const cmp = cmpMap.get(id);
    if (!cur || !cmp) continue;
    if (cur.scheduled === cmp.scheduled) continue;

    rows.push({
      original_block_id: id,
      block_name: cur.block_name || cmp.block_name,
      priority: Math.max(cur.priority, cmp.priority),
      current_scheduled: cur.scheduled,
      comparison_scheduled: cmp.scheduled,
      current_start_mjd: cur.scheduled_start_mjd,
      current_stop_mjd: cur.scheduled_stop_mjd,
      comparison_start_mjd: cmp.scheduled_start_mjd,
      comparison_stop_mjd: cmp.scheduled_stop_mjd,
    });
  }

  return rows.sort((a, b) => b.priority - a.priority);
}
