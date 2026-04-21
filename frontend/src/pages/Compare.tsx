/**
 * Compare page — multi-schedule field table.
 *
 * Route: /compare?ids=1,2,3
 *
 * Shows:
 *   1. Summary metrics table (rows = metrics, columns = schedules)
 *   2. Block status table (rows = tasks aligned by original_block_id, columns = schedules)
 */
import { useState, useMemo } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { useInsights, useFragmentation, useSchedules } from '@/hooks';
import {
  ChartPanel,
  ErrorMessage,
  LoadingSpinner,
  PageContainer,
  PageHeader,
} from '@/components';
import { SchedulePicker } from '@/features/schedules';
import { mjdToDate, isValidDate } from '@/constants/dates';
import type { InsightsData, FragmentationData, ScheduleInfo } from '@/api/types';

const PAGE_SIZE = 100;

// ─── Formatting helpers ──────────────────────────────────────────────────────

function formatMjdUtc(mjd: number | null | undefined): string {
  if (mjd == null || !Number.isFinite(mjd)) return '—';
  const d = mjdToDate(mjd);
  if (!isValidDate(d)) return '—';
  return d
    .toISOString()
    .replace('T', ' ')
    .replace(/\.\d+Z$/, ' Z');
}

function pct(v: number): string {
  return `${(v * 100).toFixed(1)} %`;
}

function fmt2(v: number | null | undefined): string {
  if (v == null || !Number.isFinite(v)) return '—';
  return v.toFixed(2);
}

function fmtH(v: number | null | undefined): string {
  if (v == null || !Number.isFinite(v)) return '—';
  return `${v.toFixed(2)} h`;
}

// ─── Per-schedule data loader ────────────────────────────────────────────────

interface ScheduleData {
  id: number;
  name: string;
  insights: InsightsData | undefined;
  fragmentation: FragmentationData | undefined;
  isLoading: boolean;
  error: Error | null;
}

function useScheduleData(ids: number[]): ScheduleData[] {
  // Hooks are called unconditionally for up to N IDs; callers ensure ≤ 10
  const i0 = useInsights(ids[0] ?? 0);
  const i1 = useInsights(ids[1] ?? 0);
  const i2 = useInsights(ids[2] ?? 0);
  const i3 = useInsights(ids[3] ?? 0);
  const i4 = useInsights(ids[4] ?? 0);
  const i5 = useInsights(ids[5] ?? 0);
  const i6 = useInsights(ids[6] ?? 0);
  const i7 = useInsights(ids[7] ?? 0);
  const i8 = useInsights(ids[8] ?? 0);
  const i9 = useInsights(ids[9] ?? 0);

  const f0 = useFragmentation(ids[0] ?? 0);
  const f1 = useFragmentation(ids[1] ?? 0);
  const f2 = useFragmentation(ids[2] ?? 0);
  const f3 = useFragmentation(ids[3] ?? 0);
  const f4 = useFragmentation(ids[4] ?? 0);
  const f5 = useFragmentation(ids[5] ?? 0);
  const f6 = useFragmentation(ids[6] ?? 0);
  const f7 = useFragmentation(ids[7] ?? 0);
  const f8 = useFragmentation(ids[8] ?? 0);
  const f9 = useFragmentation(ids[9] ?? 0);

  const insightsArr = [i0, i1, i2, i3, i4, i5, i6, i7, i8, i9];
  const fragArr = [f0, f1, f2, f3, f4, f5, f6, f7, f8, f9];

  return ids.map((id, idx) => ({
    id,
    name: fragArr[idx].data?.schedule_name ?? `Schedule #${id}`,
    insights: insightsArr[idx].data,
    fragmentation: fragArr[idx].data,
    isLoading: insightsArr[idx].isLoading || fragArr[idx].isLoading,
    error: (insightsArr[idx].error ?? fragArr[idx].error) as Error | null,
  }));
}

// ─── Summary metrics table ───────────────────────────────────────────────────

interface MetricRow {
  label: string;
  getValue: (s: ScheduleData) => number | null;
  format: (v: number | null) => string;
  bestIs: 'max' | 'min';
}

