/**
 * SkyMapFilters - Settings panel for filtering SkyMap visualization.
 * Allows filtering by scheduled status, scheduled period range, and priority.
 *
 * Checkboxes apply immediately. Numeric/datetime inputs are debounced
 * to prevent expensive re-filtering of the scatter plot on every keystroke.
 */
import { memo, useState, useRef, useCallback, useEffect } from 'react';
import type { PriorityBinInfo } from '@/api/types';

export interface SkyMapFilterState {
  showScheduled: boolean;
  showUnscheduled: boolean;
  scheduledBeginUtc: string; // ISO 8601 datetime string
  scheduledEndUtc: string; // ISO 8601 datetime string
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
  /** Priority bins for the color legend */
  bins?: PriorityBinInfo[];
  /** Callback to reset filters to defaults */
  onReset: () => void;
}

const DEBOUNCE_MS = 250;
const PRIORITY_SLIDER_CLASS =
  'pointer-events-none absolute inset-x-0 top-1/2 h-6 w-full -translate-y-1/2 appearance-none bg-transparent focus:outline-none [&::-moz-range-thumb]:pointer-events-auto [&::-moz-range-thumb]:h-4 [&::-moz-range-thumb]:w-4 [&::-moz-range-thumb]:cursor-pointer [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-2 [&::-moz-range-thumb]:border-slate-950 [&::-moz-range-thumb]:bg-primary-400 [&::-moz-range-track]:h-2 [&::-moz-range-track]:rounded-full [&::-moz-range-track]:border-0 [&::-moz-range-track]:bg-transparent [&::-webkit-slider-runnable-track]:h-2 [&::-webkit-slider-runnable-track]:rounded-full [&::-webkit-slider-runnable-track]:bg-transparent [&::-webkit-slider-thumb]:pointer-events-auto [&::-webkit-slider-thumb]:mt-[-4px] [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:border-2 [&::-webkit-slider-thumb]:border-slate-950 [&::-webkit-slider-thumb]:bg-primary-400';

