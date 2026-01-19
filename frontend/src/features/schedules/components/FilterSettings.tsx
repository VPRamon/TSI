/**
 * FilterSettings - Controls for histogram filtering.
 * 
 * ARCHITECTURE:
 * - This component OWNS its filter state completely (no sync from parent)
 * - Parent only provides initial values via ref-based initialization
 * - Changes are debounced before calling onParamsChange
 * - No feedback loop: parent state changes don't trigger local state sync
 * 
 * ROOT CAUSE FIX:
 * The previous implementation had a useEffect that synced from `initialParams`,
 * which caused re-renders when the parent's appliedFilters changed. This created
 * a feedback loop and unnecessary re-renders.
 * 
 * Now: FilterSettings is fully uncontrolled after initial mount. The parent
 * cannot push state changes back to this component (except via remount, which
 * we avoid). Reset is handled internally.
 */
import { useState, useCallback, useRef, memo, useEffect } from 'react';
import { useRemountDetector, useRenderCounter } from '@/hooks/useRemountDetector';

export interface FilterParams {
  numBins: number;
  binDurationMinutes: number | undefined;
  priorityMin: number | undefined;
  priorityMax: number | undefined;
  useCustomDuration: boolean;
}

interface FilterSettingsProps {
  /** Default values (only used on mount, not synced) */
  defaultParams: FilterParams;
  /** Priority range from map data */
  mapPriorityMin: number;
  mapPriorityMax: number;
  /** Callback when filters change (debounced internally) */
  onParamsChange: (params: FilterParams) => void;
}

/** Debounce delay in milliseconds */
const DEBOUNCE_MS = 150;

/**
 * FilterSettings component with fully internal state.
 * No sync from parent after mount - prevents feedback loops and remounts.
 */
