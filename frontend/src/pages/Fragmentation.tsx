/**
 * Fragmentation page — measures schedule fragmentation against the
 * telescope-operable baseline (dark_periods, fallback astronomical_nights).
 *
 * Supports side-by-side comparison via `?compare=<schedule_id>`: we fetch the
 * same single-schedule endpoint twice and diff on the frontend. No dedicated
 * compare endpoint is used in v1.
 */
import { useMemo } from 'react';
import { useParams, useSearchParams } from 'react-router-dom';
import { useFragmentation } from '@/hooks';
import { SchedulePicker } from '@/features/schedules';
import {
  LoadingSpinner,
  ErrorMessage,
  Icon,
  MetricCard,
  PageHeader,
  PageContainer,
  MetricsGrid,
} from '@/components';
import type {
  FragmentationData,
  FragmentationSegment,
  FragmentationSegmentKind,
  ReasonBreakdownEntry,
  UnscheduledReasonSummary,
} from '@/api/types';

function getScheduleDisplayName(
  schedule: Pick<FragmentationData, 'schedule_id' | 'schedule_name'>
) {
  const trimmedName = schedule.schedule_name.trim();
  return trimmedName.length > 0 ? trimmedName : `Schedule ${schedule.schedule_id}`;
}

// ──────────────────────────────────────────────────────────────────────────
// Presentation helpers
// ──────────────────────────────────────────────────────────────────────────

export const SEGMENT_LABELS: Record<FragmentationSegmentKind, string> = {
  non_operable: 'Non-operable',
  scheduled: 'Scheduled',
  no_target_visible: 'No target visible',
  visible_but_no_task_fits: 'Visible, no task fits',
  feasible_but_unused: 'Feasible, unused',
};

const SEGMENT_COLORS: Record<FragmentationSegmentKind, string> = {
  non_operable: 'bg-slate-700',
  scheduled: 'bg-emerald-500',
  no_target_visible: 'bg-indigo-600',
  visible_but_no_task_fits: 'bg-amber-500',
  feasible_but_unused: 'bg-rose-500',
};

const UNSCHEDULED_REASON_LABELS: Record<string, string> = {
  no_visibility: 'No visibility',
  no_contiguous_fit: 'No contiguous fit',
  requested_exceeds_total_visibility: 'Requested > total visibility',
  other_validation_issue: 'Other validation issue',
  feasible_but_unscheduled: 'Feasible but unscheduled',
};

function formatHours(h: number): string {
  return `${h.toFixed(2)}h`;
}

function formatPercent(frac: number): string {
  return `${(frac * 100).toFixed(1)}%`;
}

function formatDelta(value: number, suffix = 'h'): string {
  const sign = value > 0 ? '+' : '';
  return `${sign}${value.toFixed(2)}${suffix}`;
}

function deltaClass(value: number, invert = false): string {
  if (Math.abs(value) < 1e-9) return 'text-slate-400';
  const positiveGood = invert ? value < 0 : value > 0;
  return positiveGood ? 'text-emerald-400' : 'text-rose-400';
}

// ──────────────────────────────────────────────────────────────────────────
// Subcomponents
// ──────────────────────────────────────────────────────────────────────────

export function FragmentationTimelineStrip({
  segments,
  windowStart,
  windowEnd,
}: {
  segments: FragmentationSegment[];
  windowStart: number;
  windowEnd: number;
}) {
  const span = Math.max(windowEnd - windowStart, 1e-9);

  return (
    <div
      className="relative flex h-6 w-full overflow-hidden rounded-md border border-slate-700"
      aria-label="Classified timeline strip"
      role="img"
    >
      {segments.map((seg, i) => {
        const width = Math.max(0, ((seg.stop_mjd - seg.start_mjd) / span) * 100);
        if (width <= 0) return null;
        return (
          <div
            key={i}
            className={SEGMENT_COLORS[seg.kind]}
            style={{ width: `${width}%` }}
            title={`${SEGMENT_LABELS[seg.kind]} — ${formatHours(seg.duration_hours)}`}
          />
        );
      })}
    </div>
  );
}

