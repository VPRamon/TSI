import { describe, it, expect } from 'vitest';
import {
  deriveCoverageSeries,
  deriveSummary,
  deriveDisagreements,
} from './cumulativeCoverage';
import type { CompareBlock } from '@/api/types';

function makeBlock(
  id: string,
  priority: number,
  scheduled: boolean,
  name = '',
): CompareBlock {
  return {
    scheduling_block_id: `sched-${id}`,
    original_block_id: id,
    block_name: name || id,
    priority,
    scheduled,
    requested_hours: 1,
    scheduled_start_mjd: scheduled ? 60000 : null,
    scheduled_stop_mjd: scheduled ? 60001 : null,
  };
}

describe('deriveCoverageSeries', () => {
  it('ranks tasks by descending priority', () => {
    const cur = [makeBlock('A', 10, true), makeBlock('B', 5, true)];
    const cmp = [makeBlock('A', 10, true), makeBlock('B', 5, false)];
    const ids = ['A', 'B'];

    const pts = deriveCoverageSeries(cur, cmp, ids);

    expect(pts[0].original_block_id).toBe('A');
    expect(pts[1].original_block_id).toBe('B');
  });

  it('one schedule jumps early on top-priority tasks while the other catches up later', () => {
    const cur = [makeBlock('A', 10, true), makeBlock('B', 5, false)];
    const cmp = [makeBlock('A', 10, false), makeBlock('B', 5, true)];
    const ids = ['A', 'B'];

    const pts = deriveCoverageSeries(cur, cmp, ids);

    // After rank 1 (A): current=10, comparison=0
    expect(pts[0].current_cumulative).toBe(10);
    expect(pts[0].comparison_cumulative).toBe(0);

    // After rank 2 (B): current=10, comparison=5
    expect(pts[1].current_cumulative).toBe(10);
    expect(pts[1].comparison_cumulative).toBe(5);
  });

  it('tasks unscheduled in both schedules create flat segments', () => {
    const cur = [makeBlock('A', 10, true), makeBlock('B', 8, false)];
    const cmp = [makeBlock('A', 10, true), makeBlock('B', 8, false)];
    const ids = ['A', 'B'];

    const pts = deriveCoverageSeries(cur, cmp, ids);

    // B is unscheduled in both — cumulative should not change at rank 2
    expect(pts[1].current_cumulative).toBe(pts[0].current_cumulative);
    expect(pts[1].comparison_cumulative).toBe(pts[0].comparison_cumulative);
    expect(pts[1].current_increment).toBe(0);
    expect(pts[1].comparison_increment).toBe(0);
  });

  it('stable ordering for equal priorities (alphabetical by id)', () => {
    const cur = [makeBlock('Z', 5, true), makeBlock('A', 5, true)];
    const cmp = [makeBlock('Z', 5, true), makeBlock('A', 5, true)];
    const ids = ['Z', 'A'];

    const pts = deriveCoverageSeries(cur, cmp, ids);

    expect(pts[0].original_block_id).toBe('A');
    expect(pts[1].original_block_id).toBe('Z');
  });

  it('uses max(current.priority, comparison.priority) for ordering', () => {
    // B has higher comparison priority — should rank higher despite lower current priority
    const cur = [makeBlock('A', 10, true), makeBlock('B', 3, true)];
    const cmp = [makeBlock('A', 10, true), makeBlock('B', 12, true)];
    const ids = ['A', 'B'];

    const pts = deriveCoverageSeries(cur, cmp, ids);

    expect(pts[0].original_block_id).toBe('B'); // max priority = 12
    expect(pts[1].original_block_id).toBe('A'); // max priority = 10
  });

  it('schedule-exclusive tasks (not in commonIds) do not appear in the curve', () => {
    const cur = [makeBlock('A', 10, true), makeBlock('EXCL', 20, true)];
    const cmp = [makeBlock('A', 10, true)];
    const ids = ['A']; // EXCL is not a common id

    const pts = deriveCoverageSeries(cur, cmp, ids);

    expect(pts).toHaveLength(1);
    expect(pts[0].original_block_id).toBe('A');
  });

  it('returns empty array when commonIds is empty', () => {
    const pts = deriveCoverageSeries([], [], []);
    expect(pts).toHaveLength(0);
  });
});

