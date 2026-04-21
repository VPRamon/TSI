/**
 * Compare page — multi-schedule field table with reference baseline.
 *
 * Route: /schedules/:scheduleId/compare/:otherIds
 *
 * scheduleId is the reference schedule. otherIds is a comma-separated list of
 * additional schedule IDs. All non-reference schedules show their value plus a
 * coloured Δ relative to the reference (green = better, red = worse).
 *
 * Shows:
 *   1. Summary metrics table (rows = metrics, columns = schedules)
 *   2. Block status table (rows = tasks aligned by original_block_id, columns = schedules)
 */
import { type ReactNode, useEffect, useState, useMemo } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
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

function formatBlockId(blockId: string): string {
  const parts = blockId
    .split(':')
    .map((part) => part.trim())
    .filter((part) => part.length > 0);
  if (parts.length > 1 && parts.every((part) => part === parts[0])) {
    return parts[0];
  }
  return blockId;
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

function FullscreenButton({
  isFullscreen,
  onClick,
  label,
}: {
  isFullscreen: boolean;
  onClick: () => void;
  label: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="rounded-lg border border-slate-600 p-1.5 text-slate-300 transition-colors hover:bg-slate-700"
      aria-label={isFullscreen ? `Exit full screen for ${label}` : `Enter full screen for ${label}`}
      title={isFullscreen ? 'Exit full screen' : 'Full screen'}
    >
      {isFullscreen ? (
        <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M9 15H5v4m0-4 5 5m5-5h4v4m0-4-5 5M9 9H5V5m0 4 5-5m5 5h4V5m0 4-5-5"
          />
        </svg>
      ) : (
        <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M8 3H5a2 2 0 00-2 2v3m16 0V5a2 2 0 00-2-2h-3m0 18h3a2 2 0 002-2v-3M8 21H5a2 2 0 01-2-2v-3"
          />
        </svg>
      )}
    </button>
  );
}

function ComparePanel({
  title,
  children,
  headerActions,
}: {
  title: string;
  children: ReactNode;
  headerActions?: ReactNode;
}) {
  const [isFullscreen, setIsFullscreen] = useState(false);

  useEffect(() => {
    if (!isFullscreen) return;

    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setIsFullscreen(false);
      }
    };

    document.addEventListener('keydown', onKeyDown);

    return () => {
      document.body.style.overflow = previousOverflow;
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [isFullscreen]);

  const panel = (
    <ChartPanel
      title={title}
      className={isFullscreen ? 'flex h-full flex-col' : ''}
      headerActions={
        <div className="flex items-center gap-2">
          {headerActions}
          <FullscreenButton
            isFullscreen={isFullscreen}
            onClick={() => setIsFullscreen((value) => !value)}
            label={title}
          />
        </div>
      }
    >
      <div className={isFullscreen ? 'max-h-[calc(100vh-11rem)] overflow-auto' : ''}>{children}</div>
    </ChartPanel>
  );

  if (!isFullscreen) return panel;

  return (
    <div className="fixed inset-0 z-50 bg-slate-950/85 p-2 backdrop-blur-sm sm:p-4 lg:p-5">
      <div className="mx-auto h-full w-full max-w-none">
        {panel}
      </div>
    </div>
  );
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

// ─── Delta helpers ───────────────────────────────────────────────────────────

/** Format a raw numeric delta with sign. */
function formatDelta(delta: number, row: MetricRow): string {
  const sign = delta >= 0 ? '+' : '−';
  const abs = Math.abs(delta);

  if (row.label === 'Scheduling rate') {
    return `${sign}${(abs * 100).toFixed(1)} pp`;
  }
  if (row.label.toLowerCase().includes('hour') ||
      row.label.toLowerCase().includes('gap') ||
      row.label === 'Scheduled hours' ||
      row.label === 'Operable hours') {
    return `${sign}${abs.toFixed(2)} h`;
  }
  if (row.label === 'Scheduled tasks' || row.label === 'Unscheduled tasks' || row.label === 'Gap count') {
    return `${sign}${Math.round(abs).toLocaleString()}`;
  }
  return `${sign}${abs.toFixed(2)}`;
}

/** True if a positive delta means the comparison schedule is better than ref. */
function deltaIsGood(delta: number, bestIs: 'max' | 'min'): boolean {
  if (delta === 0) return false;
  return bestIs === 'max' ? delta > 0 : delta < 0;
}

function DeltaBadge({
  delta,
  row,
}: {
  delta: number;
  row: MetricRow;
}) {
  if (delta === 0) return <span className="ml-1 text-xs text-slate-500">—</span>;
  const good = deltaIsGood(delta, row.bestIs);
  const colorClass = good ? 'text-emerald-400' : 'text-red-400';
  const arrow = good ? '▲' : '▼';
  return (
    <span className={`ml-1 text-xs tabular-nums ${colorClass}`} title="vs reference">
      {arrow} {formatDelta(delta, row)}
    </span>
  );
}

function SummaryTable({ schedules }: { schedules: ScheduleData[] }) {
  const ref = schedules[0];
  const rest = schedules.slice(1);

  return (
    <ComparePanel title="Summary Metrics">
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-slate-700 text-slate-400">
              <th className="sticky left-0 bg-slate-800/90 px-3 py-2 text-left">Metric</th>
              {/* Reference column header */}
              <th className="px-3 py-2 text-right whitespace-nowrap">
                <div>{ref.name}<span className="ml-1 text-xs text-slate-500">#{ref.id}</span></div>
                <div className="mt-0.5">
                  <span className="rounded-sm bg-sky-700/60 px-1 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-sky-300">
                    Reference
                  </span>
                </div>
              </th>
              {/* Comparison column headers */}
              {rest.map((s) => (
                <th key={s.id} className="px-3 py-2 text-right whitespace-nowrap">
                  {s.name}
                  <span className="ml-1 text-xs text-slate-500">#{s.id}</span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {METRIC_ROWS.map((row) => {
              const refValue = row.getValue(ref);

              return (
                <tr key={row.label} className="border-b border-slate-700/50">
                  <td className="sticky left-0 bg-slate-800/90 px-3 py-2 text-slate-300 whitespace-nowrap">
                    {row.label}
                  </td>
                  {/* Reference cell — plain value, sky tint */}
                  <td className="bg-sky-950/20 px-3 py-2 text-right tabular-nums text-slate-200">
                    {ref.isLoading ? '…' : row.format(refValue)}
                  </td>
                  {/* Comparison cells — value + delta badge */}
                  {rest.map((s) => {
                    const v = row.getValue(s);
                    const delta =
                      v != null && refValue != null ? v - refValue : null;
                    return (
                      <td
                        key={s.id}
                        className={`px-3 py-2 text-right tabular-nums ${s.isLoading ? 'text-slate-500' : 'text-slate-300'}`}
                      >
                        {s.isLoading ? (
                          '…'
                        ) : (
                          <>
                            {row.format(v)}
                            {delta != null && refValue != null && (
                              <DeltaBadge delta={delta} row={row} />
                            )}
                          </>
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
    </ComparePanel>
  );
}

// ─── Block status table ──────────────────────────────────────────────────────

interface BlockEntry {
  original_block_id: string;
  maxPriority: number;
  maxRequestedHours: number;
  perSchedule: Record<
    number,
    { scheduled: boolean; start_mjd: number | null; requested_hours: number }
  >;
}

function BlockStatusTable({ schedules }: { schedules: ScheduleData[] }) {
  const [page, setPage] = useState(0);
  const [showDifferencesOnly, setShowDifferencesOnly] = useState(false);
  const refId = schedules[0]?.id;

  const blockMap = useMemo(() => {
    const map = new Map<string, BlockEntry>();
    for (const s of schedules) {
      if (!s.insights) continue;
      for (const b of s.insights.blocks) {
        const key = b.original_block_id;
        if (!map.has(key)) {
          map.set(key, {
            original_block_id: key,
            maxPriority: b.priority,
            maxRequestedHours: b.requested_hours,
            perSchedule: {},
          });
        }
        const entry = map.get(key)!;
        entry.maxPriority = Math.max(entry.maxPriority, b.priority);
        entry.maxRequestedHours = Math.max(entry.maxRequestedHours, b.requested_hours);
        entry.perSchedule[s.id] = {
          scheduled: b.scheduled,
          start_mjd: b.scheduled_start_mjd,
          requested_hours: b.requested_hours,
        };
      }
    }
    return map;
  }, [schedules]);

  const sortedBlocks = useMemo(
    () => [...blockMap.values()].sort((a, b) => b.maxPriority - a.maxPriority),
    [blockMap]
  );

  const filteredBlocks = useMemo(() => {
    if (!showDifferencesOnly) return sortedBlocks;

    return sortedBlocks.filter((block) => {
      const states = schedules.map((s) => {
        const entry = block.perSchedule[s.id];
        if (!entry) return 'missing' as const;
        return entry.scheduled ? 'scheduled' : 'unscheduled';
      });

      const allScheduled = states.every((state) => state === 'scheduled');
      const allUnscheduled = states.every((state) => state === 'unscheduled');

      return !allScheduled && !allUnscheduled;
    });
  }, [showDifferencesOnly, sortedBlocks, schedules]);

  useEffect(() => {
    setPage(0);
  }, [showDifferencesOnly, schedules]);

  const totalPages = Math.ceil(filteredBlocks.length / PAGE_SIZE);
  const pageBlocks = filteredBlocks.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE);

  return (
    <ComparePanel
      title={`Block Status (${filteredBlocks.length} of ${sortedBlocks.length} unique blocks)`}
      headerActions={
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={() => setShowDifferencesOnly((v) => !v)}
            className={`rounded-lg border px-2.5 py-1 text-xs transition-colors ${
              showDifferencesOnly
                ? 'border-emerald-500/60 bg-emerald-900/30 text-emerald-300'
                : 'border-slate-600 text-slate-300 hover:bg-slate-700'
            }`}
            title="Hide rows that are all scheduled or all unscheduled"
          >
            {showDifferencesOnly ? 'Show all rows' : 'Differences only'}
          </button>
          {totalPages > 1 && (
            <span className="text-xs text-slate-400">
              Page {page + 1} / {totalPages}
            </span>
          )}
        </div>
      }
    >
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-slate-700 text-slate-400">
              <th className="sticky left-0 bg-slate-800/90 px-3 py-2 text-left">Block ID</th>
              <th className="px-3 py-2 text-right">Priority</th>
              <th className="px-3 py-2 text-right">Duration (min)</th>
              {schedules.map((s) => (
                <th key={s.id} className="px-3 py-2 text-center whitespace-nowrap">
                  <div>{s.name}</div>
                  {s.id === refId && (
                    <div className="mt-0.5">
                      <span className="rounded-sm bg-sky-700/60 px-1 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-sky-300">
                        Reference
                      </span>
                    </div>
                  )}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {pageBlocks.map((block) => (
              <tr key={block.original_block_id} className="border-b border-slate-700/50">
                <td className="sticky left-0 bg-slate-800/90 px-3 py-2 font-mono text-white">
                  {formatBlockId(block.original_block_id)}
                </td>
                <td className="px-3 py-2 text-right text-slate-300 tabular-nums">
                  {block.maxPriority.toFixed(2)}
                </td>
                <td className="px-3 py-2 text-right text-slate-300 tabular-nums">
                  {Math.round(block.maxRequestedHours * 60).toLocaleString()}
                </td>
                {schedules.map((s) => {
                  const entry = block.perSchedule[s.id];
                  const refEntry = block.perSchedule[refId];
                  const isRef = s.id === refId;

                  if (!entry) {
                    return (
                      <td key={s.id} className="px-3 py-2 text-center text-slate-600">
                        —
                      </td>
                    );
                  }

                  // Gained vs reference indicator (only on scheduled cells)
                  const gainedVsRef =
                    !isRef && refEntry && entry.scheduled && !refEntry.scheduled;

                  return (
                    <td key={s.id} className={`px-3 py-2 text-center ${isRef ? 'bg-sky-950/20' : ''}`}>
                      {entry.scheduled ? (
                        <span className="text-emerald-400 text-xs font-mono">
                          {gainedVsRef && (
                            <span className="mr-1 text-[10px] font-bold text-emerald-400" title="Gained vs reference">▲</span>
                          )}
                          {formatMjdUtc(entry.start_mjd)}
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
    </ComparePanel>
  );
}

// ─── Schedule chip header ────────────────────────────────────────────────────

function ScheduleChips({
  comparisonIds,
  refId,
  scheduleInfoMap,
  onRemove,
  onAdd,
}: {
  comparisonIds: number[];
  refId: number;
  scheduleInfoMap: Map<number, ScheduleInfo>;
  onRemove: (id: number) => void;
  onAdd: (schedules: ScheduleInfo[]) => void;
}) {
  const [showPicker, setShowPicker] = useState(false);

  return (
    <div className="flex flex-wrap items-center gap-2">
      {comparisonIds.map((id) => {
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
              excludeId={refId}
              initialSelectedIds={comparisonIds}
              placeholder="Search schedules to compare..."
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
  const { scheduleId: scheduleIdParam, otherIds: otherIdsParam } = useParams<{
    scheduleId: string;
    otherIds?: string;
  }>();
  const navigate = useNavigate();
  const [showEmptyPicker, setShowEmptyPicker] = useState(false);

  const refId = useMemo(
    () => parseInt(scheduleIdParam ?? '0', 10),
    [scheduleIdParam]
  );

  const comparisonIds = useMemo(() => {
    const others = (otherIdsParam ?? '')
      .split(',')
      .map((s) => parseInt(s.trim(), 10))
      .filter((n) => Number.isFinite(n) && n > 0 && n !== refId);
    return [...new Set(others)].slice(0, 9);
  }, [otherIdsParam, refId]);

  const ids = useMemo(
    () => (refId > 0 ? [refId, ...comparisonIds] : comparisonIds),
    [comparisonIds, refId]
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
    const next = comparisonIds.filter((comparisonId) => comparisonId !== id);
    if (next.length === 0) {
      navigate(`/schedules/${refId}/compare`);
      return;
    }
    navigate(`/schedules/${refId}/compare/${next.join(',')}`);
  };

  const handleAdd = (selected: ScheduleInfo[]) => {
    const next = [...new Set(
      selected
        .map((schedule) => schedule.schedule_id)
        .filter((scheduleId) => scheduleId !== refId)
    )].slice(0, 9);

    if (next.length === 0) {
      navigate(`/schedules/${refId}/compare`);
      return;
    }

    navigate(`/schedules/${refId}/compare/${next.join(',')}`);
  };

  const anyLoading = schedules.some((s) => s.isLoading);
  const anyError = schedules.find((s) => s.error);

  // Empty state
  if (comparisonIds.length === 0) {
    return (
      <PageContainer>
        <PageHeader
          title="Compare Schedules"
          description="Select one or more schedules to compare against the loaded schedule."
        />
        <div className="flex flex-col items-center gap-4 py-12">
          <p className="text-slate-400">Add schedules to compare</p>
          <div className="w-80">
            {showEmptyPicker ? (
              <SchedulePicker
                multiSelect
                excludeId={refId}
                initialSelectedIds={comparisonIds}
                placeholder="Search schedules to compare..."
                onConfirm={(selected) => {
                  setShowEmptyPicker(false);
                  const next = [...new Set(
                    selected
                      .map((schedule) => schedule.schedule_id)
                      .filter((scheduleId) => scheduleId !== refId)
                  )].slice(0, 9);
                  navigate(`/schedules/${refId}/compare/${next.join(',')}`);
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
      <PageHeader
        title="Compare Schedules"
        description="The loaded schedule is used as the reference baseline."
      />

      {/* Schedule chips */}
      <ScheduleChips
        comparisonIds={comparisonIds}
        refId={refId}
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