export function ReasonBreakdownChart({ entries }: { entries: ReasonBreakdownEntry[] }) {
  const max = Math.max(...entries.map((e) => e.total_hours), 1e-9);
  return (
    <div className="space-y-2">
      {entries.map((entry) => {
        const width = (entry.total_hours / max) * 100;
        return (
          <div key={entry.kind} className="flex items-center gap-2">
            <div className="w-44 shrink-0 text-xs text-slate-400">{SEGMENT_LABELS[entry.kind]}</div>
            <div className="relative h-4 flex-1 overflow-hidden rounded bg-slate-700/30">
              <div
                className={`${SEGMENT_COLORS[entry.kind]} h-full`}
                style={{ width: `${width}%` }}
              />
            </div>
            <div className="w-32 shrink-0 text-right text-xs tabular-nums text-slate-300">
              {formatHours(entry.total_hours)} · {formatPercent(entry.fraction_of_operable)}
            </div>
          </div>
        );
      })}
    </div>
  );
}

export function UnscheduledReasonsList({ summaries }: { summaries: UnscheduledReasonSummary[] }) {
  return (
    <div className="space-y-2">
      {summaries.map((s) => (
        <div
          key={s.reason}
          className="rounded-md border border-slate-700/50 bg-slate-700/20 px-3 py-2"
        >
          <div className="flex items-center justify-between">
            <span className="text-sm text-slate-300">
              {UNSCHEDULED_REASON_LABELS[s.reason] ?? s.reason}
            </span>
            <span className="rounded bg-slate-800 px-2 py-0.5 text-xs tabular-nums text-white">
              {s.block_count}
            </span>
          </div>
          {s.example_block_ids.length > 0 && (
            <div className="mt-1 truncate text-xs text-slate-500">
              e.g. {s.example_block_ids.slice(0, 5).join(', ')}
              {s.example_block_ids.length > 5 ? ' …' : ''}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────────────
// Single-schedule section — also used (twice) in compare mode
// ──────────────────────────────────────────────────────────────────────────

interface SchedulePanelProps {
  data: FragmentationData;
  title: string;
  compareMetrics?: FragmentationData['metrics'];
}

function SchedulePanel({ data, title, compareMetrics }: SchedulePanelProps) {
  const m = data.metrics;
  const delta = compareMetrics
    ? {
        requested: m.requested_hours - compareMetrics.requested_hours,
        scheduled: m.scheduled_hours - compareMetrics.scheduled_hours,
        operable: m.operable_hours - compareMetrics.operable_hours,
        idle: m.idle_operable_hours - compareMetrics.idle_operable_hours,
      }
    : null;

  return (
    <div className="space-y-4 rounded-lg border border-slate-700 bg-slate-800/30 p-4">
      <div className="flex items-center justify-between">
        <h2 className="text-base font-semibold text-white">{title}</h2>
        <span className="rounded bg-slate-700/50 px-2 py-0.5 text-xs text-slate-300">
          baseline: {data.operable_source}
        </span>
      </div>

      <MetricsGrid>
        <MetricCard label="Requested" value={formatHours(m.requested_hours)} />
        <MetricCard
          label="Scheduled"
          value={formatHours(m.scheduled_hours)}
          icon={<Icon name="check-circle" />}
        />
        <MetricCard
          label="Operable"
          value={formatHours(m.operable_hours)}
          icon={<Icon name="chart-bar" />}
        />
        <MetricCard label="Idle operable" value={formatHours(m.idle_operable_hours)} />
      </MetricsGrid>

      {delta && (
        <div className="grid grid-cols-2 gap-2 rounded-md bg-slate-900/50 p-3 text-xs sm:grid-cols-4">
          <div>
            <p className="text-slate-500">Δ requested</p>
            <p className={`tabular-nums ${deltaClass(delta.requested)}`}>
              {formatDelta(delta.requested)}
            </p>
          </div>
          <div>
            <p className="text-slate-500">Δ scheduled</p>
            <p className={`tabular-nums ${deltaClass(delta.scheduled)}`}>
              {formatDelta(delta.scheduled)}
            </p>
          </div>
          <div>
            <p className="text-slate-500">Δ operable</p>
            <p className={`tabular-nums ${deltaClass(delta.operable)}`}>
              {formatDelta(delta.operable)}
            </p>
          </div>
          <div>
            <p className="text-slate-500">Δ idle</p>
            <p className={`tabular-nums ${deltaClass(delta.idle, true)}`}>
              {formatDelta(delta.idle)}
            </p>
          </div>
        </div>
      )}

      <section>
        <h3 className="mb-2 text-sm font-medium text-slate-300">Reason breakdown</h3>
        <ReasonBreakdownChart entries={data.reason_breakdown} />
      </section>

      <section>
        <h3 className="mb-2 text-sm font-medium text-slate-300">Classified timeline</h3>
        <FragmentationTimelineStrip
          segments={data.segments}
          windowStart={data.schedule_window.start_mjd}
          windowEnd={data.schedule_window.end_mjd}
        />
        <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-slate-400">
          {(Object.keys(SEGMENT_LABELS) as FragmentationSegmentKind[]).map((k) => (
            <span key={k} className="flex items-center gap-1">
              <span className={`inline-block h-2 w-3 rounded-sm ${SEGMENT_COLORS[k]}`} />
              {SEGMENT_LABELS[k]}
            </span>
          ))}
        </div>
      </section>

      <section>
        <h3 className="mb-2 text-sm font-medium text-slate-300">Unscheduled reasons</h3>
        <UnscheduledReasonsList summaries={data.unscheduled_reasons} />
      </section>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────────────────
// Page
// ──────────────────────────────────────────────────────────────────────────

function Fragmentation() {
  const { scheduleId } = useParams();
  const [searchParams, setSearchParams] = useSearchParams();
  const id = parseInt(scheduleId ?? '0', 10);

  const compareParam = searchParams.get('compare');
  const compareId = compareParam ? parseInt(compareParam, 10) : 0;

  const primary = useFragmentation(id);
  const secondary = useFragmentation(compareId);

  const compareMode = compareId > 0;

  const headerActions = useMemo(
    () => (
      <div className="flex items-center gap-2">
        {compareMode ? (
          <button
            className="rounded-md border border-slate-600 bg-slate-700/50 px-3 py-1.5 text-xs text-slate-200 hover:bg-slate-700"
            onClick={() => {
              const next = new URLSearchParams(searchParams);
              next.delete('compare');
              setSearchParams(next);
            }}
          >
            Clear comparison
          </button>
        ) : (
          <div className="w-64">
            <SchedulePicker
              excludeId={id}
              placeholder="Compare with..."
              onSelect={(s) => {
                const next = new URLSearchParams(searchParams);
                next.set('compare', String(s.schedule_id));
                setSearchParams(next);
              }}
            />
          </div>
        )}
      </div>
    ),
    [compareMode, id, searchParams, setSearchParams]
  );

  if (primary.isLoading || (compareMode && secondary.isLoading)) {
    return (
      <div className="flex h-full items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (primary.error) {
    return (
      <ErrorMessage
        title="Failed to load fragmentation data"
        message={(primary.error as Error).message}
        onRetry={() => primary.refetch()}
      />
    );
  }

  if (!primary.data) {
    return <ErrorMessage message="No fragmentation data available" />;
  }

  if (compareMode && secondary.error) {
    return (
      <ErrorMessage
        title="Failed to load comparison schedule"
        message={(secondary.error as Error).message}
        onRetry={() => secondary.refetch()}
      />
    );
  }

  return (
    <PageContainer>
      <PageHeader
        title="Fragmentation"
        description={
          compareMode && secondary.data
            ? `${getScheduleDisplayName(primary.data)} vs ${getScheduleDisplayName(secondary.data)}`
            : `${getScheduleDisplayName(primary.data)} measured against the telescope-operable baseline.`
        }
        actions={headerActions}
      />

      {compareMode && secondary.data ? (
        <div className="grid gap-4 lg:grid-cols-2">
          <SchedulePanel
            data={primary.data}
            title={getScheduleDisplayName(primary.data)}
            compareMetrics={secondary.data.metrics}
          />
          <SchedulePanel
            data={secondary.data}
            title={getScheduleDisplayName(secondary.data)}
            compareMetrics={primary.data.metrics}
          />
        </div>
      ) : (
        <SchedulePanel data={primary.data} title={getScheduleDisplayName(primary.data)} />
      )}
    </PageContainer>
  );
}

export default Fragmentation;