describe('deriveSummary', () => {
  it('computes matched_task_count correctly', () => {
    const cur = [makeBlock('A', 5, true), makeBlock('B', 3, true)];
    const cmp = [makeBlock('A', 5, true), makeBlock('B', 3, true)];
    const pts = deriveCoverageSeries(cur, cmp, ['A', 'B']);
    const s = deriveSummary(pts);
    expect(s.matched_task_count).toBe(2);
  });

  it('computes total_matched_priority as sum of per-task max priorities', () => {
    const cur = [makeBlock('A', 5, true), makeBlock('B', 3, false)];
    const cmp = [makeBlock('A', 5, true), makeBlock('B', 3, false)];
    const pts = deriveCoverageSeries(cur, cmp, ['A', 'B']);
    const s = deriveSummary(pts);
    expect(s.total_matched_priority).toBeCloseTo(8);
  });

  it('computes lead_after_top10 clamped to available tasks', () => {
    // Only 2 tasks — top-10 window = min(10, 2) = 2
    const cur = [makeBlock('A', 10, true), makeBlock('B', 5, false)];
    const cmp = [makeBlock('A', 10, false), makeBlock('B', 5, true)];
    const pts = deriveCoverageSeries(cur, cmp, ['A', 'B']);
    const s = deriveSummary(pts);
    // After 2 tasks: comparison_cumulative=5, current_cumulative=10 → lead = 5-10 = -5
    expect(s.lead_after_top10).toBeCloseTo(-5);
  });

  it('final_delta equals comparison minus current at last rank', () => {
    const cur = [makeBlock('A', 10, true), makeBlock('B', 5, true)];
    const cmp = [makeBlock('A', 10, true), makeBlock('B', 5, false)];
    const pts = deriveCoverageSeries(cur, cmp, ['A', 'B']);
    const s = deriveSummary(pts);
    const last = pts[pts.length - 1];
    expect(s.final_delta).toBeCloseTo(last.comparison_cumulative - last.current_cumulative);
  });

  it('returns zeros for empty series', () => {
    const s = deriveSummary([]);
    expect(s.matched_task_count).toBe(0);
    expect(s.total_matched_priority).toBe(0);
    expect(s.lead_after_top10).toBe(0);
    expect(s.final_delta).toBe(0);
  });
});

describe('deriveDisagreements', () => {
  it('includes only tasks with differing scheduled status', () => {
    const cur = [makeBlock('A', 10, true), makeBlock('B', 5, true), makeBlock('C', 3, false)];
    const cmp = [makeBlock('A', 10, false), makeBlock('B', 5, true), makeBlock('C', 3, false)];
    const ids = ['A', 'B', 'C'];

    const rows = deriveDisagreements(cur, cmp, ids);

    expect(rows).toHaveLength(1);
    expect(rows[0].original_block_id).toBe('A');
  });

  it('sorts by descending priority', () => {
    const cur = [makeBlock('A', 3, true), makeBlock('B', 10, true)];
    const cmp = [makeBlock('A', 3, false), makeBlock('B', 10, false)];
    const ids = ['A', 'B'];

    const rows = deriveDisagreements(cur, cmp, ids);

    expect(rows[0].original_block_id).toBe('B');
    expect(rows[1].original_block_id).toBe('A');
  });

  it('excludes schedule-exclusive tasks (not in commonIds)', () => {
    const cur = [makeBlock('A', 10, true), makeBlock('EXCL', 20, true)];
    const cmp = [makeBlock('A', 10, false)];
    const ids = ['A'];

    const rows = deriveDisagreements(cur, cmp, ids);

    expect(rows).toHaveLength(1);
    expect(rows[0].original_block_id).toBe('A');
  });

  it('returns empty when all statuses agree', () => {
    const cur = [makeBlock('A', 10, true), makeBlock('B', 5, false)];
    const cmp = [makeBlock('A', 10, true), makeBlock('B', 5, false)];
    const rows = deriveDisagreements(cur, cmp, ['A', 'B']);
    expect(rows).toHaveLength(0);
  });
});
