/**
 * RangeFilterGroup — generic dual-thumb numeric range sliders, used to
 * filter chart datasets by configuration values (e.g. EST e/k/b,
 * priority bins).
 *
 * Each spec describes one parameter; the component:
 *   - Auto-derives min/max bounds from `spec.values` (sorted, unique).
 *   - Hides itself when the parameter has fewer than two distinct values
 *     (no point exposing a one-position slider).
 *   - Emits `{ [key]: [min, max] }` updates on every change, debounced
 *     to a single React commit per interaction (no extra dep).
 */
import { useCallback, useEffect, useMemo, useRef, useState, type ChangeEvent } from 'react';

/** Debounce window (ms) applied to slider drags before `onChange` fires. */
const DEBOUNCE_MS = 120;

export interface RangeFilterSpec {
  /** Stable key used in the `values` map. */
  key: string;
  /** Human-readable label rendered above the slider. */
  label: string;
  /** All distinct numeric values that exist in the dataset. */
  values: number[];
  /** Optional unit suffix appended to the displayed bounds. */
  unit?: string;
  /** Optional override step. Defaults to the smallest gap between values. */
  step?: number;
}

export interface RangeFilterValue {
  min: number;
  max: number;
}

export interface RangeFilterGroupProps {
  specs: RangeFilterSpec[];
  values: Record<string, RangeFilterValue>;
  onChange: (next: Record<string, RangeFilterValue>) => void;
  /** Optional caption rendered above the group. */
  label?: string;
}

interface ResolvedSpec {
  spec: RangeFilterSpec;
  min: number;
  max: number;
  step: number;
}

function resolveSpec(spec: RangeFilterSpec): ResolvedSpec | null {
  const distinct = Array.from(new Set(spec.values.filter((v) => Number.isFinite(v)))).sort(
    (a, b) => a - b,
  );
  if (distinct.length < 2) return null;
  const min = distinct[0];
  const max = distinct[distinct.length - 1];
  let step = spec.step;
  if (step == null) {
    let smallest = Infinity;
    for (let i = 1; i < distinct.length; i++) {
      const diff = distinct[i] - distinct[i - 1];
      if (diff > 0 && diff < smallest) smallest = diff;
    }
    step = Number.isFinite(smallest) ? smallest : 1;
  }
  return { spec, min, max, step };
}

function format(value: number, unit?: string): string {
  const rendered = Number.isInteger(value) ? value.toString() : value.toFixed(2);
  return unit ? `${rendered} ${unit}` : rendered;
}

export function RangeFilterGroup({ specs, values, onChange, label }: RangeFilterGroupProps) {
  const resolved = useMemo(
    () => specs.map(resolveSpec).filter((r): r is ResolvedSpec => r !== null),
    [specs],
  );

  // Local mirror so rapid drags update the UI immediately, while the
  // upstream `onChange` is throttled to one trailing call per DEBOUNCE_MS
  // window (no extra deps; behaves like lodash.debounce trailing-only).
  const [local, setLocal] = useState<Record<string, RangeFilterValue>>(values);
  const dirtyRef = useRef(false);

  useEffect(() => {
    if (!dirtyRef.current) setLocal(values);
  }, [values]);

  const onChangeRef = useRef(onChange);
  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);

  useEffect(() => {
    if (!dirtyRef.current) return;
    const handle = setTimeout(() => {
      dirtyRef.current = false;
      onChangeRef.current(local);
    }, DEBOUNCE_MS);
    return () => clearTimeout(handle);
  }, [local]);

  const update = useCallback((key: string, next: RangeFilterValue) => {
    dirtyRef.current = true;
    setLocal((prev) => ({ ...prev, [key]: next }));
  }, []);

  if (resolved.length === 0) return null;

  return (
    <div className="flex flex-col gap-2 rounded-lg border border-slate-700 bg-slate-800/40 p-3">
      {label && (
        <span className="text-xs font-medium uppercase tracking-wider text-slate-400">{label}</span>
      )}
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {resolved.map(({ spec, min, max, step }) => {
          const current = local[spec.key] ?? values[spec.key] ?? { min, max };
          const lo = Math.max(min, Math.min(current.min, current.max));
          const hi = Math.min(max, Math.max(current.min, current.max));

          const onLow = (e: ChangeEvent<HTMLInputElement>) => {
            const v = Math.min(Number(e.target.value), hi);
            update(spec.key, { min: v, max: hi });
          };
          const onHigh = (e: ChangeEvent<HTMLInputElement>) => {
            const v = Math.max(Number(e.target.value), lo);
            update(spec.key, { min: lo, max: v });
          };

          return (
            <div key={spec.key} className="flex flex-col gap-1">
              <div className="flex items-center justify-between text-xs text-slate-300">
                <span className="font-medium">{spec.label}</span>
                <span className="font-mono text-slate-400">
                  {format(lo, spec.unit)} – {format(hi, spec.unit)}
                </span>
              </div>
              <div className="grid grid-cols-2 gap-2">
                <input
                  type="range"
                  min={min}
                  max={max}
                  step={step}
                  value={lo}
                  onChange={onLow}
                  aria-label={`${spec.label} minimum`}
                  className="accent-primary-500"
                />
                <input
                  type="range"
                  min={min}
                  max={max}
                  step={step}
                  value={hi}
                  onChange={onHigh}
                  aria-label={`${spec.label} maximum`}
                  className="accent-primary-500"
                />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

/**
 * Build the initial `values` map (full range) for a list of specs.
 * Specs that resolve to a single value are omitted from the result.
 */
export function initialRangeValues(specs: RangeFilterSpec[]): Record<string, RangeFilterValue> {
  const out: Record<string, RangeFilterValue> = {};
  for (const spec of specs) {
    const r = resolveSpec(spec);
    if (r) out[spec.key] = { min: r.min, max: r.max };
  }
  return out;
}

export default RangeFilterGroup;
