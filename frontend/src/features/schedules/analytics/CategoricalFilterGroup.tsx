/**
 * CategoricalFilterGroup — chip-style multi-select for categorical
 * configuration dimensions, mirroring the dual-thumb numeric
 * RangeFilterGroup. Each dimension renders as a row of toggleable
 * chips; clicking a chip toggles its membership in the included set.
 */
import type { CategoricalSelection } from './useConfigFilters';
import type { Dimension } from './dimensions';

export interface CategoricalFilterGroupProps {
  dimensions: Dimension[];
  values: Record<string, CategoricalSelection>;
  onChange: (key: string, included: Set<string>) => void;
  label?: string;
}

export function CategoricalFilterGroup({
  dimensions,
  values,
  onChange,
  label,
}: CategoricalFilterGroupProps) {
  if (dimensions.length === 0) return null;

  return (
    <div className="flex flex-col gap-3">
      {label ? (
        <span className="text-xs font-semibold uppercase tracking-wider text-slate-400">
          {label}
        </span>
      ) : null}
      <div className="grid gap-3 md:grid-cols-2">
        {dimensions.map((dim) => {
          const selection = values[dim.key];
          const included = selection?.included ?? new Set<string>();
          const allOn = included.size === dim.values.length;
          return (
            <div
              key={dim.key}
              className="rounded-lg border border-slate-700 bg-slate-900/40 p-3"
            >
              <div className="mb-2 flex items-center justify-between gap-2">
                <span className="text-xs font-medium uppercase tracking-wider text-slate-300">
                  {dim.key}
                </span>
                <button
                  type="button"
                  onClick={() =>
                    onChange(
                      dim.key,
                      allOn
                        ? new Set<string>()
                        : new Set<string>(dim.values.map(String)),
                    )
                  }
                  className="text-[11px] text-slate-400 hover:text-slate-200"
                >
                  {allOn ? 'Clear' : 'All'}
                </button>
              </div>
              <div className="flex flex-wrap gap-1.5">
                {dim.values.map((rawValue) => {
                  const value = String(rawValue);
                  const active = included.has(value);
                  return (
                    <button
                      key={value}
                      type="button"
                      onClick={() => {
                        const next = new Set(included);
                        if (active) next.delete(value);
                        else next.add(value);
                        onChange(dim.key, next);
                      }}
                      className={`rounded-full border px-2.5 py-0.5 text-xs transition-colors ${
                        active
                          ? 'border-sky-400 bg-sky-500/20 text-sky-100'
                          : 'border-slate-700 bg-slate-800 text-slate-400 hover:border-slate-500 hover:text-slate-200'
                      }`}
                      aria-pressed={active}
                    >
                      {value}
                    </button>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export default CategoricalFilterGroup;
