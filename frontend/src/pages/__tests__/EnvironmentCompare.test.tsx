/**
 * Tests for the EnvironmentCompare page.
 *
 * Route: /environments/:envId/compare
 */
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Route, Routes } from 'react-router-dom';
import { MemoryRouterProvider } from '../../test/test-utils';
import type { ScheduleAnalysisData } from '@/features/schedules';
import type { EnvironmentInfo } from '@/api/types';

vi.mock('react-plotly.js', () => ({ default: () => null }));
vi.mock('react-plotly.js/factory', () => ({ default: () => () => null }));
vi.mock('plotly.js-dist-min', () => ({
  default: { newPlot: vi.fn(), react: vi.fn(), purge: vi.fn() },
}));

vi.mock('../../hooks', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../hooks')>();
  return {
    ...actual,
    useEnvironment: vi.fn(),
    useSchedules: vi.fn(),
  };
});

vi.mock('../../features/schedules', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../features/schedules')>();
  return {
    ...actual,
    useScheduleAnalysisData: vi.fn(),
  };
});

import * as hooks from '../../hooks';
import * as schedulesFeature from '../../features/schedules';
import EnvironmentCompare from '../EnvironmentCompare';

function makeAnalysis(id: number, name: string, scheduledCount: number): ScheduleAnalysisData {
  const blocks = Array.from({ length: scheduledCount + 1 }, (_, i) => ({
    scheduling_block_id: i + 1,
    original_block_id: `OB-${i + 1}`,
    block_name: `Block ${i + 1}`,
    priority: 5,
    total_visibility_hours: 4,
    requested_hours: 2,
    elevation_range_deg: 45,
    scheduled: i < scheduledCount,
    scheduled_start_mjd: i < scheduledCount ? 60000 + i : null,
    scheduled_stop_mjd: i < scheduledCount ? 60001 + i : null,
  }));

  return {
    id,
    name,
    isLoading: false,
    error: null,
    insights: {
      blocks,
      metrics: {
        total_observations: blocks.length,
        scheduled_count: scheduledCount,
        unscheduled_count: blocks.length - scheduledCount,
        scheduling_rate: scheduledCount / blocks.length,
        mean_priority: 5,
        median_priority: 5,
        mean_priority_scheduled: 5,
        mean_priority_unscheduled: 5,
        priority_capture_ratio: scheduledCount / blocks.length,
        sum_priority_scheduled: 5 * scheduledCount,
        sum_priority_total: 5 * blocks.length,
        total_visibility_hours: 12,
        mean_requested_hours: 2,
      },
      correlations: [],
      top_priority: [],
      top_visibility: [],
      conflicts: [],
      total_count: blocks.length,
      scheduled_count: scheduledCount,
      impossible_count: 0,
    },
    fragmentation: {
      schedule_id: id,
      schedule_name: name,
      schedule_window: { start_mjd: 59000, end_mjd: 60000 },
      operable_periods: [],
      operable_source: 'dark_time',
      segments: [],
      largest_gaps: [],
      reason_breakdown: [],
      unscheduled_reasons: [],
      metrics: {
        schedule_hours: 100,
        requested_hours: 80,
        operable_hours: 90,
        scheduled_hours: 60,
        idle_operable_hours: 30,
        raw_visibility_coverage_hours: 85,
        fit_visibility_coverage_hours: 75,
        gap_count: 2,
        gap_mean_hours: 1,
        gap_median_hours: 1,
        gap_std_dev_hours: 0.2,
        gap_p90_hours: 1.5,
        largest_gap_hours: 2,
        scheduled_fraction_of_operable: 0.66,
        idle_fraction_of_operable: 0.33,
        raw_visibility_fraction_of_operable: 0.94,
        fit_visibility_fraction_of_operable: 0.83,
      },
    },
  };
}

const baseEnv: EnvironmentInfo = {
  environment_id: 7,
  name: 'CTAO South',
  structure: {
    period_start_mjd: 60000,
    period_end_mjd: 60030,
    lat_deg: -24.6,
    lon_deg: -70.4,
    elevation_m: 2400,
    blocks_hash: 'abc',
  },
  schedule_ids: [10, 11],
  created_at: '2025-01-01T00:00:00Z',
};

function renderPage() {
  return render(
    <MemoryRouterProvider initialEntries={['/environments/7/compare']}>
      <Routes>
        <Route path="/environments/:envId/compare" element={<EnvironmentCompare />} />
      </Routes>
    </MemoryRouterProvider>
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(hooks.useSchedules).mockReturnValue({
    data: {
      schedules: [
        { schedule_id: 10, schedule_name: 'Plan A' },
        { schedule_id: 11, schedule_name: 'Plan B' },
      ],
      total: 2,
    },
    isLoading: false,
    error: null,
  } as unknown as ReturnType<typeof hooks.useSchedules>);
  vi.mocked(schedulesFeature.useScheduleAnalysisData).mockImplementation((ids: number[]) =>
    ids.map((id, idx) => makeAnalysis(id, idx === 0 ? 'Plan A' : 'Plan B', idx + 1))
  );
});

describe('EnvironmentCompare page', () => {
  it('shows a loading spinner while the environment is loading', () => {
    vi.mocked(hooks.useEnvironment).mockReturnValue({
      data: undefined,
      isLoading: true,
      error: null,
    } as unknown as ReturnType<typeof hooks.useEnvironment>);

    renderPage();    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it('renders the env header and comparison sections for the members', () => {
    vi.mocked(hooks.useEnvironment).mockReturnValue({
      data: baseEnv,
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof hooks.useEnvironment>);

    renderPage();

    expect(screen.getByRole('heading', { name: 'CTAO South' })).toBeInTheDocument();
    expect(screen.getByText(/2 schedules/i)).toBeInTheDocument();
    expect(screen.getAllByText('Plan A').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Plan B').length).toBeGreaterThan(0);
  });

  it('shows the empty-state hint when fewer than 2 members are present', () => {
    vi.mocked(hooks.useEnvironment).mockReturnValue({
      data: { ...baseEnv, schedule_ids: [10] },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof hooks.useEnvironment>);

    renderPage();
    expect(screen.getByText(/at least two schedules to compare/i)).toBeInTheDocument();
  });
});
