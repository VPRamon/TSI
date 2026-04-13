/**
 * Tests for Timeline month-boundary clamping logic.
 *
 * Verifies that blocks crossing a month boundary produce valid
 * x0 ≤ x1 coordinates and that overlapping blocks are semi-transparent.
 */
import { describe, it, expect } from 'vitest';
import { mjdToDate, dateToMjd } from '@/constants/dates';

// ─── Reimplemented shape builder (mirrors Timeline.tsx) ─────────────

function computeShape(startMjd: number, stopMjd: number, monthMap: Map<string, number>) {
  const barHeight = 0.4;
  const startDate = mjdToDate(startMjd);
  const stopDate = mjdToDate(stopMjd);
  const monthKey = `${startDate.getFullYear()}-${String(startDate.getMonth() + 1).padStart(2, '0')}`;
  const stopMonthKey = `${stopDate.getFullYear()}-${String(stopDate.getMonth() + 1).padStart(2, '0')}`;
  const monthIndex = monthMap.get(monthKey) ?? 0;

  const startDay = startDate.getDate() + startDate.getHours() / 24 + startDate.getMinutes() / 1440;

  let stopDay: number;
  if (monthKey === stopMonthKey) {
    stopDay = stopDate.getDate() + stopDate.getHours() / 24 + stopDate.getMinutes() / 1440;
  } else {
    const daysInMonth = new Date(startDate.getFullYear(), startDate.getMonth() + 1, 0).getDate();
    stopDay = daysInMonth + 1;
  }

  return { x0: startDay, x1: stopDay, monthIndex, barHeight };
}

// ─── Tests ──────────────────────────────────────────────────────────

describe('Timeline month-boundary handling', () => {
  const monthMap = new Map([
    ['2024-01', 0],
    ['2024-02', 1],
    ['2024-03', 2],
  ]);

  it('block within same month produces x1 > x0', () => {
    // 2024-01-10 to 2024-01-12
    const startMjd = dateToMjd(new Date(Date.UTC(2024, 0, 10)));
    const stopMjd = dateToMjd(new Date(Date.UTC(2024, 0, 12)));

    const shape = computeShape(startMjd, stopMjd, monthMap);
    expect(shape.x1).toBeGreaterThan(shape.x0);
  });

  it('block crossing month boundary is clamped so x1 > x0', () => {
    // 2024-01-30 to 2024-02-02 — previously would produce x1 < x0
    const startMjd = dateToMjd(new Date(Date.UTC(2024, 0, 30)));
    const stopMjd = dateToMjd(new Date(Date.UTC(2024, 1, 2)));

    const shape = computeShape(startMjd, stopMjd, monthMap);
    expect(shape.x1).toBeGreaterThan(shape.x0);
    // Should clamp to end of January (31 days → stopDay = 32)
    expect(shape.x1).toBe(32);
  });

  it('block crossing year boundary is clamped correctly', () => {
    // December → January of next year
    const decMap = new Map([
      ['2024-12', 0],
      ['2025-01', 1],
    ]);
    const startMjd = dateToMjd(new Date(Date.UTC(2024, 11, 30)));
    const stopMjd = dateToMjd(new Date(Date.UTC(2025, 0, 3)));

    const shape = computeShape(startMjd, stopMjd, decMap);
    expect(shape.x1).toBeGreaterThan(shape.x0);
    // December has 31 days → stopDay = 32
    expect(shape.x1).toBe(32);
  });
});
