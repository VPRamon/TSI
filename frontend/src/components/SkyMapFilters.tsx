/**
 * SkyMapFilters - Settings panel for filtering SkyMap visualization.
 * Allows filtering by scheduled status, scheduled period range, and priority.
 */
import { memo } from 'react';

export interface SkyMapFilterState {
  showScheduled: boolean;
  showUnscheduled: boolean;
  scheduledBeginUtc: string; // ISO 8601 datetime string
  scheduledEndUtc: string;   // ISO 8601 datetime string
  priorityMin: number;
  priorityMax: number;
}

interface SkyMapFiltersProps {
  filters: SkyMapFilterState;
  onChange: (filters: SkyMapFilterState) => void;
  /** Min/max scheduled time from the data in UTC (for display) */
  scheduledTimeRange: { min: string | null; max: string | null };
  /** Min/max priority from the data (for slider bounds) */
  priorityRange: { min: number; max: number };
  /** Callback to reset filters to defaults */
  onReset: () => void;
}

const SkyMapFilters = memo(function SkyMapFilters({
  filters,
  onChange,
  scheduledTimeRange,
  priorityRange,
  onReset,
}: SkyMapFiltersProps) {
  const handleCheckboxChange = (field: 'showScheduled' | 'showUnscheduled') => {
    onChange({ ...filters, [field]: !filters[field] });
  };

  const handlePriorityChange = (field: 'priorityMin' | 'priorityMax', value: number) => {
    onChange({ ...filters, [field]: value });
  };

  const handleScheduledTimeChange = (field: 'scheduledBeginUtc' | 'scheduledEndUtc', value: string) => {
    onChange({ ...filters, [field]: value });
  };

  const hasScheduledTimeRange =
    scheduledTimeRange.min !== null && scheduledTimeRange.max !== null;

  // Format time range for display
  const formatTimeRange = () => {
    if (!scheduledTimeRange.min || !scheduledTimeRange.max) return '';
    const minDate = new Date(scheduledTimeRange.min).toLocaleString('en-US', { 
      timeZone: 'UTC', 
      dateStyle: 'short', 
      timeStyle: 'short' 
    });
    const maxDate = new Date(scheduledTimeRange.max).toLocaleString('en-US', { 
      timeZone: 'UTC', 
      dateStyle: 'short', 
      timeStyle: 'short' 
    });
    return `Range: ${minDate} – ${maxDate} UTC`;
  };

  return (
    <div className="rounded-lg border border-slate-700 bg-slate-800/50 p-4">
      <div className="mb-4 flex items-center justify-between">
        <h3 className="text-sm font-semibold text-slate-200">Filters</h3>
        <button
          onClick={onReset}
          className="rounded px-2 py-1 text-xs text-slate-400 transition-colors hover:bg-slate-700 hover:text-slate-200"
        >
          Reset
        </button>
      </div>

      {/* Status filters */}
      <div className="mb-5">
        <label className="mb-2 block text-xs font-medium uppercase tracking-wide text-slate-400">
          Status
        </label>
        <div className="space-y-2">
          <label className="flex cursor-pointer items-center gap-2">
            <input
              type="checkbox"
              checked={filters.showScheduled}
              onChange={() => handleCheckboxChange('showScheduled')}
              className="h-4 w-4 rounded border-slate-600 bg-slate-700 text-green-500 focus:ring-green-500 focus:ring-offset-slate-800"
            />
            <span className="flex items-center gap-2 text-sm text-slate-300">
              <span className="inline-block h-2.5 w-2.5 rounded-full bg-green-500" />
              Scheduled
            </span>
          </label>
          <label className="flex cursor-pointer items-center gap-2">
            <input
              type="checkbox"
              checked={filters.showUnscheduled}
              onChange={() => handleCheckboxChange('showUnscheduled')}
              className="h-4 w-4 rounded border-slate-600 bg-slate-700 text-red-500 focus:ring-red-500 focus:ring-offset-slate-800"
            />
            <span className="flex items-center gap-2 text-sm text-slate-300">
              <span className="inline-block h-2.5 w-2.5 rounded-full bg-red-500" />
              Unscheduled
            </span>
          </label>
        </div>
      </div>

      {/* Scheduled period range (only show if there's scheduled data) */}
      {hasScheduledTimeRange && (
        <div className="mb-5">
          <label className="mb-2 block text-xs font-medium uppercase tracking-wide text-slate-400">
            Scheduled Period (UTC)
          </label>
          <div className="space-y-2">
            <div>
              <label className="mb-1 block text-xs text-slate-500">Begin</label>
              <input
                type="datetime-local"
                value={filters.scheduledBeginUtc}
                onChange={(e) => handleScheduledTimeChange('scheduledBeginUtc', e.target.value)}
                className="w-full rounded border border-slate-600 bg-slate-700 px-2 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
              />
            </div>
            <div>
              <label className="mb-1 block text-xs text-slate-500">End</label>
              <input
                type="datetime-local"
                value={filters.scheduledEndUtc}
                onChange={(e) => handleScheduledTimeChange('scheduledEndUtc', e.target.value)}
                className="w-full rounded border border-slate-600 bg-slate-700 px-2 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
              />
            </div>
          </div>
          <p className="mt-2 text-xs text-slate-500">{formatTimeRange()}</p>
        </div>
      )}

      {/* Priority range */}
      <div className="mb-2">
        <label className="mb-2 block text-xs font-medium uppercase tracking-wide text-slate-400">
          Priority Range
        </label>
        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="mb-1 block text-xs text-slate-500">Min</label>
            <input
              type="number"
              step="0.1"
              min={priorityRange.min}
              max={priorityRange.max}
              value={filters.priorityMin}
              onChange={(e) => handlePriorityChange('priorityMin', parseFloat(e.target.value) || priorityRange.min)}
              className="w-full rounded border border-slate-600 bg-slate-700 px-2 py-1.5 text-sm text-slate-200 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            />
          </div>
          <div>
            <label className="mb-1 block text-xs text-slate-500">Max</label>
            <input
              type="number"
              step="0.1"
              min={priorityRange.min}
              max={priorityRange.max}
              value={filters.priorityMax}
              onChange={(e) => handlePriorityChange('priorityMax', parseFloat(e.target.value) || priorityRange.max)}
              className="w-full rounded border border-slate-600 bg-slate-700 px-2 py-1.5 text-sm text-slate-200 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            />
          </div>
        </div>
        <p className="mt-1 text-xs text-slate-500">
          Data range: {priorityRange.min.toFixed(1)} – {priorityRange.max.toFixed(1)}
        </p>
      </div>
    </div>
  );
});

export default SkyMapFilters;
