/**
 * Advanced page — multi-schedule comparison launcher.
 *
 * Groups available schedules by (observatory × schedule-period).
 * The user selects ≥ 2 schedules within one group, designates a reference,
 * and is redirected to /schedules/:refId/compare/:otherIds.
 *
 * Hard constraint: once a schedule from a group is checked, only schedules
 * from that same group can be checked.
 */
import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useSchedules } from '@/hooks';
import type { ScheduleInfo } from '@/api/types';
import { ErrorMessage, LoadingSpinner, PageContainer, PageHeader } from '@/components';

// ---------------------------------------------------------------------------
// MJD helpers
// ---------------------------------------------------------------------------

const MJD_TO_UNIX_OFFSET_DAYS = 40587;

function mjdToDate(mjd: number): Date {
  return new Date((mjd - MJD_TO_UNIX_OFFSET_DAYS) * 86_400_000);
}

function mjdRangeLabel(startMjd: number, endMjd: number): string {
  const fmt = (d: Date) =>
    d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' });
  return `${fmt(mjdToDate(startMjd))} – ${fmt(mjdToDate(endMjd))}`;
}

// ---------------------------------------------------------------------------
// Group-key computation
// ---------------------------------------------------------------------------

interface ScheduleGroup {
  key: string;
  observatoryLabel: string;
  periodLabel: string;
  schedules: ScheduleInfo[];
}

function buildGroupKey(info: ScheduleInfo): string {
  const { lat_deg, lon_deg } = info.observer_location;
  const { start_mjd, end_mjd } = info.schedule_period;
  return `${lat_deg.toFixed(3)},${lon_deg.toFixed(3)}|${start_mjd.toFixed(2)},${end_mjd.toFixed(2)}`;
}

function observatoryLabel(info: ScheduleInfo): string {
  const { lat_deg, lon_deg } = info.observer_location;
  const latStr = `${Math.abs(lat_deg).toFixed(2)}° ${lat_deg >= 0 ? 'N' : 'S'}`;
  const lonStr = `${Math.abs(lon_deg).toFixed(2)}° ${lon_deg >= 0 ? 'E' : 'W'}`;
  return `${latStr}, ${lonStr}`;
}

