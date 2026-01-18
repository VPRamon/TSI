/**
 * Schedule domain models and adapters.
 */
import type { ScheduleInfo, SkyMapData, DistributionData, ScheduleTimelineData } from '@/api';

/**
 * View model for schedule list items.
 */
export interface ScheduleViewModel {
  id: number;
  name: string;
  displayName: string;
}

/**
 * View model for sky map visualization.
 */
export interface SkyMapViewModel {
  scheduledBlocks: SkyMapBlockViewModel[];
  unscheduledBlocks: SkyMapBlockViewModel[];
  totalCount: number;
  scheduledCount: number;
  schedulingRate: number;
  priorityRange: { min: number; max: number };
  priorityBins: Array<{
    label: string;
    range: string;
    color: string;
  }>;
}

export interface SkyMapBlockViewModel {
  id: string;
  ra: number;
  dec: number;
  priority: number;
  hoverText: string;
}

/**
 * View model for distribution statistics.
 */
export interface DistributionViewModel {
  totalCount: number;
  scheduledCount: number;
  unscheduledCount: number;
  impossibleCount: number;
  priorityStats: StatsViewModel;
  visibilityStats: StatsViewModel;
  scheduledPriorities: number[];
  unscheduledPriorities: number[];
  scheduledVisibilities: number[];
  unscheduledVisibilities: number[];
}

export interface StatsViewModel {
  mean: string;
  median: string;
  stdDev: string;
  range: string;
}

/**
 * View model for timeline visualization.
 */
export interface TimelineViewModel {
  totalCount: number;
  scheduledCount: number;
  uniqueMonths: string[];
  darkPeriodCount: number;
  blocks: TimelineBlockViewModel[];
  darkPeriods: Array<{ start: Date; end: Date }>;
}

export interface TimelineBlockViewModel {
  id: string;
  index: number;
  startDate: Date;
  endDate: Date;
  priority: number;
  color: string;
  hoverText: string;
}

// =============================================================================
// Adapters - Transform API responses to view models
// =============================================================================

/**
 * Convert API ScheduleInfo to ScheduleViewModel.
 */
export function toScheduleViewModel(schedule: ScheduleInfo): ScheduleViewModel {
  return {
    id: schedule.schedule_id,
    name: schedule.schedule_name,
    displayName: schedule.schedule_name.length > 40
      ? `${schedule.schedule_name.substring(0, 37)}...`
      : schedule.schedule_name,
  };
}

/**
 * Convert API SkyMapData to SkyMapViewModel.
 */
export function toSkyMapViewModel(data: SkyMapData): SkyMapViewModel {
  const scheduled = data.blocks.filter((b) => b.scheduled_period !== null);
  const unscheduled = data.blocks.filter((b) => b.scheduled_period === null);

  return {
    scheduledBlocks: scheduled.map((b) => ({
      id: b.original_block_id,
      ra: b.target_ra_deg,
      dec: b.target_dec_deg,
      priority: b.priority,
      hoverText: `ID: ${b.original_block_id}<br>Priority: ${b.priority.toFixed(1)}`,
    })),
    unscheduledBlocks: unscheduled.map((b) => ({
      id: b.original_block_id,
      ra: b.target_ra_deg,
      dec: b.target_dec_deg,
      priority: b.priority,
      hoverText: `ID: ${b.original_block_id}<br>Priority: ${b.priority.toFixed(1)}`,
    })),
    totalCount: data.total_count,
    scheduledCount: data.scheduled_count,
    schedulingRate:
      data.total_count > 0 ? (data.scheduled_count / data.total_count) * 100 : 0,
    priorityRange: { min: data.priority_min, max: data.priority_max },
    priorityBins: data.priority_bins.map((bin) => ({
      label: bin.label,
      range: `${bin.min_priority.toFixed(1)} - ${bin.max_priority.toFixed(1)}`,
      color: bin.color,
    })),
  };
}

/**
 * Convert API DistributionData to DistributionViewModel.
 */
export function toDistributionViewModel(data: DistributionData): DistributionViewModel {
  const scheduled = data.blocks.filter((b) => b.scheduled);
  const unscheduled = data.blocks.filter((b) => !b.scheduled);

  return {
    totalCount: data.total_count,
    scheduledCount: data.scheduled_count,
    unscheduledCount: data.unscheduled_count,
    impossibleCount: data.impossible_count,
    priorityStats: {
      mean: data.priority_stats.mean.toFixed(2),
      median: data.priority_stats.median.toFixed(2),
      stdDev: data.priority_stats.std_dev.toFixed(2),
      range: `${data.priority_stats.min.toFixed(1)} - ${data.priority_stats.max.toFixed(1)}`,
    },
    visibilityStats: {
      mean: `${data.visibility_stats.mean.toFixed(1)}h`,
      median: `${data.visibility_stats.median.toFixed(1)}h`,
      stdDev: `${data.visibility_stats.std_dev.toFixed(1)}h`,
      range: `${data.visibility_stats.min.toFixed(0)} - ${data.visibility_stats.max.toFixed(0)}h`,
    },
    scheduledPriorities: scheduled.map((b) => b.priority),
    unscheduledPriorities: unscheduled.map((b) => b.priority),
    scheduledVisibilities: scheduled.map((b) => b.total_visibility_hours),
    unscheduledVisibilities: unscheduled.map((b) => b.total_visibility_hours),
  };
}

/**
 * Convert API ScheduleTimelineData to TimelineViewModel.
 */
export function toTimelineViewModel(
  data: ScheduleTimelineData,
  mjdToDate: (mjd: number) => Date
): TimelineViewModel {
  return {
    totalCount: data.total_count,
    scheduledCount: data.scheduled_count,
    uniqueMonths: data.unique_months,
    darkPeriodCount: data.dark_periods.length,
    blocks: data.blocks.map((block, index) => ({
      id: block.original_block_id,
      index,
      startDate: mjdToDate(block.scheduled_start_mjd),
      endDate: mjdToDate(block.scheduled_stop_mjd),
      priority: block.priority,
      color: `hsl(${(block.priority / 10) * 240}, 70%, 50%)`,
      hoverText: `${block.original_block_id}<br>Priority: ${block.priority.toFixed(1)}<br>Duration: ${block.requested_hours.toFixed(1)}h`,
    })),
    darkPeriods: data.dark_periods.map((period) => ({
      start: mjdToDate(period.start),
      end: mjdToDate(period.stop),
    })),
  };
}