const METRIC_ROWS: MetricRow[] = [
  {
    label: 'Scheduled tasks',
    getValue: (s) => s.insights?.metrics.scheduled_count ?? null,
    format: (v) => (v == null ? '—' : v.toLocaleString()),
    bestIs: 'max',
  },
  {
    label: 'Unscheduled tasks',
    getValue: (s) => s.insights?.metrics.unscheduled_count ?? null,
    format: (v) => (v == null ? '—' : v.toLocaleString()),
    bestIs: 'min',
  },
  {
    label: 'Scheduling rate',
    getValue: (s) => s.insights?.metrics.scheduling_rate ?? null,
    format: (v) => (v == null ? '—' : pct(v)),
    bestIs: 'max',
  },
  {
    label: 'Cumulative priority',
    getValue: (s) => {
      if (!s.insights) return null;
      return s.insights.blocks
        .filter((b) => b.scheduled)
        .reduce((sum, b) => sum + b.priority, 0);
    },
    format: (v) => fmt2(v),
    bestIs: 'max',
  },
  {
    label: 'Mean priority (sched.)',
    getValue: (s) => s.insights?.metrics.mean_priority_scheduled ?? null,
    format: (v) => fmt2(v),
    bestIs: 'max',
  },
  {
    label: 'Scheduled hours',
    getValue: (s) => s.fragmentation?.metrics.scheduled_hours ?? null,
    format: (v) => fmtH(v),
    bestIs: 'max',
  },
  {
    label: 'Operable hours',
    getValue: (s) => s.fragmentation?.metrics.operable_hours ?? null,
    format: (v) => fmtH(v),
    bestIs: 'max',
  },
  {
    label: 'Gap count',
    getValue: (s) => s.fragmentation?.metrics.gap_count ?? null,
    format: (v) => (v == null ? '—' : v.toLocaleString()),
    bestIs: 'min',
  },
  {
    label: 'Gap mean',
    getValue: (s) => s.fragmentation?.metrics.gap_mean_hours ?? null,
    format: (v) => fmtH(v),
    bestIs: 'min',
  },
  {
    label: 'Gap p90',
    getValue: (s) => s.fragmentation?.metrics.gap_p90_hours ?? null,
    format: (v) => fmtH(v),
    bestIs: 'min',
  },
  {
    label: 'Largest gap',
    getValue: (s) => s.fragmentation?.metrics.largest_gap_hours ?? null,
    format: (v) => fmtH(v),
    bestIs: 'min',
  },
];

