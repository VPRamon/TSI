/**
 * Tests for the redesigned multi-schedule Compare page.
 *
 * Route: /schedules/:scheduleId/compare/:otherIds
 *
 * Tests:
 *   - Empty state (no otherIds) shows select-schedules prompt
 *   - With 2+ IDs renders summary table metric labels
 *   - Schedule chip header renders schedule names
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import { MemoryRouterProvider, userEvent } from '../../test/test-utils';
import { Routes, Route } from 'react-router-dom';
import type { ScheduleAnalysisData } from '@/features/schedules';

// ─── Mocks ───────────────────────────────────────────────────────────────────

vi.mock('react-plotly.js', () => ({ default: () => null }));
vi.mock('plotly.js-dist-min', () => ({
  default: { newPlot: vi.fn(), react: vi.fn(), purge: vi.fn() },
}));

vi.mock('../../hooks', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../hooks')>();
  return {
    ...actual,
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

vi.mock('@/hooks/useApi', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/hooks/useApi')>();
  return {
    ...actual,
    useSchedules: vi.fn(),
  };
});

import * as hooks from '../../hooks';
import * as schedulesFeature from '../../features/schedules';
import * as apiHooks from '@/hooks/useApi';
import Compare from '../Compare';

// ─── Realistic stub data ──────────────────────────────────────────────────────

function makeScheduleAnalysisData(
  scheduleId: number,
  name: string,
  scheduledCount: number
): ScheduleAnalysisData {
  const blocks = Array.from({ length: scheduledCount + 2 }, (_, i) => ({
    scheduling_block_id: i + 1,
    original_block_id: `OB-${i + 1}`,
    block_name: `Block ${i + 1}`,
    priority: 10 - i,
    total_visibility_hours: 4,
    requested_hours: 2,
    elevation_range_deg: 45,
    scheduled: i < scheduledCount,
    scheduled_start_mjd: i < scheduledCount ? 60000 + i : null,
    scheduled_stop_mjd: i < scheduledCount ? 60001 + i : null,
  }));

  return {
    id: scheduleId,
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
        mean_priority: 8,
        median_priority: 8,
        mean_priority_scheduled: 9,
        mean_priority_unscheduled: 6,
        total_visibility_hours: 40,
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
      schedule_id: scheduleId,
      schedule_name: name,
      schedule_window: { start_mjd: 59000, end_mjd: 60000 },
      operable_periods: [],
      operable_source: 'dark_time',
      segments: [],
      largest_gaps: [],
      reason_breakdown: [],
      unscheduled_reasons: [],
      metrics: {
        schedule_hours: 200,
        requested_hours: 150,
        operable_hours: 180,
        scheduled_hours: 120,
        idle_operable_hours: 60,
        raw_visibility_coverage_hours: 170,
        fit_visibility_coverage_hours: 150,
        gap_count: 5,
        gap_mean_hours: 2.5,
        gap_median_hours: 2.0,
        gap_std_dev_hours: 0.8,
        gap_p90_hours: 4.0,
        largest_gap_hours: 6.0,
        scheduled_fraction_of_operable: 0.67,
        idle_fraction_of_operable: 0.33,
        raw_visibility_fraction_of_operable: 0.94,
        fit_visibility_fraction_of_operable: 0.83,
      },
    },
  };
}

function makeLoadingAnalysisData(scheduleId: number): ScheduleAnalysisData {
  return {
    id: scheduleId,
    name: `Schedule #${scheduleId}`,
    insights: undefined,
    fragmentation: undefined,
    isLoading: true,
    error: null,
  };
}

// ─── Test helpers ─────────────────────────────────────────────────────────────

function renderCompare(path = '/schedules/1/compare') {
  return render(
    <MemoryRouterProvider initialEntries={[path]}>
      <Routes>
        <Route path="/schedules/:scheduleId/compare" element={<Compare />} />
        <Route path="/schedules/:scheduleId/compare/:otherIds" element={<Compare />} />
      </Routes>
    </MemoryRouterProvider>
  );
}

beforeEach(() => {
  const schedulesResult = {
    schedules: [
      { schedule_id: 1, schedule_name: 'Schedule Alpha' },
      { schedule_id: 2, schedule_name: 'Schedule Beta' },
    ],
    total: 2,
  };

  vi.mocked(hooks.useSchedules).mockReturnValue({
    data: schedulesResult,
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  } as unknown as ReturnType<typeof hooks.useSchedules>);
  vi.mocked(apiHooks.useSchedules).mockReturnValue({
    data: schedulesResult,
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  } as unknown as ReturnType<typeof apiHooks.useSchedules>);

  vi.mocked(schedulesFeature.useScheduleAnalysisData).mockImplementation((ids: number[]) =>
    ids.map((id) => {
      if (id === 1) return makeScheduleAnalysisData(1, 'Schedule Alpha', 3);
      if (id === 2) return makeScheduleAnalysisData(2, 'Schedule Beta', 2);
      return makeScheduleAnalysisData(id, `Schedule #${id}`, 1);
    })
  );
});

// ─── Tests ───────────────────────────────────────────────────────────────────

describe('Compare page — empty state', () => {
  it('shows select-schedules prompt when no IDs in URL', () => {
    renderCompare('/schedules/1/compare');
    expect(screen.getByText(/add schedules to compare/i)).toBeInTheDocument();
  });

  it('shows select-schedules prompt when only reference in URL', () => {
    renderCompare('/schedules/1/compare');
    expect(screen.getByText(/add schedules to compare/i)).toBeInTheDocument();
  });

  it('renders "Compare Schedules" page title', () => {
    renderCompare('/schedules/1/compare');
    expect(screen.getByText('Compare Schedules')).toBeInTheDocument();
  });
});

describe('Compare page — 2+ schedules', () => {
  it('renders summary metrics table with expected metric labels', () => {
    renderCompare('/schedules/1/compare/2');

    const expectedLabels = [
      'Scheduled tasks',
      'Unscheduled tasks',
      'Scheduling rate',
      'Cumulative priority',
      'Mean priority (sched.)',
      'Scheduled hours',
      'Operable hours',
      'Gap count',
      'Gap mean',
      'Gap p90',
      'Largest gap',
    ];

    for (const label of expectedLabels) {
      expect(screen.getByText(label)).toBeInTheDocument();
    }
  });

  it('renders schedule chip header with schedule names', () => {
    renderCompare('/schedules/1/compare/2');
    expect(screen.getAllByText(/Schedule Alpha/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Schedule Beta/).length).toBeGreaterThan(0);
  });

  it('renders "Summary Metrics" panel title', () => {
    renderCompare('/schedules/1/compare/2');
    expect(screen.getByText('Summary Metrics')).toBeInTheDocument();
  });

  it('orders summary schedule columns by selected metric while keeping reference fixed', async () => {
    const user = userEvent.setup();

    const schedulesResult = {
      schedules: [
        { schedule_id: 1, schedule_name: 'Schedule Alpha' },
        { schedule_id: 2, schedule_name: 'Schedule Beta' },
        { schedule_id: 3, schedule_name: 'Schedule Gamma' },
      ],
      total: 3,
    };

    vi.mocked(hooks.useSchedules).mockReturnValue({
      data: schedulesResult,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as unknown as ReturnType<typeof hooks.useSchedules>);
    vi.mocked(apiHooks.useSchedules).mockReturnValue({
      data: schedulesResult,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as unknown as ReturnType<typeof apiHooks.useSchedules>);

    vi.mocked(schedulesFeature.useScheduleAnalysisData).mockImplementation((ids: number[]) =>
      ids.map((id) => {
        if (id === 1) return makeScheduleAnalysisData(1, 'Schedule Alpha', 3);
        if (id === 2) return makeScheduleAnalysisData(2, 'Schedule Beta', 1);
        if (id === 3) return makeScheduleAnalysisData(3, 'Schedule Gamma', 4);
        return makeScheduleAnalysisData(id, `Schedule #${id}`, 1);
      })
    );

    renderCompare('/schedules/1/compare/2,3');

    await user.selectOptions(screen.getByLabelText(/order schedules by metric/i), 'Scheduled tasks');

    const summaryTable = screen
      .getByRole('heading', { name: 'Summary Metrics' })
      .closest('section')
      ?.querySelector('table');
    expect(summaryTable).toBeTruthy();

    const getSummaryHeaders = () =>
      within(summaryTable as HTMLTableElement)
        .getAllByRole('columnheader')
        .map((header) => header.textContent?.replace(/\s+/g, ' ').trim() ?? '');

    expect(getSummaryHeaders()[1]).toContain('Schedule Alpha');
    expect(getSummaryHeaders()[2]).toContain('Schedule Gamma');
    expect(getSummaryHeaders()[3]).toContain('Schedule Beta');

    await user.click(
      screen.getByRole('button', { name: /toggle summary schedule sort direction/i })
    );

    expect(getSummaryHeaders()[1]).toContain('Schedule Alpha');
    expect(getSummaryHeaders()[2]).toContain('Schedule Beta');
    expect(getSummaryHeaders()[3]).toContain('Schedule Gamma');
  });

  it('toggles full screen for compare tables', async () => {
    const user = userEvent.setup();

    renderCompare('/schedules/1/compare/2');

    const openButtons = screen.getAllByRole('button', { name: /enter full screen for/i });
    expect(openButtons).toHaveLength(2);

    await user.click(
      screen.getByRole('button', { name: /enter full screen for summary metrics/i })
    );

    expect(
      screen.getByRole('button', { name: /exit full screen for summary metrics/i })
    ).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /exit full screen for summary metrics/i }));

    expect(
      screen.getByRole('button', { name: /enter full screen for summary metrics/i })
    ).toBeInTheDocument();
  });

  it('keeps the route schedule fixed as reference and out of the add-schedule dropdown', async () => {
    const user = userEvent.setup();

    renderCompare('/schedules/1/compare/2');

    expect(screen.queryByLabelText('Remove reference Schedule Alpha')).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /\+ add schedule/i }));
    await user.click(screen.getByRole('button', { name: /open dropdown/i }));

    const listbox = await screen.findByRole('listbox');
    expect(within(listbox).queryByText('Schedule Alpha')).not.toBeInTheDocument();
    expect(await within(listbox).findByText('Schedule Beta')).toBeInTheDocument();
  });

  it('renders block status table panel', () => {
    renderCompare('/schedules/1/compare/2');
    expect(screen.getByText(/Block Status/i)).toBeInTheDocument();
  });

  it('renders duration column in minutes in the block table', () => {
    renderCompare('/schedules/1/compare/2');
    expect(screen.getByText('Duration (min)')).toBeInTheDocument();
    expect(screen.getAllByText('120').length).toBeGreaterThan(0);
  });

  it('renders block IDs from insights data in the block table', () => {
    renderCompare('/schedules/1/compare/2');
    // Blocks from makeInsightsData have original_block_id OB-1, OB-2, etc.
    expect(screen.getAllByText(/^OB-/).length).toBeGreaterThan(0);
  });

  it('collapses duplicated colon-separated block IDs in the block table display', () => {
    vi.mocked(schedulesFeature.useScheduleAnalysisData).mockImplementation((ids: number[]) =>
      ids.map((id) => {
        const data = makeScheduleAnalysisData(
          id,
          id === 1 ? 'Schedule Alpha' : 'Schedule Beta',
          id === 1 ? 3 : 2
        );
        if (data.insights) {
          data.insights.blocks[0].original_block_id = '1000002306:1000002306';
        }
        return data;
      })
    );

    renderCompare('/schedules/1/compare/2');
    expect(screen.getByText('1000002306')).toBeInTheDocument();
    expect(screen.queryByText('1000002306:1000002306')).not.toBeInTheDocument();
  });

  it('sorts the block status table by Block ID when the header is clicked', async () => {
    const user = userEvent.setup();

    vi.mocked(schedulesFeature.useScheduleAnalysisData).mockImplementation((ids: number[]) =>
      ids.map((id) => {
        const data = makeScheduleAnalysisData(
          id,
          id === 1 ? 'Schedule Alpha' : 'Schedule Beta',
          id === 1 ? 3 : 2
        );
        if (data.insights) {
          data.insights.blocks = [
            {
              ...data.insights.blocks[0],
              original_block_id: 'OB-10',
              priority: 9,
              requested_hours: 2,
              scheduled: true,
              scheduled_start_mjd: 60010,
            },
            {
              ...data.insights.blocks[1],
              original_block_id: 'OB-2',
              priority: 7,
              requested_hours: 1,
              scheduled: true,
              scheduled_start_mjd: 60002,
            },
            {
              ...data.insights.blocks[2],
              original_block_id: 'OB-1',
              priority: 8,
              requested_hours: 3,
              scheduled: true,
              scheduled_start_mjd: 60001,
            },
          ];
        }
        return data;
      })
    );

    renderCompare('/schedules/1/compare/2');

    const blockIdHeader = screen.getByRole('columnheader', { name: /block id/i });
    const blockTable = blockIdHeader.closest('table');
    expect(blockTable).toBeTruthy();

    const getVisibleBlockIds = () =>
      within(blockTable as HTMLTableElement)
        .getAllByRole('row')
        .slice(1)
        .map((row) => within(row).getAllByRole('cell')[0]?.textContent?.trim());

    expect(getVisibleBlockIds().slice(0, 3)).toEqual(['OB-10', 'OB-1', 'OB-2']);

    await user.click(within(blockIdHeader).getByRole('button', { name: /block id/i }));
    expect(getVisibleBlockIds().slice(0, 3)).toEqual(['OB-1', 'OB-2', 'OB-10']);

    await user.click(within(blockIdHeader).getByRole('button', { name: /block id/i }));
    expect(getVisibleBlockIds().slice(0, 3)).toEqual(['OB-10', 'OB-2', 'OB-1']);
  });
});

describe('Compare page — loading state', () => {
  it('shows loading spinner while data is loading', () => {
    vi.mocked(schedulesFeature.useScheduleAnalysisData).mockImplementation((ids: number[]) =>
      ids.map((id) => makeLoadingAnalysisData(id))
    );
    renderCompare('/schedules/1/compare/2');
    // Loading spinner should be present (LoadingSpinner has role="status")
    expect(document.querySelector('[role="status"]')).toBeTruthy();
  });
});
