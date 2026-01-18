/**
 * Tests for schedule domain models and adapters.
 */
import { describe, it, expect } from 'vitest';
import {
  toScheduleViewModel,
  toSkyMapViewModel,
  toDistributionViewModel,
} from './schedule';
import type { ScheduleInfo, SkyMapData, DistributionData } from '@/api';

describe('toScheduleViewModel', () => {
  it('converts schedule info to view model', () => {
    const schedule: ScheduleInfo = {
      schedule_id: 1,
      schedule_name: 'Test Schedule',
    };

    const result = toScheduleViewModel(schedule);

    expect(result.id).toBe(1);
    expect(result.name).toBe('Test Schedule');
    expect(result.displayName).toBe('Test Schedule');
  });

  it('truncates long schedule names', () => {
    const schedule: ScheduleInfo = {
      schedule_id: 2,
      schedule_name: 'This is a very long schedule name that exceeds the limit',
    };

    const result = toScheduleViewModel(schedule);

    expect(result.displayName).toBe('This is a very long schedule name tha...');
    expect(result.displayName.length).toBe(40);
  });

  it('keeps short names unchanged', () => {
    const schedule: ScheduleInfo = {
      schedule_id: 3,
      schedule_name: 'Short name',
    };

    const result = toScheduleViewModel(schedule);

    expect(result.displayName).toBe('Short name');
  });
});

describe('toSkyMapViewModel', () => {
  const mockSkyMapData: SkyMapData = {
    blocks: [
      {
        original_block_id: 'B001',
        priority: 8.5,
        priority_bin: 'high',
        requested_duration_seconds: 3600,
        target_ra_deg: 180,
        target_dec_deg: 45,
        scheduled_period: { start: 60000, stop: 60001 },
      },
      {
        original_block_id: 'B002',
        priority: 3.0,
        priority_bin: 'low',
        requested_duration_seconds: 1800,
        target_ra_deg: 90,
        target_dec_deg: -30,
        scheduled_period: null,
      },
    ],
    priority_bins: [
      { label: 'High', min_priority: 7, max_priority: 10, color: '#ff0000' },
      { label: 'Low', min_priority: 0, max_priority: 4, color: '#00ff00' },
    ],
    priority_min: 0,
    priority_max: 10,
    ra_min: 0,
    ra_max: 360,
    dec_min: -90,
    dec_max: 90,
    total_count: 2,
    scheduled_count: 1,
    scheduled_time_min: 60000,
    scheduled_time_max: 60001,
  };

  it('separates scheduled and unscheduled blocks', () => {
    const result = toSkyMapViewModel(mockSkyMapData);

    expect(result.scheduledBlocks).toHaveLength(1);
    expect(result.unscheduledBlocks).toHaveLength(1);
    expect(result.scheduledBlocks[0].id).toBe('B001');
    expect(result.unscheduledBlocks[0].id).toBe('B002');
  });

  it('calculates scheduling rate correctly', () => {
    const result = toSkyMapViewModel(mockSkyMapData);

    expect(result.schedulingRate).toBe(50);
  });

  it('handles empty blocks', () => {
    const emptyData: SkyMapData = {
      ...mockSkyMapData,
      blocks: [],
      total_count: 0,
      scheduled_count: 0,
    };

    const result = toSkyMapViewModel(emptyData);

    expect(result.schedulingRate).toBe(0);
    expect(result.scheduledBlocks).toHaveLength(0);
    expect(result.unscheduledBlocks).toHaveLength(0);
  });

  it('maps priority bins correctly', () => {
    const result = toSkyMapViewModel(mockSkyMapData);

    expect(result.priorityBins).toHaveLength(2);
    expect(result.priorityBins[0].label).toBe('High');
    expect(result.priorityBins[0].range).toBe('7.0 - 10.0');
    expect(result.priorityBins[0].color).toBe('#ff0000');
  });

  it('includes priority range', () => {
    const result = toSkyMapViewModel(mockSkyMapData);

    expect(result.priorityRange.min).toBe(0);
    expect(result.priorityRange.max).toBe(10);
  });
});

describe('toDistributionViewModel', () => {
  const mockDistributionData: DistributionData = {
    blocks: [
      { priority: 8.0, total_visibility_hours: 4.5, requested_hours: 2, elevation_range_deg: 45, scheduled: true },
      { priority: 3.0, total_visibility_hours: 2.0, requested_hours: 1, elevation_range_deg: 30, scheduled: false },
      { priority: 6.0, total_visibility_hours: 3.5, requested_hours: 1.5, elevation_range_deg: 35, scheduled: true },
    ],
    priority_stats: { count: 3, mean: 5.67, median: 6.0, std_dev: 2.05, min: 3.0, max: 8.0, sum: 17.0 },
    visibility_stats: { count: 3, mean: 3.33, median: 3.5, std_dev: 1.04, min: 2.0, max: 4.5, sum: 10.0 },
    requested_hours_stats: { count: 3, mean: 1.5, median: 1.5, std_dev: 0.41, min: 1.0, max: 2.0, sum: 4.5 },
    total_count: 3,
    scheduled_count: 2,
    unscheduled_count: 1,
    impossible_count: 0,
  };

  it('extracts scheduled and unscheduled priorities', () => {
    const result = toDistributionViewModel(mockDistributionData);

    expect(result.scheduledPriorities).toEqual([8.0, 6.0]);
    expect(result.unscheduledPriorities).toEqual([3.0]);
  });

  it('formats priority stats correctly', () => {
    const result = toDistributionViewModel(mockDistributionData);

    expect(result.priorityStats.mean).toBe('5.67');
    expect(result.priorityStats.median).toBe('6.00');
    expect(result.priorityStats.stdDev).toBe('2.05');
    expect(result.priorityStats.range).toBe('3.0 - 8.0');
  });

  it('formats visibility stats with units', () => {
    const result = toDistributionViewModel(mockDistributionData);

    expect(result.visibilityStats.mean).toBe('3.3h');
    expect(result.visibilityStats.median).toBe('3.5h');
  });

  it('includes count summaries', () => {
    const result = toDistributionViewModel(mockDistributionData);

    expect(result.totalCount).toBe(3);
    expect(result.scheduledCount).toBe(2);
    expect(result.unscheduledCount).toBe(1);
    expect(result.impossibleCount).toBe(0);
  });
});