function SummaryTable({ schedules }: { schedules: ScheduleData[] }) {
  return (
    <ChartPanel title="Summary Metrics">
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-slate-700 text-slate-400">
              <th className="sticky left-0 bg-slate-800/90 px-3 py-2 text-left">Metric</th>
              {schedules.map((s) => (
                <th key={s.id} className="px-3 py-2 text-right whitespace-nowrap">
                  {s.name}
                  <span className="ml-1 text-xs text-slate-500">#{s.id}</span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {METRIC_ROWS.map((row) => {
              const values = schedules.map((s) => row.getValue(s));
              const nonNullValues = values.filter((v): v is number => v != null);
              const bestValue =
                nonNullValues.length > 0
                  ? row.bestIs === 'max'
                    ? Math.max(...nonNullValues)
                    : Math.min(...nonNullValues)
                  : null;

              return (
                <tr key={row.label} className="border-b border-slate-700/50">
                  <td className="sticky left-0 bg-slate-800/90 px-3 py-2 text-slate-300 whitespace-nowrap">
                    {row.label}
                  </td>
                  {schedules.map((s, idx) => {
                    const v = values[idx];
                    const isBest =
                      v != null &&
                      bestValue != null &&
                      nonNullValues.length > 1 &&
                      v === bestValue;
                    return (
                      <td
                        key={s.id}
                        className={`px-3 py-2 text-right tabular-nums ${
                          s.isLoading
                            ? 'text-slate-500'
                            : isBest
                              ? 'font-semibold text-emerald-400'
                              : 'text-slate-300'
                        }`}
                      >
                        {s.isLoading ? '…' : row.format(v)}
                      </td>
                    );
                  })}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </ChartPanel>
  );
}

// ─── Block status table ──────────────────────────────────────────────────────

interface BlockEntry {
  original_block_id: string;
  name: string;
  maxPriority: number;
  perSchedule: Record<
    number,
    { scheduled: boolean; start_mjd: number | null; stop_mjd: number | null }
  >;
}

function BlockStatusTable({ schedules }: { schedules: ScheduleData[] }) {
  const [page, setPage] = useState(0);

  const blockMap = useMemo(() => {
    const map = new Map<string, BlockEntry>();
    for (const s of schedules) {
      if (!s.insights) continue;
      for (const b of s.insights.blocks) {
        const key = b.original_block_id;
        if (!map.has(key)) {
          map.set(key, {
            original_block_id: key,
            name: b.block_name,
            maxPriority: b.priority,
            perSchedule: {},
          });
        }
        const entry = map.get(key)!;
        entry.maxPriority = Math.max(entry.maxPriority, b.priority);
        entry.perSchedule[s.id] = {
          scheduled: b.scheduled,
          start_mjd: b.scheduled_start_mjd,
          stop_mjd: b.scheduled_stop_mjd,
        };
      }
    }
    return map;
  }, [schedules]);

  const sortedBlocks = useMemo(
    () => [...blockMap.values()].sort((a, b) => b.maxPriority - a.maxPriority),
    [blockMap]
  );

  const totalPages = Math.ceil(sortedBlocks.length / PAGE_SIZE);
  const pageBlocks = sortedBlocks.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE);

  return (
    <ChartPanel
      title={`Block Status (${sortedBlocks.length} unique blocks)`}
      headerActions={
        totalPages > 1 ? (
          <span className="text-xs text-slate-400">
            Page {page + 1} / {totalPages}
          </span>
        ) : undefined
      }
    >
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-slate-700 text-slate-400">
              <th className="sticky left-0 bg-slate-800/90 px-3 py-2 text-left">Original ID</th>
              <th className="px-3 py-2 text-left">Name</th>
              <th className="px-3 py-2 text-right">Priority</th>
              {schedules.map((s) => (
                <th key={s.id} className="px-3 py-2 text-center whitespace-nowrap">
                  {s.name}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {pageBlocks.map((block) => (
              <tr key={block.original_block_id} className="border-b border-slate-700/50">
                <td className="sticky left-0 bg-slate-800/90 px-3 py-2 font-mono text-white">
                  {block.original_block_id}
                </td>
                <td className="px-3 py-2 text-slate-300">{block.name || '—'}</td>
                <td className="px-3 py-2 text-right text-slate-300 tabular-nums">
                  {block.maxPriority.toFixed(2)}
                </td>
                {schedules.map((s) => {
                  const entry = block.perSchedule[s.id];
                  if (!entry) {
                    return (
                      <td key={s.id} className="px-3 py-2 text-center text-slate-600">
                        —
                      </td>
                    );
                  }
                  return (
                    <td key={s.id} className="px-3 py-2 text-center">
                      {entry.scheduled ? (
                        <span className="text-emerald-400">
                          ✓{' '}
                          <span className="text-xs font-mono">
                            {formatMjdUtc(entry.start_mjd)}
                          </span>
                        </span>
                      ) : (
                        <span className="text-red-400">✗</span>
                      )}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div className="mt-4 flex items-center justify-center gap-3">
          <button
            type="button"
            onClick={() => setPage((p) => Math.max(0, p - 1))}
            disabled={page === 0}
            className="rounded-lg border border-slate-600 px-3 py-1.5 text-sm text-slate-300 hover:bg-slate-700 disabled:opacity-40"
          >
            Prev
          </button>
          <span className="text-sm text-slate-400">
            {page + 1} / {totalPages}
          </span>
          <button
            type="button"
            onClick={() => setPage((p) => Math.min(totalPages - 1, p + 1))}
            disabled={page === totalPages - 1}
            className="rounded-lg border border-slate-600 px-3 py-1.5 text-sm text-slate-300 hover:bg-slate-700 disabled:opacity-40"
          >
            Next
          </button>
        </div>
      )}
    </ChartPanel>
  );
}

// ─── Schedule chip header ────────────────────────────────────────────────────

function ScheduleChips({
  ids,
  scheduleInfoMap,
  onRemove,
  onAdd,
}: {
  ids: number[];
  scheduleInfoMap: Map<number, ScheduleInfo>;
  onRemove: (id: number) => void;
  onAdd: (schedules: ScheduleInfo[]) => void;
}) {
  const [showPicker, setShowPicker] = useState(false);

  return (
    <div className="flex flex-wrap items-center gap-2">
      {ids.map((id) => {
        const info = scheduleInfoMap.get(id);
        const label = info?.schedule_name ?? `#${id}`;
        return (
          <span
            key={id}
            className="flex items-center gap-1 rounded-full border border-slate-600 bg-slate-700 px-3 py-1 text-sm text-white"
          >
            {label}
            <button
              type="button"
              onClick={() => onRemove(id)}
              className="ml-1 text-slate-400 hover:text-red-400"
              aria-label={`Remove ${label}`}
            >
              ×
            </button>
          </span>
        );
      })}

      {/* Add schedule button */}
      <div className="relative">
        <button
          type="button"
          onClick={() => setShowPicker((v) => !v)}
          className="flex items-center gap-1 rounded-full border border-dashed border-slate-600 px-3 py-1 text-sm text-slate-400 hover:border-slate-400 hover:text-slate-300"
        >
          + Add schedule
        </button>
        {showPicker && (
          <div className="absolute left-0 top-full z-50 mt-2 w-72">
            <SchedulePicker
              multiSelect
              initialSelectedIds={ids}
              placeholder="Search schedules..."
              onConfirm={(schedules) => {
                setShowPicker(false);
                onAdd(schedules);
              }}
            />
          </div>
        )}
      </div>
    </div>
  );
}

// ─── Main page ───────────────────────────────────────────────────────────────

function ComparePage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const navigate = useNavigate();
  const [showEmptyPicker, setShowEmptyPicker] = useState(false);

  const idsParam = searchParams.get('ids') ?? '';
  const ids = useMemo(
    () =>
      idsParam
        .split(',')
        .map((s) => parseInt(s.trim(), 10))
        .filter((n) => Number.isFinite(n) && n > 0)
        .slice(0, 10),
    [idsParam]
  );

  const { data: schedulesData } = useSchedules();
  const scheduleInfoMap = useMemo(() => {
    const m = new Map<number, ScheduleInfo>();
    schedulesData?.schedules.forEach((s) => m.set(s.schedule_id, s));
    return m;
  }, [schedulesData]);

  // Clamp to 10 IDs max; hooks called unconditionally
  const paddedIds = useMemo(() => {
    const arr = [...ids];
    while (arr.length < 10) arr.push(0);
    return arr;
  }, [ids]);

  const allData = useScheduleData(paddedIds);
  const schedules = allData.slice(0, ids.length);

  const handleRemove = (id: number) => {
    const next = ids.filter((i) => i !== id);
    if (next.length === 0) {
      navigate('/compare');
    } else {
      setSearchParams({ ids: next.join(',') });
    }
  };

  const handleAdd = (selected: ScheduleInfo[]) => {
    const next = [...new Set([...ids, ...selected.map((s) => s.schedule_id)])];
    setSearchParams({ ids: next.join(',') });
  };

  const anyLoading = schedules.some((s) => s.isLoading);
  const anyError = schedules.find((s) => s.error);

  // Empty state
  if (ids.length < 2) {
    return (
      <PageContainer>
        <PageHeader
          title="Compare Schedules"
          description="Select two or more schedules to compare their metrics side by side."
        />
        <div className="flex flex-col items-center gap-4 py-12">
          <p className="text-slate-400">Add schedules to compare</p>
          <div className="w-80">
            {showEmptyPicker ? (
              <SchedulePicker
                multiSelect
                initialSelectedIds={ids}
                placeholder="Search schedules..."
                onConfirm={(selected) => {
                  setShowEmptyPicker(false);
                  setSearchParams({ ids: selected.map((s) => s.schedule_id).join(',') });
                }}
              />
            ) : (
              <button
                type="button"
                onClick={() => setShowEmptyPicker(true)}
                className="w-full rounded-lg border border-dashed border-slate-600 px-4 py-3 text-sm text-slate-400 hover:border-slate-400 hover:text-slate-300"
              >
                + Select schedules to compare
              </button>
            )}
          </div>
        </div>
      </PageContainer>
    );
  }

  return (
    <PageContainer>
      <PageHeader title="Compare Schedules" />

      {/* Schedule chips */}
      <ScheduleChips
        ids={ids}
        scheduleInfoMap={scheduleInfoMap}
        onRemove={handleRemove}
        onAdd={handleAdd}
      />

      {anyError && <ErrorMessage message={`Error loading data: ${anyError.error?.message}`} />}

      {anyLoading && (
        <div className="flex justify-center py-8">
          <LoadingSpinner size="lg" />
        </div>
      )}

      {!anyLoading && (
        <>
          <SummaryTable schedules={schedules} />
          <BlockStatusTable schedules={schedules} />
        </>
      )}
    </PageContainer>
  );
}

export default ComparePage;
