/**
 * Generic configuration filter hook.
 *
 * Builds RangeFilterSpecs from numeric dimensions discovered in a batch
 * of `algorithm_config` blobs and tracks user-selected categorical
 * inclusion sets. Returns a `predicate` callers can apply to drop items
 * outside the active filter set, plus the filtered subset.
 *
 * Decoupled from the EST-specific RunRow so the same hook drives both
 * `ScheduleComparisonCharts` (operating on `ScheduleAnalysisData`) and
 * the EST extension panels (operating on `RunRow`).
 */
import { useMemo, useState } from 'react';
import type { RangeFilterSpec, RangeFilterValue } from '@/components';
import { initialRangeValues } from '@/components';
import { extractDimensions, readDimension, type Dimension } from './dimensions';

export interface CategoricalSelection {
  /** Set of currently-included categorical values (stringified). */
  included: Set<string>;
}

export interface ConfigFilterState<T> {
  numericSpecs: RangeFilterSpec[];
  categorical: Dimension[];
  numericValues: Record<string, RangeFilterValue>;
  setNumericValues: (next: Record<string, RangeFilterValue>) => void;
  categoricalValues: Record<string, CategoricalSelection>;
  setCategoricalValue: (key: string, included: Set<string>) => void;
  /** True iff at least one user-controlled filter is active. */
  hasAnyFilter: boolean;
  /** Returns true when the item satisfies every active filter. */
  predicate: (item: T) => boolean;
  /** Subset of `items` that pass the predicate. */
  filtered: T[];
  /** Reset all filters back to their defaults. */
  reset: () => void;
}

export interface UseConfigFiltersOptions<T> {
  items: T[];
  /** Extracts the algorithm config blob for the given item. */
  getConfig: (item: T) => Record<string, unknown> | undefined | null;
}

export function useConfigFilters<T>({
  items,
  getConfig,
}: UseConfigFiltersOptions<T>): ConfigFilterState<T> {
  const dims = useMemo(
    () => extractDimensions(items.map((it) => getConfig(it))),
    // getConfig is expected to be stable; depend on items only to avoid resetting
    // filters on unrelated parent re-renders.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [items],
  );

  const numericSpecs = useMemo<RangeFilterSpec[]>(
    () =>
      dims.numeric.map((d) => ({
        key: d.key,
        label: d.key,
        values: d.values.filter((v): v is number => typeof v === 'number'),
      })),
    [dims],
  );

  const initialNumeric = useMemo(() => initialRangeValues(numericSpecs), [numericSpecs]);
  const [numericValues, setNumericValues] =
    useState<Record<string, RangeFilterValue>>(initialNumeric);

  const initialCategorical = useMemo<Record<string, CategoricalSelection>>(() => {
    const out: Record<string, CategoricalSelection> = {};
    for (const dim of dims.categorical) {
      out[dim.key] = { included: new Set(dim.values.map(String)) };
    }
    return out;
  }, [dims]);
  const [categoricalValues, setCategoricalValues] =
    useState<Record<string, CategoricalSelection>>(initialCategorical);

  // Re-seed when the dimension set changes (different items / config keys).
  const seedSignature = useMemo(
    () =>
      [
        ...numericSpecs.map((s) => `n:${s.key}:${s.values.length}`),
        ...dims.categorical.map((d) => `c:${d.key}:${d.values.length}`),
      ].join('|'),
    [numericSpecs, dims.categorical],
  );
  const [lastSeed, setLastSeed] = useState(seedSignature);
  if (seedSignature !== lastSeed) {
    setLastSeed(seedSignature);
    setNumericValues(initialNumeric);
    setCategoricalValues(initialCategorical);
  }

  const setCategoricalValue = (key: string, included: Set<string>) => {
    setCategoricalValues((prev) => ({ ...prev, [key]: { included } }));
  };

  const reset = () => {
    setNumericValues(initialNumeric);
    setCategoricalValues(initialCategorical);
  };

  const hasAnyFilter = useMemo(() => {
    for (const spec of numericSpecs) {
      const v = numericValues[spec.key];
      if (!v) continue;
      const all = spec.values;
      if (all.length === 0) continue;
      const lo = Math.min(...all);
      const hi = Math.max(...all);
      if (v.min > lo || v.max < hi) return true;
    }
    for (const dim of dims.categorical) {
      const sel = categoricalValues[dim.key];
      if (!sel) continue;
      if (sel.included.size !== dim.values.length) return true;
    }
    return false;
  }, [numericSpecs, numericValues, dims.categorical, categoricalValues]);

  const predicate = (item: T): boolean => {
    const cfg = getConfig(item);
    for (const dim of dims.numeric) {
      const range = numericValues[dim.key];
      if (!range) continue;
      const v = readDimension(cfg, dim);
      if (typeof v !== 'number') continue;
      if (v < range.min || v > range.max) return false;
    }
    for (const dim of dims.categorical) {
      const sel = categoricalValues[dim.key];
      if (!sel) continue;
      const v = readDimension(cfg, dim);
      if (v == null) continue;
      if (!sel.included.has(String(v))) return false;
    }
    return true;
  };

  const filtered = useMemo(
    () => items.filter(predicate),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [items, numericValues, categoricalValues, dims],
  );

  return {
    numericSpecs,
    categorical: dims.categorical,
    numericValues,
    setNumericValues,
    categoricalValues,
    setCategoricalValue,
    hasAnyFilter,
    predicate,
    filtered,
    reset,
  };
}
