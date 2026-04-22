import { type ReactNode, useCallback, useEffect, useMemo, useState } from 'react';
import { mjdToDate, isValidDate } from '@/constants/dates';
import type { ScheduleAnalysisData } from '../hooks/useScheduleAnalysisData';

const PAGE_SIZE = 100;

function formatMjdUtc(mjd: number | null | undefined): string {
  if (mjd == null || !Number.isFinite(mjd)) return '—';
  const date = mjdToDate(mjd);
  if (!isValidDate(date)) return '—';
  return date
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

function pct(value: number): string {
  return `${(value * 100).toFixed(1)} %`;
}

function fmt2(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value)) return '—';
  return value.toFixed(2);
}

function fmtH(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value)) return '—';
  return `${value.toFixed(2)} h`;
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
    <section
      className={`rounded-2xl border border-slate-800 bg-slate-900/80 p-5 shadow-[0_20px_60px_rgba(2,6,23,0.35)] ${
        isFullscreen ? 'flex h-full flex-col' : ''
      }`}
    >
      <div className="mb-4 flex items-center justify-between gap-4">
        <h2 className="text-lg font-semibold text-white">{title}</h2>
        <div className="flex items-center gap-2">
          {headerActions}
          <FullscreenButton
            isFullscreen={isFullscreen}
            onClick={() => setIsFullscreen((value) => !value)}
            label={title}
          />
        </div>
      </div>
      <div className={isFullscreen ? 'max-h-[calc(100vh-11rem)] overflow-auto' : ''}>
        {children}
      </div>
    </section>
  );

  if (!isFullscreen) return panel;

  return (
    <div className="fixed inset-0 z-50 bg-slate-950/85 p-2 backdrop-blur-sm sm:p-4 lg:p-5">
      <div className="mx-auto h-full w-full max-w-none">{panel}</div>
    </div>
  );
}

interface MetricRow {
  label: string;
  getValue: (schedule: ScheduleAnalysisData) => number | null;
  format: (value: number | null) => string;
  bestIs: 'max' | 'min';
}

const METRIC_ROWS: MetricRow[] = [
  {
    label: 'Scheduled tasks',
    getValue: (schedule) => schedule.insights?.metrics.scheduled_count ?? null,
    format: (value) => (value == null ? '—' : value.toLocaleString()),
    bestIs: 'max',
  },
  {
    label: 'Unscheduled tasks',
    getValue: (schedule) => schedule.insights?.metrics.unscheduled_count ?? null,
    format: (value) => (value == null ? '—' : value.toLocaleString()),
    bestIs: 'min',
  },
  {
    label: 'Scheduling rate',
    getValue: (schedule) => schedule.insights?.metrics.scheduling_rate ?? null,
    format: (value) => (value == null ? '—' : pct(value)),
    bestIs: 'max',
  },
  {
    label: 'Cumulative priority',
    getValue: (schedule) => {
      if (!schedule.insights) return null;
      return schedule.insights.blocks
        .filter((block) => block.scheduled)
        .reduce((sum, block) => sum + block.priority, 0);
    },
    format: (value) => fmt2(value),
    bestIs: 'max',
  },
  {
    label: 'Mean priority (sched.)',
    getValue: (schedule) => schedule.insights?.metrics.mean_priority_scheduled ?? null,
    format: (value) => fmt2(value),
    bestIs: 'max',
  },
  {
    label: 'Scheduled hours',
    getValue: (schedule) => schedule.fragmentation?.metrics.scheduled_hours ?? null,
    format: (value) => fmtH(value),
    bestIs: 'max',
  },
  {
    label: 'Operable hours',
    getValue: (schedule) => schedule.fragmentation?.metrics.operable_hours ?? null,
    format: (value) => fmtH(value),
    bestIs: 'max',
  },
  {
    label: 'Gap count',
    getValue: (schedule) => schedule.fragmentation?.metrics.gap_count ?? null,
    format: (value) => (value == null ? '—' : value.toLocaleString()),
    bestIs: 'min',
  },
  {
    label: 'Gap mean',
    getValue: (schedule) => schedule.fragmentation?.metrics.gap_mean_hours ?? null,
    format: (value) => fmtH(value),
    bestIs: 'min',
  },
  {
    label: 'Gap p90',
    getValue: (schedule) => schedule.fragmentation?.metrics.gap_p90_hours ?? null,
    format: (value) => fmtH(value),
    bestIs: 'min',
  },
  {
    label: 'Largest gap',
    getValue: (schedule) => schedule.fragmentation?.metrics.largest_gap_hours ?? null,
    format: (value) => fmtH(value),
    bestIs: 'min',
  },
];

