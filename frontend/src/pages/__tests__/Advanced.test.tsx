import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Route, Routes } from 'react-router-dom';
import { render as rtlRender, waitFor } from '@testing-library/react';
import { MemoryRouterProvider, screen, userEvent, within } from '../../test/test-utils';

vi.mock('react-plotly.js', () => ({ default: () => null }));
vi.mock('plotly.js-dist-min', () => ({
  default: { newPlot: vi.fn(), react: vi.fn(), purge: vi.fn() },
}));

vi.mock('@/hooks', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/hooks')>();
  return {
    ...actual,
    useEnvironments: vi.fn(),
    useSchedules: vi.fn(),
    useCreateEnvironment: vi.fn(),
    useDeleteEnvironment: vi.fn(),
    useBulkImportToEnvironment: vi.fn(),
    useRemoveScheduleFromEnvironment: vi.fn(),
  };
});

import * as hooks from '@/hooks';
import Advanced from '../Advanced';
import type { EnvironmentInfo } from '@/api/types';

function renderAdvanced() {
  return rtlRender(
    <MemoryRouterProvider initialEntries={['/advanced']}>
      <Routes>
        <Route path="/advanced" element={<Advanced />} />
      </Routes>
    </MemoryRouterProvider>
  );
}

const baseEnvironments: EnvironmentInfo[] = [
  {
    environment_id: 1,
    name: 'CTAO South March',
    structure: {
      period_start_mjd: 60000,
      period_end_mjd: 60030,
      lat_deg: -24.6,
      lon_deg: -70.4,
      elevation_m: 2400,
      blocks_hash: 'abc123',
    },
    schedule_ids: [101, 102],
    created_at: '2025-03-01T00:00:00Z',
  },
  {
    environment_id: 2,
    name: 'Empty draft',
    structure: null,
    schedule_ids: [],
    created_at: '2025-02-01T00:00:00Z',
  },
];

describe('Advanced page (environments)', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    vi.mocked(hooks.useEnvironments).mockReturnValue({
      data: { environments: baseEnvironments, total: baseEnvironments.length },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof hooks.useEnvironments>);

    vi.mocked(hooks.useSchedules).mockReturnValue({
      data: {
        schedules: [
          { schedule_id: 101, schedule_name: 'Plan A' },
          { schedule_id: 102, schedule_name: 'Plan B' },
        ],
        total: 2,
      },
      isLoading: false,
      error: null,
    } as unknown as ReturnType<typeof hooks.useSchedules>);

    vi.mocked(hooks.useDeleteEnvironment).mockReturnValue({
      mutateAsync: vi.fn(),
      isPending: false,
    } as unknown as ReturnType<typeof hooks.useDeleteEnvironment>);

    vi.mocked(hooks.useRemoveScheduleFromEnvironment).mockReturnValue({
      mutate: vi.fn(),
      isPending: false,
    } as unknown as ReturnType<typeof hooks.useRemoveScheduleFromEnvironment>);

    vi.mocked(hooks.useBulkImportToEnvironment).mockReturnValue({
      mutateAsync: vi.fn(),
      isPending: false,
    } as unknown as ReturnType<typeof hooks.useBulkImportToEnvironment>);

    vi.mocked(hooks.useCreateEnvironment).mockReturnValue({
      mutateAsync: vi.fn(),
      isPending: false,
    } as unknown as ReturnType<typeof hooks.useCreateEnvironment>);
  });

  it('renders the environment cards', () => {
    renderAdvanced();
    expect(screen.getByRole('heading', { name: /environments/i })).toBeInTheDocument();
    expect(screen.getByText('CTAO South March')).toBeInTheDocument();
    expect(screen.getByText('Empty draft')).toBeInTheDocument();
    expect(screen.getByText(/2 schedules/i)).toBeInTheDocument();
    expect(screen.getByText(/no structure/i)).toBeInTheDocument();
  });

  it('creates an environment then bulk-imports the dropped files', async () => {
    const user = userEvent.setup();
    const createMutate = vi
      .fn()
      .mockResolvedValue({
        environment_id: 99,
        name: 'New env',
        structure: null,
        schedule_ids: [],
        created_at: '2025-03-15T00:00:00Z',
      });
    const bulkMutate = vi.fn().mockResolvedValue({
      created: [{ schedule_id: 500, name: 'good' }],
      rejected: [
        {
          name: 'mismatch',
          reason: 'period mismatch',
          mismatch_fields: ['period_start_mjd', 'blocks_hash'],
        },
      ],
    });

    vi.mocked(hooks.useCreateEnvironment).mockReturnValue({
      mutateAsync: createMutate,
      isPending: false,
    } as unknown as ReturnType<typeof hooks.useCreateEnvironment>);

    vi.mocked(hooks.useBulkImportToEnvironment).mockReturnValue({
      mutateAsync: bulkMutate,
      isPending: false,
    } as unknown as ReturnType<typeof hooks.useBulkImportToEnvironment>);

    renderAdvanced();

    await user.click(screen.getByRole('button', { name: /create environment/i }));

    const dialog = await screen.findByRole('dialog', { name: /create environment/i });
    await user.type(within(dialog).getByLabelText(/environment name/i), 'New env');

    const goodFile = new File(['{"foo":1}'], 'good.json', { type: 'application/json' });
    const badFile = new File(['{"foo":2}'], 'mismatch.json', { type: 'application/json' });
    const fileInput = dialog.querySelector('input[type="file"]') as HTMLInputElement;
    expect(fileInput).toBeTruthy();
    await user.upload(fileInput, [goodFile, badFile]);

    await waitFor(() => {
      expect(within(dialog).getByText('good.json')).toBeInTheDocument();
      expect(within(dialog).getByText('mismatch.json')).toBeInTheDocument();
    });

    await user.click(within(dialog).getByRole('button', { name: /^create$/i }));

    await waitFor(() => {
      expect(createMutate).toHaveBeenCalledWith({ name: 'New env' });
    });
    expect(bulkMutate).toHaveBeenCalledWith({
      environmentId: 99,
      request: {
        items: [
          { name: 'good', schedule_json: { foo: 1 } },
          { name: 'mismatch', schedule_json: { foo: 2 } },
        ],
      },
    });

    const result = await within(dialog).findByTestId('bulk-import-result');
    expect(within(result).getByText(/1 schedule accepted/i)).toBeInTheDocument();
    const rejected = within(result).getByTestId('rejected-list');
    expect(within(rejected).getByText('mismatch')).toBeInTheDocument();
    expect(within(rejected).getByText(/period mismatch/i)).toBeInTheDocument();
    expect(within(rejected).getByText(/period_start_mjd, blocks_hash/i)).toBeInTheDocument();
  });
});
