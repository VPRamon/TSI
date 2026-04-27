/**
 * Verdict + KPI delta table for the EnvironmentCompare page (A2).
 *
 * Top-of-page block that:
 *   1. Auto-picks a baseline (oldest schedule, i.e. lowest `schedule_id`)
 *      with a manual override.
 *   2. Ranks all schedules by composite score and surfaces the winner.
 *   3. Shows per-KPI absolute and relative deltas vs the baseline,
 *      colour-coded by improvement / regression.
 *
 * Pure presentation — fed by `useEnvironmentKpis` data passed from the
 * parent. If the request errored or the env has fewer than two KPI rows
 * the component renders a quiet placeholder so the rest of the page
 * still loads.
 */
import { useMemo, useState } from 'react';
import type { ScheduleKpi } from '@/api/types';

interface EnvironmentVerdictProps {
  kpis: ScheduleKpi[];
}

interface KpiRow {
  key: keyof ScheduleKpi;
  label: string;
  unit?: string;
  /** When true a *higher* value is better; otherwise lower is better. */
  higherIsBetter: boolean;
  /** Format the raw numeric value for display. */
  format: (v: number) => string;
}

const KPI_ROWS: KpiRow[] = [
  {
    key: 'composite_score',
    label: 'Composite score (0–1)',
    higherIsBetter: true,
    format: (v) => v.toFixed(3),
  },
  {
    key: 'scheduling_rate',
    label: 'Scheduling rate',
    higherIsBetter: true,
    format: (v) => `${(v * 100).toFixed(1)}%`,
  },
  {
    key: 'scheduled_fraction_of_operable',
    label: 'Operable time used',
    higherIsBetter: true,
    format: (v) => `${(v * 100).toFixed(1)}%`,
  },
  {
    key: 'fit_visibility_fraction_of_operable',
    label: 'Fit visibility / operable',
    higherIsBetter: true,
    format: (v) => `${(v * 100).toFixed(1)}%`,
  },
  {
    key: 'scheduled_hours',
    label: 'Scheduled time',
    unit: 'h',
    higherIsBetter: true,
    format: (v) => v.toFixed(2),
  },
  {
    key: 'idle_operable_hours',
    label: 'Idle operable time',
    unit: 'h',
    higherIsBetter: false,
    format: (v) => v.toFixed(2),
  },
  {
    key: 'gap_p90_hours',
    label: 'p90 gap',
    unit: 'h',
    higherIsBetter: false,
    format: (v) => v.toFixed(2),
  },
  {
    key: 'largest_gap_hours',
    label: 'Largest gap',
    unit: 'h',
    higherIsBetter: false,
    format: (v) => v.toFixed(2),
  },
  {
    key: 'mean_priority_scheduled',
    label: 'Mean priority (scheduled)',
    higherIsBetter: true,
    format: (v) => v.toFixed(2),
  },
];

function pickDefaultBaselineId(kpis: ScheduleKpi[]): number | null {
  if (kpis.length === 0) return null;
  // "Oldest" = smallest schedule_id (auto-increment in postgres,
  // monotonic sequence in local repo). Cheap, deterministic, doesn't
  // require a `created_at` round-trip.
  return [...kpis].sort((a, b) => a.schedule_id - b.schedule_id)[0].schedule_id;
}

function deltaClass(improved: boolean, equal: boolean): string {
  if (equal) return 'text-slate-400';
  return improved ? 'text-emerald-400' : 'text-rose-400';
}

function formatRelative(delta: number): string {
  if (!Number.isFinite(delta)) return '—';
  const sign = delta > 0 ? '+' : '';
  return `${sign}${(delta * 100).toFixed(1)}%`;
}