function groupSchedules(schedules: ScheduleInfo[]): ScheduleGroup[] {
  const map = new Map<string, ScheduleGroup>();

  for (const info of schedules) {
    const key = buildGroupKey(info);
    if (!map.has(key)) {
      map.set(key, {
        key,
        observatoryLabel: observatoryLabel(info),
        periodLabel: mjdRangeLabel(info.schedule_period.start_mjd, info.schedule_period.end_mjd),
        schedules: [],
      });
    }
    map.get(key)!.schedules.push(info);
  }

  return [...map.values()];
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function GroupCard({
  group,
  selectedIds,
  lockedGroupKey,
  onToggle,
}: {
  group: ScheduleGroup;
  selectedIds: Set<number>;
  lockedGroupKey: string | null;
  onToggle: (info: ScheduleInfo) => void;
}) {
  const isLocked = lockedGroupKey !== null && lockedGroupKey !== group.key;
  const hasSelection = group.schedules.some((s) => selectedIds.has(s.schedule_id));

  return (
    <section
      className={`rounded-2xl border p-5 transition-opacity ${
        isLocked
          ? 'pointer-events-none border-slate-800 opacity-40'
          : hasSelection
            ? 'border-sky-500/50 bg-sky-950/20'
            : 'border-slate-700 bg-slate-900/60'
      }`}
      aria-disabled={isLocked}
    >
      {/* Group header */}
      <div className="mb-4 flex flex-col gap-0.5">
        <div className="flex items-center gap-2">
          <svg
            className="h-4 w-4 shrink-0 text-slate-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"
            />
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>
          <span className="text-sm font-semibold text-slate-200">{group.observatoryLabel}</span>
        </div>
        <div className="flex items-center gap-2 pl-6">
          <svg
            className="h-3.5 w-3.5 shrink-0 text-slate-500"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
            />
          </svg>
          <span className="text-xs text-slate-400">{group.periodLabel}</span>
        </div>
      </div>

      {/* Schedule checkboxes */}
      <ul className="space-y-2">
        {group.schedules.map((schedule) => {
          const checked = selectedIds.has(schedule.schedule_id);
          return (
            <li key={schedule.schedule_id}>
              <label className="flex cursor-pointer items-center gap-3 rounded-xl px-3 py-2.5 transition-colors hover:bg-slate-800/60">
                <input
                  type="checkbox"
                  className="h-4 w-4 rounded border-slate-600 accent-sky-500"
                  checked={checked}
                  onChange={() => onToggle(schedule)}
                  disabled={isLocked}
                />
                <span className="flex-1 truncate text-sm font-medium text-white">
                  {schedule.schedule_name}
                </span>
                <span className="shrink-0 text-xs text-slate-500">#{schedule.schedule_id}</span>
              </label>
            </li>
          );
        })}
      </ul>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

function AdvancedPage() {
  const navigate = useNavigate();
  const { data, isLoading, error } = useSchedules();

  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [lockedGroupKey, setLockedGroupKey] = useState<string | null>(null);
  const [referenceId, setReferenceId] = useState<number | null>(null);

  const allSchedules = data?.schedules ?? [];
  const groups = useMemo(() => groupSchedules(allSchedules), [allSchedules]);

  // Map from id → ScheduleInfo for quick lookup
  const infoMap = useMemo(() => {
    const map = new Map<number, ScheduleInfo>();
    allSchedules.forEach((s) => map.set(s.schedule_id, s));
    return map;
  }, [allSchedules]);

  // Schedules currently selected (in order: reference first, then others)
  const selectedSchedules = useMemo(
    () =>
      [...selectedIds]
        .map((id) => infoMap.get(id))
        .filter((s): s is ScheduleInfo => s !== undefined),
    [selectedIds, infoMap]
  );

  const handleToggle = (info: ScheduleInfo) => {
    const key = buildGroupKey(info);
    const id = info.schedule_id;

    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
        if (next.size === 0) {
          setLockedGroupKey(null);
          setReferenceId(null);
        } else if (referenceId === id) {
          setReferenceId(null);
        }
      } else {
        next.add(id);
        setLockedGroupKey(key);
      }
      return next;
    });
  };

  const handleCompare = () => {
    if (referenceId === null || selectedIds.size < 2) return;

    const otherIds = [...selectedIds].filter((id) => id !== referenceId);
    navigate(`/schedules/${referenceId}/compare/${otherIds.join(',')}`);
  };

  const canCompare = selectedIds.size >= 2 && referenceId !== null;

  if (isLoading) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4">
        <ErrorMessage title="Failed to load schedules" message={(error as Error).message} />
      </div>
    );
  }

  return (
    <PageContainer className="gap-8">
      <PageHeader
        title="Compare Schedules"
        description="Select two or more schedules from the same observatory and schedule period, then choose which one acts as the reference."
      />

      {groups.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-slate-700 bg-slate-900/50 px-6 py-16 text-center text-slate-400">
          No schedules found. Import a schedule from the home page to get started.
        </div>
      ) : (
        <div className="grid gap-5 lg:grid-cols-2 xl:grid-cols-3">
          {groups.map((group) => (
            <GroupCard
              key={group.key}
              group={group}
              selectedIds={selectedIds}
              lockedGroupKey={lockedGroupKey}
              onToggle={handleToggle}
            />
          ))}
        </div>
      )}

      {/* Selection summary & action bar — shown when ≥1 schedule is selected */}
      {selectedIds.size > 0 && (
        <div className="sticky bottom-4 z-20 rounded-2xl border border-slate-700 bg-slate-900/95 p-5 shadow-2xl backdrop-blur">
          <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
            {/* Reference picker */}
            <div className="flex-1">
              <label
                htmlFor="advanced-reference"
                className="mb-2 block text-xs font-semibold uppercase tracking-[0.22em] text-slate-400"
              >
                Reference schedule
              </label>
              <select
                id="advanced-reference"
                value={referenceId ?? ''}
                onChange={(e) => setReferenceId(Number(e.target.value) || null)}
                className="w-full max-w-xs rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white outline-none transition focus:border-sky-500"
              >
                <option value="">— pick a reference —</option>
                {selectedSchedules.map((s) => (
                  <option key={s.schedule_id} value={s.schedule_id}>
                    {s.schedule_name}
                  </option>
                ))}
              </select>
            </div>

            {/* Selection chips + Compare button */}
            <div className="flex flex-col items-start gap-3 sm:items-end">
              <div className="flex flex-wrap gap-2">
                {selectedSchedules.map((s) => (
                  <span
                    key={s.schedule_id}
                    className={`flex items-center gap-1.5 rounded-full border px-3 py-1 text-sm ${
                      s.schedule_id === referenceId
                        ? 'border-sky-500/50 bg-sky-500/15 text-sky-200'
                        : 'border-slate-600 bg-slate-800 text-slate-200'
                    }`}
                  >
                    {s.schedule_id === referenceId && (
                      <span className="text-[10px] font-semibold uppercase tracking-wider text-sky-400">
                        ref
                      </span>
                    )}
                    {s.schedule_name}
                    <button
                      type="button"
                      onClick={() => handleToggle(s)}
                      className="ml-0.5 text-slate-400 hover:text-red-400"
                      aria-label={`Remove ${s.schedule_name}`}
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>

              <button
                type="button"
                onClick={handleCompare}
                disabled={!canCompare}
                className="rounded-xl bg-sky-600 px-6 py-2.5 text-sm font-semibold text-white shadow transition hover:bg-sky-500 disabled:cursor-not-allowed disabled:opacity-40"
              >
                {canCompare
                  ? `Compare ${selectedIds.size} schedules →`
                  : referenceId === null
                    ? 'Pick a reference to continue'
                    : 'Select at least 2 schedules'}
              </button>
            </div>
          </div>
        </div>
      )}
    </PageContainer>
  );
}

export default AdvancedPage;