function formatDelta(delta: number, row: MetricRow): string {
  const sign = delta >= 0 ? '+' : '−';
  const abs = Math.abs(delta);

  if (row.label === 'Scheduling rate') {
    return `${sign}${(abs * 100).toFixed(1)} pp`;
  }

  if (
    row.label.toLowerCase().includes('hour') ||
    row.label.toLowerCase().includes('gap') ||
    row.label === 'Scheduled hours' ||
    row.label === 'Operable hours'
  ) {
    return `${sign}${abs.toFixed(2)} h`;
  }

  if (
    row.label === 'Scheduled tasks' ||
    row.label === 'Unscheduled tasks' ||
    row.label === 'Gap count'
  ) {
    return `${sign}${Math.round(abs).toLocaleString()}`;
  }

  return `${sign}${abs.toFixed(2)}`;
}

function deltaIsGood(delta: number, bestIs: 'max' | 'min'): boolean {
  if (delta === 0) return false;
  return bestIs === 'max' ? delta > 0 : delta < 0;
}

function DeltaBadge({ delta, row }: { delta: number; row: MetricRow }) {
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

export function SummaryTable({ schedules }: { schedules: ScheduleAnalysisData[] }) {
  const [orderMetricLabel, setOrderMetricLabel] = useState<string>('');
  const [orderDirection, setOrderDirection] = useState<'asc' | 'desc'>('desc');
  const reference = schedules[0];
  const comparisons = schedules.slice(1);
  const selectedOrderMetric = useMemo(
    () => METRIC_ROWS.find((row) => row.label === orderMetricLabel) ?? null,
    [orderMetricLabel]
  );

  const orderedComparisons = useMemo(() => {
    if (!selectedOrderMetric) return comparisons;

    const directionMultiplier = orderDirection === 'asc' ? 1 : -1;

    return [...comparisons].sort((left, right) => {
      const leftValue = selectedOrderMetric.getValue(left);
      const rightValue = selectedOrderMetric.getValue(right);

      if (leftValue == null && rightValue == null) return compareText(left.name, right.name);
      if (leftValue == null) return 1;
      if (rightValue == null) return -1;

      const byMetric = compareNumber(leftValue, rightValue);
      if (byMetric !== 0) return byMetric * directionMultiplier;

      const byName = compareText(left.name, right.name);
      if (byName !== 0) return byName;

      return compareNumber(left.id, right.id);
    });
  }, [comparisons, orderDirection, selectedOrderMetric]);

  const handleMetricClick = useCallback(
    (label: string) => {
      if (orderMetricLabel === label) {
        setOrderDirection((d) => (d === 'asc' ? 'desc' : 'asc'));
      } else {
        setOrderMetricLabel(label);
        const metric = METRIC_ROWS.find((r) => r.label === label);
        setOrderDirection(metric?.bestIs === 'max' ? 'desc' : 'asc');
      }
    },
    [orderMetricLabel]
  );

  const renderSortIndicator = (label: string) => {
    if (orderMetricLabel !== label) return <span className="ml-1 text-slate-600">↕</span>;
    return (
      <span className="ml-1 text-sky-400">{orderDirection === 'asc' ? '↑' : '↓'}</span>
    );
  };

  return (
    <ComparePanel
      title="Summary Metrics"
      headerActions={
        selectedOrderMetric ? (
          <button
            type="button"
            onClick={() => setOrderMetricLabel('')}
            className="text-xs text-slate-400 transition-colors hover:text-slate-200"
            title="Clear sort"
          >
            ✕ Clear sort
          </button>
        ) : undefined
      }
    >
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-slate-700 text-slate-400">
              <th className="sticky left-0 bg-slate-900 px-3 py-2 text-left">Metric</th>
              <th className="whitespace-nowrap px-3 py-2 text-right">
                <div>
                  {reference.name}
                  <span className="ml-1 text-xs text-slate-500">#{reference.id}</span>
                </div>
                <div className="mt-0.5">
                  <span className="rounded-sm bg-sky-700/60 px-1 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-sky-300">
                    Reference
                  </span>
                </div>
              </th>
              {orderedComparisons.map((schedule) => (
                <th key={schedule.id} className="whitespace-nowrap px-3 py-2 text-right">
                  {schedule.name}
                  <span className="ml-1 text-xs text-slate-500">#{schedule.id}</span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {METRIC_ROWS.map((row) => {
              const referenceValue = row.getValue(reference);
              const isActive = orderMetricLabel === row.label;

              return (
                <tr key={row.label} className="border-b border-slate-700/50">
                  <td
                    className={`sticky left-0 cursor-pointer select-none whitespace-nowrap bg-slate-900 px-3 py-2 text-left transition-colors ${
                      isActive ? 'text-sky-300 hover:text-sky-200' : 'text-slate-300 hover:text-white'
                    }`}
                    onClick={() => handleMetricClick(row.label)}
                    title={`Sort schedules by ${row.label}`}
                  >
                    <span className="inline-flex items-center gap-0.5">
                      {row.label}
                      {renderSortIndicator(row.label)}
                    </span>
                  </td>

                  <td className="bg-sky-950/20 px-3 py-2 text-right tabular-nums text-slate-200">
                    {reference.isLoading ? '…' : row.format(referenceValue)}
                  </td>

                  {orderedComparisons.map((schedule) => {
                    const value = row.getValue(schedule);
                    const delta =
                      value != null && referenceValue != null ? value - referenceValue : null;
                    const good =
                      delta != null && delta !== 0 ? deltaIsGood(delta, row.bestIs) : null;

                    const colorClass =
                      schedule.isLoading
                        ? 'text-slate-500'
                        : good === true
                          ? 'bg-emerald-950/30 text-emerald-300'
                          : good === false
                            ? 'bg-red-950/30 text-red-400'
                            : 'text-slate-200';

                    return (
                      <td
                        key={schedule.id}
                        className={`px-3 py-2 text-right tabular-nums ${colorClass}`}
                      >
                        {schedule.isLoading ? (
                          '…'
                        ) : (
                          <>
                            {row.format(value)}
                            {good === true && <span className="ml-1 text-xs">▲</span>}
                            {good === false && <span className="ml-1 text-xs">▼</span>}
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

interface BlockEntry {
  original_block_id: string;
  maxPriority: number;
  maxRequestedHours: number;
  perSchedule: Record<
    number,
    { scheduled: boolean; start_mjd: number | null; requested_hours: number }
  >;
}

type BlockSortField =
  | { kind: 'blockId' }
  | { kind: 'priority' }
  | { kind: 'duration' }
  | { kind: 'schedule'; scheduleId: number };

type BlockSortDirection = 'asc' | 'desc';

function compareText(left: string, right: string): number {
  return left.localeCompare(right, undefined, { numeric: true, sensitivity: 'base' });
}

function compareNumber(left: number, right: number): number {
  return left - right;
}

function compareScheduleEntry(
  left:
    | {
        scheduled: boolean;
        start_mjd: number | null;
        requested_hours: number;
      }
    | undefined,
  right:
    | {
        scheduled: boolean;
        start_mjd: number | null;
        requested_hours: number;
      }
    | undefined
): number {
  const leftRank = left ? (left.scheduled ? 0 : 1) : 2;
  const rightRank = right ? (right.scheduled ? 0 : 1) : 2;

  if (leftRank !== rightRank) {
    return leftRank - rightRank;
  }

  if (left?.scheduled && right?.scheduled) {
    const leftStart = left.start_mjd ?? Number.POSITIVE_INFINITY;
    const rightStart = right.start_mjd ?? Number.POSITIVE_INFINITY;
    const byStart = compareNumber(leftStart, rightStart);
    if (byStart !== 0) {
      return byStart;
    }
  }

  return 0;
}

function sortBlocks(
  blocks: BlockEntry[],
  sortField: BlockSortField,
  sortDirection: BlockSortDirection
): BlockEntry[] {
  const directionMultiplier = sortDirection === 'asc' ? 1 : -1;

  return [...blocks].sort((left, right) => {
    let comparison = 0;

    switch (sortField.kind) {
      case 'blockId':
        comparison = compareText(left.original_block_id, right.original_block_id);
        break;
      case 'priority':
        comparison = compareNumber(left.maxPriority, right.maxPriority);
        break;
      case 'duration':
        comparison = compareNumber(left.maxRequestedHours, right.maxRequestedHours);
        break;
      case 'schedule':
        comparison = compareScheduleEntry(
          left.perSchedule[sortField.scheduleId],
          right.perSchedule[sortField.scheduleId]
        );
        break;
    }

    if (comparison !== 0) {
      return comparison * directionMultiplier;
    }

    const byPriority = compareNumber(left.maxPriority, right.maxPriority);
    if (byPriority !== 0) {
      return byPriority * -1;
    }

    return compareText(left.original_block_id, right.original_block_id);
  });
}

export function BlockStatusTable({ schedules }: { schedules: ScheduleAnalysisData[] }) {
  const [page, setPage] = useState(0);
  const [showDifferencesOnly, setShowDifferencesOnly] = useState(false);
  const [sortField, setSortField] = useState<BlockSortField>({ kind: 'priority' });
  const [sortDirection, setSortDirection] = useState<BlockSortDirection>('desc');
  const referenceId = schedules[0]?.id;

  const blockMap = useMemo(() => {
    const map = new Map<string, BlockEntry>();

    for (const schedule of schedules) {
      if (!schedule.insights) continue;

      for (const block of schedule.insights.blocks) {
        const key = block.original_block_id;
        if (!map.has(key)) {
          map.set(key, {
            original_block_id: key,
            maxPriority: block.priority,
            maxRequestedHours: block.requested_hours,
            perSchedule: {},
          });
        }

        const entry = map.get(key)!;
        entry.maxPriority = Math.max(entry.maxPriority, block.priority);
        entry.maxRequestedHours = Math.max(entry.maxRequestedHours, block.requested_hours);
        entry.perSchedule[schedule.id] = {
          scheduled: block.scheduled,
          start_mjd: block.scheduled_start_mjd,
          requested_hours: block.requested_hours,
        };
      }
    }

    return map;
  }, [schedules]);

  const sortedBlocks = useMemo(
    () => sortBlocks([...blockMap.values()], sortField, sortDirection),
    [blockMap, sortDirection, sortField]
  );

  const filteredBlocks = useMemo(() => {
    if (!showDifferencesOnly) return sortedBlocks;

    return sortedBlocks.filter((block) => {
      const states = schedules.map((schedule) => {
        const entry = block.perSchedule[schedule.id];
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
  }, [showDifferencesOnly, schedules, sortDirection, sortField]);

  const isSortFieldActive = (field: BlockSortField): boolean => {
    if (field.kind !== sortField.kind) return false;

    if (field.kind === 'schedule' && sortField.kind === 'schedule') {
      return field.scheduleId === sortField.scheduleId;
    }

    return true;
  };

  const getAriaSort = (field: BlockSortField): 'none' | 'ascending' | 'descending' =>
    isSortFieldActive(field) ? (sortDirection === 'asc' ? 'ascending' : 'descending') : 'none';

  const handleSort = (field: BlockSortField) => {
    if (isSortFieldActive(field)) {
      setSortDirection((value) => (value === 'asc' ? 'desc' : 'asc'));
      return;
    }

    setSortField(field);
    setSortDirection(field.kind === 'blockId' ? 'asc' : 'desc');
  };

  const renderSortLabel = (field: BlockSortField) => {
    if (!isSortFieldActive(field)) {
      return <span className="ml-1 text-slate-600">↕</span>;
    }

    return <span className="ml-1 text-sky-400">{sortDirection === 'asc' ? '↑' : '↓'}</span>;
  };

  const totalPages = Math.ceil(filteredBlocks.length / PAGE_SIZE);
  const pageBlocks = filteredBlocks.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE);

  return (
    <ComparePanel
      title={`Block Status (${filteredBlocks.length} of ${sortedBlocks.length} unique blocks)`}
      headerActions={
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={() => setShowDifferencesOnly((value) => !value)}
            className={`rounded-lg border px-2.5 py-1 text-xs transition-colors ${
              showDifferencesOnly
                ? 'border-emerald-500/60 bg-emerald-900/30 text-emerald-300'
                : 'border-slate-600 text-slate-300 hover:bg-slate-700'
            }`}
            title="Hide rows that are all scheduled or all unscheduled"
          >
            {showDifferencesOnly ? 'Show all rows' : 'Differences only'}
          </button>
          {totalPages > 1 ? (
            <span className="text-xs text-slate-400">
              Page {page + 1} / {totalPages}
            </span>
          ) : null}
        </div>
      }
    >
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-slate-700 text-slate-400">
              <th
                aria-sort={getAriaSort({ kind: 'blockId' })}
                className="sticky left-0 bg-slate-900 px-3 py-2 text-left"
              >
                <button
                  type="button"
                  onClick={() => handleSort({ kind: 'blockId' })}
                  className="inline-flex items-center gap-1 text-left hover:text-white"
                >
                  <span>Block ID</span>
                  {renderSortLabel({ kind: 'blockId' })}
                </button>
              </th>
              <th aria-sort={getAriaSort({ kind: 'priority' })} className="px-3 py-2 text-right">
                <button
                  type="button"
                  onClick={() => handleSort({ kind: 'priority' })}
                  className="inline-flex items-center gap-1 hover:text-white"
                >
                  <span>Priority</span>
                  {renderSortLabel({ kind: 'priority' })}
                </button>
              </th>
              <th aria-sort={getAriaSort({ kind: 'duration' })} className="px-3 py-2 text-right">
                <button
                  type="button"
                  onClick={() => handleSort({ kind: 'duration' })}
                  className="inline-flex items-center gap-1 hover:text-white"
                >
                  <span>Duration (min)</span>
                  {renderSortLabel({ kind: 'duration' })}
                </button>
              </th>
              {schedules.map((schedule) => (
                <th
                  key={schedule.id}
                  aria-sort={getAriaSort({ kind: 'schedule', scheduleId: schedule.id })}
                  className="whitespace-nowrap px-3 py-2 text-center"
                >
                  <button
                    type="button"
                    onClick={() => handleSort({ kind: 'schedule', scheduleId: schedule.id })}
                    className="inline-flex items-center gap-1 hover:text-white"
                  >
                    <span>{schedule.name}</span>
                    {renderSortLabel({ kind: 'schedule', scheduleId: schedule.id })}
                  </button>
                  {schedule.id === referenceId ? (
                    <div className="mt-0.5">
                      <span className="rounded-sm bg-sky-700/60 px-1 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-sky-300">
                        Reference
                      </span>
                    </div>
                  ) : null}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {pageBlocks.map((block) => (
              <tr key={block.original_block_id} className="border-b border-slate-700/50">
                <td className="sticky left-0 bg-slate-900 px-3 py-2 font-mono text-white">
                  {formatBlockId(block.original_block_id)}
                </td>
                <td className="px-3 py-2 text-right tabular-nums text-slate-300">
                  {block.maxPriority.toFixed(2)}
                </td>
                <td className="px-3 py-2 text-right tabular-nums text-slate-300">
                  {Math.round(block.maxRequestedHours * 60).toLocaleString()}
                </td>
                {schedules.map((schedule) => {
                  const entry = block.perSchedule[schedule.id];
                  const referenceEntry = block.perSchedule[referenceId];
                  const isReference = schedule.id === referenceId;

                  if (!entry) {
                    return (
                      <td key={schedule.id} className="px-3 py-2 text-center text-slate-600">
                        —
                      </td>
                    );
                  }

                  const gainedVsRef =
                    !isReference && referenceEntry && entry.scheduled && !referenceEntry.scheduled;

                  return (
                    <td
                      key={schedule.id}
                      className={`px-3 py-2 text-center ${isReference ? 'bg-sky-950/20' : ''}`}
                    >
                      {entry.scheduled ? (
                        <span className="font-mono text-xs text-emerald-400">
                          {gainedVsRef ? (
                            <span
                              className="mr-1 text-[10px] font-bold text-emerald-400"
                              title="Gained vs reference"
                            >
                              ▲
                            </span>
                          ) : null}
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

      {totalPages > 1 ? (
        <div className="mt-4 flex items-center justify-center gap-3">
          <button
            type="button"
            onClick={() => setPage((value) => Math.max(0, value - 1))}
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
            onClick={() => setPage((value) => Math.min(totalPages - 1, value + 1))}
            disabled={page === totalPages - 1}
            className="rounded-lg border border-slate-600 px-3 py-1.5 text-sm text-slate-300 hover:bg-slate-700 disabled:opacity-40"
          >
            Next
          </button>
        </div>
      ) : null}
    </ComparePanel>
  );
}