export function EnvironmentVerdict({ kpis }: EnvironmentVerdictProps) {
  const defaultBaselineId = useMemo(() => pickDefaultBaselineId(kpis), [kpis]);
  const [pinnedBaselineId, setPinnedBaselineId] = useState<number | null>(null);
  const baselineId = pinnedBaselineId ?? defaultBaselineId;

  const ranked = useMemo(
    () => [...kpis].sort((a, b) => b.composite_score - a.composite_score),
    [kpis],
  );

  if (kpis.length === 0) {
    return null;
  }

  const baseline = kpis.find((k) => k.schedule_id === baselineId);
  const winner = ranked[0];

  if (!baseline || !winner) {
    return null;
  }

  const compareTargets = kpis.filter((k) => k.schedule_id !== baseline.schedule_id);

  return (
    <section className="rounded-lg border border-slate-700 bg-slate-900/40 p-4">
      <header className="mb-3 flex flex-wrap items-baseline justify-between gap-2">
        <div>
          <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-300">
            Verdict
          </h2>
          <p className="text-xs text-slate-400">
            Composite score uses equal weights across 5 normalized KPI components.
            The baseline is the oldest schedule unless you pin a different one.
          </p>
        </div>
        <label className="flex items-center gap-2 text-xs text-slate-300">
          Baseline:
          <select
            value={baselineId ?? ''}
            onChange={(e) => setPinnedBaselineId(Number.parseInt(e.target.value, 10))}
            className="rounded border border-slate-600 bg-slate-800 px-2 py-1 text-xs text-white"
          >
            {kpis.map((k) => (
              <option key={k.schedule_id} value={k.schedule_id}>
                {k.schedule_name} (#{k.schedule_id})
                {k.schedule_id === defaultBaselineId ? ' — default' : ''}
              </option>
            ))}
          </select>
          {pinnedBaselineId !== null && pinnedBaselineId !== defaultBaselineId && (
            <button
              type="button"
              onClick={() => setPinnedBaselineId(null)}
              className="rounded border border-slate-600 px-1.5 py-0.5 text-[10px] text-slate-300 hover:text-white"
            >
              reset
            </button>
          )}
        </label>
      </header>

      <div className="mb-4 rounded-md border border-emerald-700/40 bg-emerald-950/20 p-3">
        <div className="flex flex-wrap items-baseline justify-between gap-2">
          <div>
            <span className="text-xs uppercase tracking-wide text-emerald-300">Best</span>
            <span className="ml-2 text-sm font-semibold text-emerald-100">
              {winner.schedule_name}
            </span>
            <span className="ml-2 text-xs text-emerald-300/70">#{winner.schedule_id}</span>
          </div>
          <div className="text-xs text-emerald-200">
            score <span className="font-mono text-base">{winner.composite_score.toFixed(3)}</span>
            {winner.schedule_id !== baseline.schedule_id && (
              <span className="ml-2 text-emerald-400">
                {formatRelative(
                  (winner.composite_score - baseline.composite_score) /
                    Math.max(baseline.composite_score, 1e-9),
                )}{' '}
                vs baseline
              </span>
            )}
          </div>
        </div>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full text-xs" data-testid="kpi-delta-table">
          <thead>
            <tr className="border-b border-slate-700 text-left text-slate-400">
              <th className="py-2 pr-3">KPI</th>
              <th className="py-2 pr-3">
                Baseline:{' '}
                <span className="font-semibold text-slate-200">{baseline.schedule_name}</span>
              </th>
              {compareTargets.map((k) => (
                <th key={k.schedule_id} className="py-2 pr-3">
                  <span className="font-semibold text-slate-200">{k.schedule_name}</span>
                  <span className="ml-1 text-slate-500">#{k.schedule_id}</span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {KPI_ROWS.map((row) => {
              const baselineValue = baseline[row.key] as number;
              return (
                <tr
                  key={String(row.key)}
                  className="border-b border-slate-800/60 text-slate-200 last:border-b-0"
                >
                  <td className="py-1.5 pr-3 text-slate-300">
                    {row.label}
                    {row.unit ? <span className="ml-1 text-slate-500">({row.unit})</span> : null}
                  </td>
                  <td className="py-1.5 pr-3 font-mono text-slate-200">
                    {row.format(baselineValue)}
                  </td>
                  {compareTargets.map((k) => {
                    const value = k[row.key] as number;
                    const absDelta = value - baselineValue;
                    const relDelta =
                      Math.abs(baselineValue) > 1e-9 ? absDelta / Math.abs(baselineValue) : NaN;
                    const equal = Math.abs(absDelta) < 1e-9;
                    const improved = row.higherIsBetter ? absDelta > 0 : absDelta < 0;
                    return (
                      <td key={k.schedule_id} className="py-1.5 pr-3 font-mono">
                        <span className={deltaClass(improved, equal)}>{row.format(value)}</span>
                        {!equal && (
                          <span className={`ml-2 text-[10px] ${deltaClass(improved, equal)}`}>
                            ({formatRelative(relDelta)})
                          </span>
                        )}
                      </td>
                    );
                  })}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </section>
  );
}

export default EnvironmentVerdict;
