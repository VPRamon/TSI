import { describe, expect, it, vi } from 'vitest';
import { render, screen, userEvent } from '../../../test/test-utils';
import type { ScheduleInfo } from '@/api/types';
import ScheduleListCard from './ScheduleListCard';

function makeSchedule(
  schedule_id: number,
  schedule_name: string,
  algorithm?: string
): ScheduleInfo {
  return {
    schedule_id,
    schedule_name,
    observer_location: {
      lon_deg: -17.89,
      lat_deg: 28.76,
      height: 2396,
    },
    schedule_period: {
      start_mjd: 61710,
      end_mjd: 62076,
    },
    schedule_metadata: algorithm ? { algorithm } : undefined,
  };
}

const schedules = [
  makeSchedule(101, 'CTA-N EST baseline', 'est'),
  makeSchedule(202, 'CTA-N HAP population 8', 'hap'),
  makeSchedule(303, 'CTA-S nightly import'),
];

function renderCard() {
  return render(
    <ScheduleListCard
      schedules={schedules}
      total={schedules.length}
      onScheduleClick={vi.fn()}
      onScheduleDownload={vi.fn()}
      onManageSchedules={vi.fn()}
    />
  );
}

describe('ScheduleListCard search', () => {
  it('filters database schedules by name and clears the query', async () => {
    const user = userEvent.setup();
    renderCard();

    const search = screen.getByLabelText('Search database schedules');
    await user.type(search, 'population');

    expect(screen.getByText('CTA-N HAP population 8')).toBeInTheDocument();
    expect(screen.queryByText('CTA-N EST baseline')).not.toBeInTheDocument();
    expect(screen.getByText('1 of 3 matching')).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /clear schedule search/i }));

    expect(screen.getByText('CTA-N EST baseline')).toBeInTheDocument();
    expect(screen.getByText('CTA-N HAP population 8')).toBeInTheDocument();
    expect(screen.getByText('3 available')).toBeInTheDocument();
  });

  it('matches schedule metadata and shows an empty search state', async () => {
    const user = userEvent.setup();
    renderCard();

    const search = screen.getByLabelText('Search database schedules');
    await user.type(search, 'hap');

    expect(screen.getByText('CTA-N HAP population 8')).toBeInTheDocument();
    expect(screen.queryByText('CTA-N EST baseline')).not.toBeInTheDocument();

    await user.clear(search);
    await user.type(search, 'does-not-exist');

    expect(screen.getByText('No schedules match "does-not-exist"')).toBeInTheDocument();
    expect(screen.getByText('0 of 3 matching')).toBeInTheDocument();
  });
});
