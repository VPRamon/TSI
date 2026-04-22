import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Route, Routes } from 'react-router-dom';
import { render as rtlRender } from '@testing-library/react';
import { waitFor } from '@testing-library/react';
import { MemoryRouterProvider, screen, userEvent, within } from '../../test/test-utils';

vi.mock('react-plotly.js', () => ({ default: () => null }));
vi.mock('plotly.js-dist-min', () => ({
  default: { newPlot: vi.fn(), react: vi.fn(), purge: vi.fn() },
}));

vi.mock('@/hooks', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/hooks')>();
  return {
    ...actual,
    useSchedules: vi.fn(),
    useDeleteSchedule: vi.fn(),
    useUpdateSchedule: vi.fn(),
  };
});

vi.mock('@/features/schedules', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/features/schedules')>();
  return {
    ...actual,
    UploadScheduleCard: ({
      onUploadComplete,
    }: {
      onUploadComplete?: (result: { schedule_id: number; schedule_name?: string }) => void;
    }) => (
      <button
        type="button"
        onClick={() => onUploadComplete?.({ schedule_id: 4, schedule_name: 'Schedule Delta' })}
      >
        Mock Upload
      </button>
    ),
    useScheduleAnalysisData: vi.fn(),
    downloadScheduleJson: vi.fn().mockResolvedValue(undefined),
  };
});

import * as hooks from '@/hooks';
import * as schedulesFeature from '@/features/schedules';
import Advanced from '../Advanced';

function renderAdvanced(path = '/advanced') {
  return rtlRender(
    <MemoryRouterProvider initialEntries={[path]}>
      <Routes>
        <Route path="/advanced" element={<Advanced />} />
      </Routes>
    </MemoryRouterProvider>
  );
}

const schedulesResponse = {
  schedules: [
    { schedule_id: 1, schedule_name: 'Schedule Alpha' },
    { schedule_id: 2, schedule_name: 'Schedule Beta' },
    { schedule_id: 3, schedule_name: 'Schedule Gamma' },
    { schedule_id: 4, schedule_name: 'Schedule Delta' },
  ],
  total: 4,
};

function makeVisibleAnalysis(ids: number[]) {
  return ids.map((id) => ({
    id,
    name:
      schedulesResponse.schedules.find((schedule) => schedule.schedule_id === id)?.schedule_name ??
      `#${id}`,
    insights: {
      blocks: [
        {
          scheduling_block_id: id * 10,
          original_block_id: `OB-${id}`,
          block_name: `Block ${id}`,
          priority: 8,
          total_visibility_hours: 4,
          requested_hours: 2,
          elevation_range_deg: 45,
          scheduled: true,
          scheduled_start_mjd: 60000 + id,
          scheduled_stop_mjd: 60000 + id + 0.1,
        },
      ],
      metrics: {
        total_observations: 1,
        scheduled_count: 1,
        unscheduled_count: 0,
        scheduling_rate: 1,
        mean_priority: 8,
        median_priority: 8,
        mean_priority_scheduled: 8,
        mean_priority_unscheduled: 0,
        total_visibility_hours: 4,
        mean_requested_hours: 2,
      },
      correlations: [],
      top_priority: [],
      top_visibility: [],
      conflicts: [],
      total_count: 1,
      scheduled_count: 1,
      impossible_count: 0,
    },
    fragmentation: {
      schedule_id: id,
      schedule_name:
        schedulesResponse.schedules.find((schedule) => schedule.schedule_id === id)
          ?.schedule_name ?? `#${id}`,
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
    isLoading: false,
    error: null,
  }));
}

describe('Advanced page', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    vi.mocked(hooks.useSchedules).mockReturnValue({
      data: schedulesResponse,
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof hooks.useSchedules>);

    vi.mocked(hooks.useDeleteSchedule).mockReturnValue({
      mutateAsync: vi.fn().mockResolvedValue({ message: 'deleted' }),
      isPending: false,
    } as unknown as ReturnType<typeof hooks.useDeleteSchedule>);

    vi.mocked(hooks.useUpdateSchedule).mockReturnValue({
      mutateAsync: vi.fn().mockResolvedValue({}),
      isPending: false,
    } as unknown as ReturnType<typeof hooks.useUpdateSchedule>);

    vi.mocked(schedulesFeature.useScheduleAnalysisData).mockImplementation((ids: number[]) =>
      makeVisibleAnalysis(ids)
    );
  });

  it('renders the /advanced workspace route and normalizes invalid params', async () => {
    renderAdvanced('/advanced?workspace=2,2,1,999&visible=999,2&baseline=999');

    expect(screen.getByText('Advanced Workspace')).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByDisplayValue('Schedule Beta')).toBeInTheDocument();
    });

    expect(screen.getByText(/missing schedule removed from the workspace/i)).toBeInTheDocument();
  });

  it('adds schedules from the database list and keeps the visible subset in sync', async () => {
    const user = userEvent.setup();

    renderAdvanced('/advanced?workspace=1&visible=1&baseline=1');

    await user.click(screen.getAllByRole('button', { name: 'Add' })[1]);

    await waitFor(() => {
      expect(screen.getByDisplayValue('Schedule Alpha')).toBeInTheDocument();
    });

    expect(screen.getByText('Schedule Beta')).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByText(/summary metrics/i)).toBeInTheDocument();
      expect(screen.getByText(/block status/i)).toBeInTheDocument();
    });
  });

  it('reassigns the baseline when hiding or deleting the current baseline', async () => {
    const user = userEvent.setup();

    renderAdvanced('/advanced?workspace=1,2&visible=1,2&baseline=1');

    const workspacePanel = screen.getByText('Workspace (2)').closest('section');
    expect(workspacePanel).toBeTruthy();

    const workspaceButtons = workspacePanel?.querySelectorAll('button');
    expect(workspaceButtons?.length).toBeGreaterThan(0);

    await user.click(screen.getAllByRole('button', { name: 'Hide' })[0]);

    await waitFor(() => {
      expect(screen.getByDisplayValue('Schedule Beta')).toBeInTheDocument();
    });

    expect(screen.getByText(/Schedule Beta is loaded as the baseline/i)).toBeInTheDocument();

    await user.click(screen.getAllByRole('button', { name: 'Delete' })[0]);
    const dialog = screen.getByText('Delete Schedule').closest('div');
    expect(dialog).toBeTruthy();
    await user.click(within(dialog as HTMLElement).getByRole('button', { name: /^Delete$/ }));

    await waitFor(() => {
      expect(screen.getByText(/Schedule Beta is loaded as the baseline/i)).toBeInTheDocument();
    });
  });

  it('adds uploaded schedules into the workspace', async () => {
    const user = userEvent.setup();

    renderAdvanced('/advanced?workspace=1&visible=1&baseline=1');

    await user.click(screen.getByRole('button', { name: 'Mock Upload' }));

    await waitFor(() => {
      expect(screen.getByText('Schedule Delta')).toBeInTheDocument();
    });
  });
});