const SkyMapFilters = memo(function SkyMapFilters({
  filters,
  onChange,
  scheduledTimeRange,
  priorityRange,
  bins,
  onReset,
}: SkyMapFiltersProps) {
  // Local state for debounced fields
  const [localPriorityMin, setLocalPriorityMin] = useState(filters.priorityMin);
  const [localPriorityMax, setLocalPriorityMax] = useState(filters.priorityMax);
  const [localBeginUtc, setLocalBeginUtc] = useState(filters.scheduledBeginUtc);
  const [localEndUtc, setLocalEndUtc] = useState(filters.scheduledEndUtc);

  // Sync local state when parent resets filters
  const prevFiltersRef = useRef(filters);
  useEffect(() => {
    const prev = prevFiltersRef.current;
    if (prev.priorityMin !== filters.priorityMin) setLocalPriorityMin(filters.priorityMin);
    if (prev.priorityMax !== filters.priorityMax) setLocalPriorityMax(filters.priorityMax);
    if (prev.scheduledBeginUtc !== filters.scheduledBeginUtc)
      setLocalBeginUtc(filters.scheduledBeginUtc);
    if (prev.scheduledEndUtc !== filters.scheduledEndUtc) setLocalEndUtc(filters.scheduledEndUtc);
    prevFiltersRef.current = filters;
  }, [filters]);

  // Debounce helper
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(
    () => () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    },
    []
  );

  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const filtersRef = useRef(filters);
  filtersRef.current = filters;

  const debouncedApply = useCallback((patch: Partial<SkyMapFilterState>) => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      onChangeRef.current({ ...filtersRef.current, ...patch });
    }, DEBOUNCE_MS);
  }, []);

  // Checkboxes apply immediately (cheap toggle)
  const handleCheckboxChange = (field: 'showScheduled' | 'showUnscheduled') => {
    onChange({ ...filters, [field]: !filters[field] });
  };

  const handlePriorityChange = (field: 'priorityMin' | 'priorityMax', value: number) => {
    const roundedValue = Math.round(value * 10) / 10;
    if (field === 'priorityMin') {
      const nextMin = Math.min(roundedValue, localPriorityMax);
      setLocalPriorityMin(nextMin);
      debouncedApply({ priorityMin: nextMin });
      return;
    }

    const nextMax = Math.max(roundedValue, localPriorityMin);
    setLocalPriorityMax(nextMax);
    debouncedApply({ priorityMax: nextMax });
  };

  const handleScheduledTimeChange = (
    field: 'scheduledBeginUtc' | 'scheduledEndUtc',
    value: string
  ) => {
    if (field === 'scheduledBeginUtc') setLocalBeginUtc(value);
    else setLocalEndUtc(value);
    debouncedApply({ [field]: value });
  };

  const hasScheduledTimeRange = scheduledTimeRange.min !== null && scheduledTimeRange.max !== null;
  const prioritySpan = Math.max(priorityRange.max - priorityRange.min, 0.1);
  const priorityMinPercent = ((localPriorityMin - priorityRange.min) / prioritySpan) * 100;
  const priorityMaxPercent = ((localPriorityMax - priorityRange.min) / prioritySpan) * 100;

  return (
    <div className="rounded-lg border border-slate-700 bg-slate-800/50 px-5 py-4">
      {/* Header row with title and reset */}
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-xs font-semibold uppercase tracking-wide text-slate-400">Filters</h3>
        <button
          onClick={onReset}
          className="rounded px-2 py-0.5 text-xs text-slate-500 transition-colors hover:bg-slate-700/50 hover:text-slate-300"
        >
          Reset
        </button>
      </div>

      {/* Filter sections in a responsive grid with dividers */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-[auto_1px_auto_1px_1fr] sm:items-start sm:gap-0">
        {/* Schedule Status */}
        <div className="flex flex-col gap-1.5 sm:pr-5">
          <p className="text-[11px] font-medium uppercase tracking-wide text-slate-400">
            Schedule Status
          </p>
          <div className="inline-flex rounded-lg border border-slate-700 bg-slate-900/40 p-1">
            <label
              className={`flex cursor-pointer items-center gap-2 rounded-md px-3 py-1.5 text-sm transition-colors ${
                filters.showScheduled
                  ? 'bg-emerald-500/15 text-emerald-200'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              <input
                type="checkbox"
                checked={filters.showScheduled}
                onChange={() => handleCheckboxChange('showScheduled')}
                className="sr-only"
              />
              <span className="inline-block h-2.5 w-2.5 rounded-full bg-green-500" />
              Scheduled
            </label>
            <label
              className={`flex cursor-pointer items-center gap-2 rounded-md px-3 py-1.5 text-sm transition-colors ${
                filters.showUnscheduled
                  ? 'bg-rose-500/15 text-rose-200'
                  : 'text-slate-400 hover:text-slate-200'
              }`}
            >
              <input
                type="checkbox"
                checked={filters.showUnscheduled}
                onChange={() => handleCheckboxChange('showUnscheduled')}
                className="sr-only"
              />
              <span className="inline-block h-2.5 w-2.5 rounded-full bg-red-500" />
              Unscheduled
            </label>
          </div>
        </div>

        {/* Divider 1 */}
        <div className="hidden self-stretch bg-slate-700 sm:block" />

        {/* Scheduled Period */}
        {hasScheduledTimeRange ? (
          <div className="flex flex-col gap-1.5 sm:px-5">
            <p className="text-[11px] font-medium uppercase tracking-wide text-slate-400">
              Scheduled Period (UTC)
            </p>
            <div className="flex items-center gap-3">
              <input
                type="datetime-local"
                value={localBeginUtc}
                onChange={(e) => handleScheduledTimeChange('scheduledBeginUtc', e.target.value)}
                className="rounded border border-slate-600 bg-slate-700 px-2 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
              />
              <span className="text-xs text-slate-600">–</span>
              <input
                type="datetime-local"
                value={localEndUtc}
                onChange={(e) => handleScheduledTimeChange('scheduledEndUtc', e.target.value)}
                className="rounded border border-slate-600 bg-slate-700 px-2 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
              />
            </div>
          </div>
        ) : (
          <div className="sm:px-5" />
        )}

        {/* Divider 2 */}
        <div className="hidden self-stretch bg-slate-700 sm:block" />

        {/* Priority Range */}
        <div className="flex items-start gap-4 sm:pl-5">
          <div className="flex min-w-0 flex-1 flex-col gap-1.5">
            <div className="flex items-center justify-between">
              <p className="text-[11px] font-medium uppercase tracking-wide text-slate-400">
                Priority Range
              </p>
              <span className="text-[11px] tabular-nums text-slate-500">
                {localPriorityMin.toFixed(1)} – {localPriorityMax.toFixed(1)}
              </span>
            </div>
            <div className="w-full">
              <div className="relative h-6">
                <div className="absolute inset-x-0 top-1/2 h-2 -translate-y-1/2 rounded-full bg-slate-700" />
                <div
                  className="absolute top-1/2 h-2 -translate-y-1/2 rounded-full bg-primary-500"
                  style={{
                    left: `${priorityMinPercent}%`,
                    width: `${Math.max(priorityMaxPercent - priorityMinPercent, 0)}%`,
                  }}
                />
                <input
                  aria-label="Minimum priority"
                  type="range"
                  step="0.1"
                  min={priorityRange.min}
                  max={priorityRange.max}
                  value={localPriorityMin}
                  onChange={(e) => handlePriorityChange('priorityMin', Number(e.target.value))}
                  className={`${PRIORITY_SLIDER_CLASS} z-10`}
                />
                <input
                  aria-label="Maximum priority"
                  type="range"
                  step="0.1"
                  min={priorityRange.min}
                  max={priorityRange.max}
                  value={localPriorityMax}
                  onChange={(e) => handlePriorityChange('priorityMax', Number(e.target.value))}
                  className={`${PRIORITY_SLIDER_CLASS} z-20`}
                />
              </div>
              <div className="mt-0.5 flex items-center justify-between text-[11px] text-slate-600">
                <span>{priorityRange.min.toFixed(1)}</span>
                <span>{priorityRange.max.toFixed(1)}</span>
              </div>
            </div>
          </div>
          {bins && bins.length > 0 && (
            <div className="flex flex-shrink-0 flex-col gap-1 self-center">
              {bins.map((bin) => (
                <div key={bin.label} className="flex items-center gap-1.5">
                  <div
                    className="h-2 w-2 flex-shrink-0 rounded-sm"
                    style={{ backgroundColor: bin.color }}
                    aria-hidden="true"
                  />
                  <span className="whitespace-nowrap text-[11px] text-slate-400">{bin.label}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
});

export default SkyMapFilters;
