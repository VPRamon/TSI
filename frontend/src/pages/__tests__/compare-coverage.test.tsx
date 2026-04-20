/**
 * Render tests for the Cumulative Priority Coverage panel in Compare.tsx.
 *
 * The panel and its sub-components are not exported, so we test through
 * the compare helper outputs (unit-tested separately) and by rendering
 * the full page with mocked hooks.
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen } from '@testing-library/react';
import { render } from '../../test/test-utils';
import {
  deriveCoverageSeries,
  deriveSummary,
  deriveDisagreements,
} from '../../lib/cumulativeCoverage';
import type { CompareBlock } from '../../api/types';

// ─── Shared test data ───────────────────────────────────────────────

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

const currentBlocks: CompareBlock[] = [
  makeBlock('A', 10, true, 'Alpha'),
  makeBlock('B', 8, false, 'Beta'),
  makeBlock('C', 6, true, 'Gamma'),
  makeBlock('D', 4, true, 'Delta'),
];

const comparisonBlocks: CompareBlock[] = [
  makeBlock('A', 10, false, 'Alpha'),
  makeBlock('B', 8, true, 'Beta'),
  makeBlock('C', 6, true, 'Gamma'),
  makeBlock('D', 4, false, 'Delta'),
];

const commonIds = ['A', 'B', 'C', 'D'];

// ─── Helper-level "render" assertions (no DOM needed) ────────────────

describe('Coverage panel derived metrics', () => {
  const points = deriveCoverageSeries(currentBlocks, comparisonBlocks, commonIds);
  const summary = deriveSummary(points);

  it('matched_task_count equals commonIds length', () => {
    expect(summary.matched_task_count).toBe(4);
  });

  it('total_matched_priority is sum of per-task max priorities', () => {
    // max priorities: A=10, B=8, C=6, D=4 → 28
    expect(summary.total_matched_priority).toBeCloseTo(28);
  });

  it('lead_after_top10 is clamped to available task count (4)', () => {
    // All 4 tasks fit within top-10 window — uses index 3 (last point)
    const last = points[points.length - 1];
    expect(summary.lead_after_top10).toBeCloseTo(
      last.comparison_cumulative - last.current_cumulative,
    );
  });

  it('final_delta equals comparison minus current at last rank', () => {
    const last = points[points.length - 1];
    expect(summary.final_delta).toBeCloseTo(
      last.comparison_cumulative - last.current_cumulative,
    );
  });
});

describe('Disagreement table derived rows', () => {
  const rows = deriveDisagreements(currentBlocks, comparisonBlocks, commonIds);

  it('shows highest-priority status flips first', () => {
    // A (priority 10) and B (priority 8) and D (priority 4) differ
    expect(rows[0].original_block_id).toBe('A');
    expect(rows[0].priority).toBeCloseTo(10);
  });

  it('excludes tasks with matching status', () => {
    // C is scheduled in both → excluded
    const cIds = rows.map((r) => r.original_block_id);
    expect(cIds).not.toContain('C');
  });

  it('includes all disagreeing tasks', () => {
    expect(rows).toHaveLength(3); // A, B, D disagree
  });

  it('current and comparison scheduled flags are correct', () => {
    const a = rows.find((r) => r.original_block_id === 'A')!;
    expect(a.current_scheduled).toBe(true);
    expect(a.comparison_scheduled).toBe(false);
  });
});

// ─── Smoke test: panel caption and title via mocked hooks ─────────────

vi.mock('react-plotly.js', () => ({ default: () => null }));
vi.mock('plotly.js-dist-min', () => ({
  default: { newPlot: vi.fn(), react: vi.fn(), purge: vi.fn() },
}));
vi.mock('../../hooks', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../hooks')>();
  return {
    ...actual,
    useCompare: vi.fn(),
    useFragmentation: vi.fn(() => ({ data: undefined, isLoading: false, error: null })),
    useSchedules: vi.fn(() => ({ data: undefined })),
  };
});

import * as hooks from '../../hooks';
import Compare from '../Compare';

const mockCompareData = {
  current_blocks: currentBlocks,
  comparison_blocks: comparisonBlocks,
  current_stats: {
    scheduled_count: 3,
    unscheduled_count: 1,
    total_priority: 20,
    mean_priority: 5,
    median_priority: 5,
    total_hours: 3,
    gap_count: null,
    gap_mean_hours: null,
    gap_median_hours: null,
  },
  comparison_stats: {
    scheduled_count: 2,
    unscheduled_count: 2,
    total_priority: 14,
    mean_priority: 7,
    median_priority: 7,
    total_hours: 2,
    gap_count: null,
    gap_mean_hours: null,
    gap_median_hours: null,
  },
  common_ids: commonIds,
  only_in_current: [],
  only_in_comparison: [],
  scheduling_changes: [],
  scheduled_only_current: [],
  scheduled_only_comparison: [],
  only_in_current_blocks: [],
  only_in_comparison_blocks: [],
  retimed_blocks: [],
  current_name: 'Schedule A',
  comparison_name: 'Schedule B',
};

describe('Compare page coverage panel smoke test', () => {
  beforeEach(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    vi.mocked(hooks.useCompare).mockReturnValue({ data: mockCompareData, isLoading: false, error: null, refetch: vi.fn() } as any);
  });

  it('renders the panel title', () => {
    render(<Compare />);
    expect(screen.getByText('Cumulative Priority Coverage')).toBeInTheDocument();
  });

  it('renders the matched-tasks caption', () => {
    render(<Compare />);
    expect(screen.getByText(/matched tasks only/i)).toBeInTheDocument();
  });

  it('renders matched tasks metric', () => {
    render(<Compare />);
    expect(screen.getByText('Matched tasks')).toBeInTheDocument();
    expect(screen.getByText('4')).toBeInTheDocument();
  });

  it('renders disagreements table with highest-priority flip first', () => {
    render(<Compare />);
    expect(screen.getByText(/Highest-Priority Disagreements/i)).toBeInTheDocument();
    const cells = screen.getAllByText('A');
    expect(cells.length).toBeGreaterThan(0);
  });
});
