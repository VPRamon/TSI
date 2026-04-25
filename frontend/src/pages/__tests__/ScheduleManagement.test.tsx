import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Route, Routes } from 'react-router-dom';
import { render as rtlRender } from '@testing-library/react';
import { MemoryRouterProvider, screen, waitFor, userEvent } from '../../test/test-utils';

vi.mock('react-plotly.js', () => ({ default: () => null }));
vi.mock('plotly.js-dist-min', () => ({
  default: { newPlot: vi.fn(), react: vi.fn(), purge: vi.fn() },
}));

vi.mock('@/hooks', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/hooks')>();
  return {
    ...actual,
    useSchedules: vi.fn(),
    useDeleteSchedules: vi.fn(),
    useUpdateSchedule: vi.fn(),
  };
});

import * as hooks from '@/hooks';
import ScheduleManagement from '../ScheduleManagement';

type MutationResult<TArgs extends unknown[] = [number[]]> = {
  mutateAsync: (...args: TArgs) => Promise<unknown>;
  isPending: boolean;
};

function renderScheduleManagement() {
  return rtlRender(
    <MemoryRouterProvider initialEntries={['/schedules/manage']}>
      <Routes>
        <Route path="/schedules/manage" element={<ScheduleManagement />} />
      </Routes>
    </MemoryRouterProvider>
  );
}

const schedulesResponse = {
  schedules: [
    { schedule_id: 1, schedule_name: 'Schedule Alpha' },
    { schedule_id: 2, schedule_name: 'Schedule Beta' },
    { schedule_id: 3, schedule_name: 'Schedule Gamma' },
  ],
  total: 3,
};

describe('ScheduleManagement', () => {
  const deleteMutation: MutationResult = {
    mutateAsync: vi.fn().mockResolvedValue({ deleted_count: 1, message: 'deleted' }),
    isPending: false,
  };

  const updateMutation: MutationResult<[{
    scheduleId: number;
    request: { name?: string };
  }]> = {
    mutateAsync: vi.fn().mockResolvedValue({}),
    isPending: false,
  };

  beforeEach(() => {
    vi.clearAllMocks();

    vi.mocked(hooks.useSchedules).mockReturnValue({
      data: schedulesResponse,
      isLoading: false,
      error: null,
      refetch: vi.fn(),
    } as unknown as ReturnType<typeof hooks.useSchedules>);
    vi.mocked(hooks.useDeleteSchedules).mockReturnValue(
      deleteMutation as unknown as ReturnType<typeof hooks.useDeleteSchedules>
    );
    vi.mocked(hooks.useUpdateSchedule).mockReturnValue(
      updateMutation as unknown as ReturnType<typeof hooks.useUpdateSchedule>
    );
  });

  it('enables bulk delete for one selected schedule and opens the confirmation dialog', async () => {
    const user = userEvent.setup();
    renderScheduleManagement();

    await user.click(screen.getByLabelText('Select Schedule Alpha'));
    expect(screen.getByText('1 selected')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /delete selected/i }));

    expect(screen.getByText('Delete Schedule')).toBeInTheDocument();
    expect(screen.getAllByText('Schedule Alpha')).toHaveLength(2);
    expect(screen.getByRole('button', { name: /delete 1 schedule/i })).toBeInTheDocument();
  });

  it('updates multi-select dialog copy for multiple schedules', async () => {
    const user = userEvent.setup();
    renderScheduleManagement();

    await user.click(screen.getByLabelText('Select Schedule Alpha'));
    await user.click(screen.getByLabelText('Select Schedule Beta'));
    await user.click(screen.getByRole('button', { name: /delete selected/i }));

    expect(screen.getByText('Delete Schedules')).toBeInTheDocument();
    expect(screen.getByText('Are you sure you want to delete these 2 schedules?')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /delete 2 schedules/i })).toBeInTheDocument();
  });

  it('select all checkbox toggles the whole list', async () => {
    const user = userEvent.setup();
    renderScheduleManagement();

    const selectAll = screen.getByLabelText('Select all schedules');
    await user.click(selectAll);
    expect(screen.getByText('3 selected')).toBeInTheDocument();

    await user.click(selectAll);
    expect(screen.queryByText('3 selected')).not.toBeInTheDocument();
  });

  it('deletes selected schedules in one batch call and clears selection after success', async () => {
    const user = userEvent.setup();
    const mutateAsync = vi.fn().mockResolvedValue({ deleted_count: 2, message: 'deleted' });
    vi.mocked(hooks.useDeleteSchedules).mockReturnValue({
      mutateAsync,
      isPending: false,
    } as unknown as ReturnType<typeof hooks.useDeleteSchedules>);

    renderScheduleManagement();

    await user.click(screen.getByLabelText('Select Schedule Alpha'));
    await user.click(screen.getByLabelText('Select Schedule Beta'));
    await user.click(screen.getByRole('button', { name: /delete selected/i }));
    await user.click(screen.getByRole('button', { name: /delete 2 schedules/i }));

    await waitFor(() => {
      expect(mutateAsync).toHaveBeenCalledTimes(1);
      expect(mutateAsync).toHaveBeenCalledWith([1, 2]);
    });

    expect(screen.getByRole('alert')).toHaveTextContent('Deleted 2 schedules successfully.');
    expect(screen.queryByText('2 selected')).not.toBeInTheDocument();
    expect(screen.getByLabelText('Select Schedule Alpha')).not.toBeChecked();
    expect(screen.getByLabelText('Select Schedule Beta')).not.toBeChecked();
  });

  it('shows an error on partial failure and keeps remaining schedules selected', async () => {
    const user = userEvent.setup();
    const mutateAsync = vi.fn().mockRejectedValue(new Error('Server exploded'));
    vi.mocked(hooks.useDeleteSchedules).mockReturnValue({
      mutateAsync,
      isPending: false,
    } as unknown as ReturnType<typeof hooks.useDeleteSchedules>);

    renderScheduleManagement();

    await user.click(screen.getByLabelText('Select Schedule Alpha'));
    await user.click(screen.getByLabelText('Select Schedule Beta'));
    await user.click(screen.getByRole('button', { name: /delete selected/i }));
    await user.click(screen.getByRole('button', { name: /delete 2 schedules/i }));

    await waitFor(() => {
      expect(mutateAsync).toHaveBeenCalledTimes(1);
      expect(mutateAsync).toHaveBeenCalledWith([1, 2]);
    });

    expect(screen.getByRole('alert')).toHaveTextContent('Server exploded');
    expect(screen.getByText('2 selected')).toBeInTheDocument();
    expect(screen.getByLabelText('Select Schedule Alpha')).toBeChecked();
    expect(screen.getByLabelText('Select Schedule Beta')).toBeChecked();
  });

  it('keeps edit actions independent from selection state', async () => {
    const user = userEvent.setup();
    renderScheduleManagement();

    await user.click(screen.getByLabelText('Select Schedule Alpha'));
    await user.click(screen.getByLabelText('Edit Schedule Beta'));

    expect(screen.getByText('Edit Schedule')).toBeInTheDocument();
    expect(screen.getByDisplayValue('Schedule Beta')).toBeInTheDocument();
    expect(screen.getByText('1 selected')).toBeInTheDocument();
  });
});
