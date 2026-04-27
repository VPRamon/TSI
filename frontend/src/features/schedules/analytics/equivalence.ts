/**
 * Schedule equivalence helper.
 *
 * Two schedules are considered "equivalent" here when they scheduled the
 * exact same set of scheduling-block ids — timing-agnostic. Use this to
 * collapse duplicate runs in dense comparison panels.
 */
import type { InsightsBlock, InsightsData } from '@/api/types';

/** Stable, order-independent fingerprint of the scheduled block-id set. */
export function fingerprintInsights(insights: InsightsData | undefined | null): string | null {
  if (!insights) return null;
  const ids = insights.blocks
    .filter((b) => b.scheduled)
    .map(blockKey)
    .sort();
  return ids.join('|');
}

function blockKey(b: InsightsBlock): string {
  return b.original_block_id || String(b.scheduling_block_id);
}

export interface EquivalenceGroup<T> {
  fingerprint: string;
  members: T[];
}

export interface EquivalenceResult<T> {
  /** Groups of items that produced the same scheduled set. Singletons included. */
  groups: EquivalenceGroup<T>[];
  /** Per-item fingerprint (null when no insights available). */
  fingerprintOf: Map<T, string | null>;
  /**
   * Map fingerprint → 0-based group index in the order returned by `groups`.
   * Useful for assigning a colour or badge label.
   */
  groupIndex: Map<string, number>;
  /** Items whose insights weren't available; cannot be grouped. */
  ungrouped: T[];
}

/**
 * Group items by the fingerprint produced by `getInsights`.
 *
 * Items with no available insights end up in `ungrouped`; they are *not*
 * included in any equivalence group. Group order matches first-occurrence.
 */
export function groupEquivalentSchedules<T>(
  items: T[],
  getInsights: (item: T) => InsightsData | undefined | null,
): EquivalenceResult<T> {
  const fingerprintOf = new Map<T, string | null>();
  const groupsByFp = new Map<string, T[]>();
  const order: string[] = [];
  const ungrouped: T[] = [];

  for (const item of items) {
    const fp = fingerprintInsights(getInsights(item));
    fingerprintOf.set(item, fp);
    if (fp == null) {
      ungrouped.push(item);
      continue;
    }
    const arr = groupsByFp.get(fp);
    if (arr) {
      arr.push(item);
    } else {
      groupsByFp.set(fp, [item]);
      order.push(fp);
    }
  }

  const groups: EquivalenceGroup<T>[] = order.map((fp) => ({
    fingerprint: fp,
    members: groupsByFp.get(fp) ?? [],
  }));
  const groupIndex = new Map<string, number>();
  groups.forEach((g, i) => groupIndex.set(g.fingerprint, i));

  return { groups, fingerprintOf, groupIndex, ungrouped };
}

/**
 * Pick a single representative per equivalence group, preserving the
 * input order. Items without insights pass through unchanged.
 */
export function collapseToRepresentatives<T>(
  items: T[],
  result: EquivalenceResult<T>,
): T[] {
  const seen = new Set<string>();
  const out: T[] = [];
  for (const item of items) {
    const fp = result.fingerprintOf.get(item);
    if (fp == null) {
      out.push(item);
      continue;
    }
    if (seen.has(fp)) continue;
    seen.add(fp);
    out.push(item);
  }
  return out;
}