const FilterSettings = memo(function FilterSettings({
  defaultParams,
  mapPriorityMin,
  mapPriorityMax,
  onParamsChange,
}: FilterSettingsProps) {
  // DEV: Remount/render detection
  useRemountDetector('FilterSettings');
  useRenderCounter('FilterSettings');

  // Use ref to capture initial values without creating dependencies
  const defaultRef = useRef(defaultParams);

  // Internal state - fully owned by this component
  const [numBins, setNumBins] = useState(() => defaultRef.current.numBins);
  const [binDurationMinutes, setBinDurationMinutes] = useState(() => defaultRef.current.binDurationMinutes);
  const [priorityMin, setPriorityMin] = useState(() => defaultRef.current.priorityMin);
  const [priorityMax, setPriorityMax] = useState(() => defaultRef.current.priorityMax);
  const [useCustomDuration, setUseCustomDuration] = useState(() => defaultRef.current.useCustomDuration);

  // Ref for debounce timer
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  
  // Stable ref for onParamsChange to avoid stale closures
  const onParamsChangeRef = useRef(onParamsChange);
  onParamsChangeRef.current = onParamsChange;

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  // Debounced apply to parent
  const scheduleApply = useCallback((params: FilterParams) => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    debounceRef.current = setTimeout(() => {
      onParamsChangeRef.current(params);
    }, DEBOUNCE_MS);
  }, []);

  // Create current params object
  const getCurrentParams = useCallback(
    (overrides: Partial<FilterParams> = {}): FilterParams => ({
      numBins,
      binDurationMinutes,
      priorityMin,
      priorityMax,
      useCustomDuration,
      ...overrides,
    }),
    [numBins, binDurationMinutes, priorityMin, priorityMax, useCustomDuration]
  );

  // Handler for number of bins
  const handleNumBinsChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = parseInt(e.target.value, 10);
      setNumBins(value);
      scheduleApply(getCurrentParams({ numBins: value }));
    },
    [scheduleApply, getCurrentParams]
  );

  // Handler for bin duration
  const handleBinDurationChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value ? parseInt(e.target.value, 10) : undefined;
      setBinDurationMinutes(value);
      scheduleApply(getCurrentParams({ binDurationMinutes: value }));
    },
    [scheduleApply, getCurrentParams]
  );

  // Handler for priority min
  const handlePriorityMinChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value ? parseFloat(e.target.value) : undefined;
      setPriorityMin(value);
      scheduleApply(getCurrentParams({ priorityMin: value }));
    },
    [scheduleApply, getCurrentParams]
  );

  // Handler for priority max
  const handlePriorityMaxChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value ? parseFloat(e.target.value) : undefined;
      setPriorityMax(value);
      scheduleApply(getCurrentParams({ priorityMax: value }));
    },
    [scheduleApply, getCurrentParams]
  );

  // Handler for binning method toggle
  const handleUseCustomDurationChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      const value = e.target.value === 'duration';
      setUseCustomDuration(value);
      scheduleApply(getCurrentParams({ useCustomDuration: value }));
    },
    [scheduleApply, getCurrentParams]
  );

  // Reset handler
  const handleReset = useCallback(() => {
    // Clear any pending debounce
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    // Reset to defaults
    const defaults = defaultRef.current;
    setNumBins(defaults.numBins);
    setBinDurationMinutes(defaults.binDurationMinutes);
    setPriorityMin(defaults.priorityMin);
    setPriorityMax(defaults.priorityMax);
    setUseCustomDuration(defaults.useCustomDuration);
    // Immediately apply defaults
    onParamsChangeRef.current(defaults);
  }, []);

  return (
    <div className="space-y-5">
      <h3 className="text-sm font-medium text-slate-200">Filters</h3>

      {/* Binning method toggle */}
      <div>
        <label className="mb-1.5 block text-xs font-medium text-slate-400">
          Binning Method
        </label>
        <select
          className="w-full rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          value={useCustomDuration ? 'duration' : 'bins'}
          onChange={handleUseCustomDurationChange}
        >
          <option value="bins">Number of Bins</option>
          <option value="duration">Bin Duration</option>
        </select>
      </div>

      {/* Number of bins OR bin duration */}
      {!useCustomDuration ? (
        <div>
          <label className="mb-1.5 block text-xs font-medium text-slate-400">
            Number of Bins
          </label>
          <div className="flex items-center gap-3">
            <input
              type="range"
              min="10"
              max="200"
              value={numBins}
              onChange={handleNumBinsChange}
              className="h-2 flex-1 cursor-pointer appearance-none rounded-lg bg-slate-600"
            />
            <span className="w-10 text-right text-sm font-medium text-white">{numBins}</span>
          </div>
        </div>
      ) : (
        <div>
          <label className="mb-1.5 block text-xs font-medium text-slate-400">
            Bin Duration (min)
          </label>
          <input
            type="number"
            min="1"
            max="10080"
            value={binDurationMinutes ?? 60}
            onChange={handleBinDurationChange}
            className="w-full rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
        </div>
      )}

      <div className="border-t border-slate-700 pt-4">
        <h4 className="mb-3 text-xs font-medium uppercase tracking-wide text-slate-500">
          Priority Range
        </h4>

        {/* Priority min */}
        <div className="mb-3">
          <label className="mb-1.5 block text-xs font-medium text-slate-400">
            Min Priority
          </label>
          <input
            type="number"
            min={mapPriorityMin}
            max={mapPriorityMax}
            step="0.1"
            value={priorityMin ?? mapPriorityMin}
            onChange={handlePriorityMinChange}
            className="w-full rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
        </div>

        {/* Priority max */}
        <div>
          <label className="mb-1.5 block text-xs font-medium text-slate-400">
            Max Priority
          </label>
          <input
            type="number"
            min={mapPriorityMin}
            max={mapPriorityMax}
            step="0.1"
            value={priorityMax ?? mapPriorityMax}
            onChange={handlePriorityMaxChange}
            className="w-full rounded-md border border-slate-600 bg-slate-700 px-3 py-2 text-sm text-white focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          />
        </div>
      </div>

      {/* Reset button */}
      <button
        onClick={handleReset}
        className="w-full rounded-md bg-slate-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-slate-500 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-800"
      >
        Reset Filters
      </button>
    </div>
  );
});

export default FilterSettings;
