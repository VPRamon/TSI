/**
 * Advanced page — multi-schedule comparison launcher.
 *
 * Groups available schedules by (observatory × schedule-period).
 * The user selects ≥ 2 schedules within one group, designates a reference,
 * and is redirected to /schedules/:refId/compare/:otherIds.
 *
 * Hard constraint: once a schedule from a group is checked, only schedules
 * from that same group can be checked.
 *
 * An upload section allows importing multiple schedule files at once
 * (file picker with multiple-select or drag-and-drop). Files are uploaded
 * sequentially; once all finish the new schedules appear in the group list.
 */
import { useMemo, useState, useRef, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useSchedules, useCreateSchedule } from '@/hooks';
import type { ScheduleInfo } from '@/api/types';
import { ErrorMessage, LoadingSpinner, LogStream, PageContainer, PageHeader } from '@/components';
import { OBSERVATORY_SITES, SITE_FROM_FILE, formatSiteLabel } from '@/constants';

// ---------------------------------------------------------------------------
// Upload section — types & state
// ---------------------------------------------------------------------------

type UploadStatus = 'pending' | 'uploading' | 'done' | 'error';

interface FileEntry {
  id: string;
  file: File;
  name: string;
  status: UploadStatus;
  jobId: string | null;
  error: string | null;
}

// ---------------------------------------------------------------------------
// UploadSection component
// ---------------------------------------------------------------------------

function UploadSection() {
  const createSchedule = useCreateSchedule();
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Use a ref to always access the latest entries inside async callbacks.
  const entriesRef = useRef<FileEntry[]>([]);
  const [entries, setEntriesState] = useState<FileEntry[]>([]);
  const setEntries = useCallback((updater: (prev: FileEntry[]) => FileEntry[]) => {
    setEntriesState((prev) => {
      const next = updater(prev);
      entriesRef.current = next;
      return next;
    });
  }, []);

  const [siteIdx, setSiteIdx] = useState('0');
  const siteIdxRef = useRef(siteIdx);
  siteIdxRef.current = siteIdx;

  const [isDragOver, setIsDragOver] = useState(false);

  // Derived: any file is currently uploading.
  const isRunning = entries.some((e) => e.status === 'uploading');

  const addFiles = useCallback(
    (fileList: FileList | File[]) => {
      const newEntries: FileEntry[] = Array.from(fileList)
        .filter((f) => f.name.toLowerCase().endsWith('.json'))
        .map((f) => ({
          id: `${f.name}-${Date.now()}-${Math.random()}`,
          file: f,
          name: f.name.replace(/\.json$/i, ''),
          status: 'pending' as UploadStatus,
          jobId: null,
          error: null,
        }));
      if (newEntries.length > 0) {
        setEntries((prev) => [...prev, ...newEntries]);
      }
    },
    [setEntries]
  );

  const removeEntry = useCallback(
    (id: string) => {
      setEntries((prev) => prev.filter((e) => e.id !== id));
    },
    [setEntries]
  );

  const updateName = useCallback(
    (id: string, name: string) => {
      setEntries((prev) => prev.map((e) => (e.id === id ? { ...e, name } : e)));
    },
    [setEntries]
  );

  // Upload a single file by index; runs concurrently with other doUpload calls.
  const doUpload = useCallback(
    async (idx: number) => {
      const entry = entriesRef.current[idx];
      if (!entry) return;

      setEntries((prev) => prev.map((e, i) => (i === idx ? { ...e, status: 'uploading' } : e)));

      try {
        const content = await entry.file.text();
        const scheduleJson: unknown = JSON.parse(content);
        const loc =
          siteIdxRef.current !== SITE_FROM_FILE
            ? OBSERVATORY_SITES[parseInt(siteIdxRef.current, 10)]?.location
            : undefined;

        const resp = await createSchedule.mutateAsync({
          name: entry.name.trim() || entry.file.name.replace(/\.json$/i, ''),
          schedule_json: scheduleJson,
          populate_analytics: true,
          location_override: loc,
        });

        setEntries((prev) =>
          prev.map((e, i) => (i === idx ? { ...e, jobId: resp.job_id } : e))
        );
        // Status transitions to 'done' / 'error' via LogStream onComplete / onError.
      } catch (err) {
        const msg = err instanceof Error ? err.message : 'Upload failed';
        setEntries((prev) =>
          prev.map((e, i) => (i === idx ? { ...e, status: 'error', error: msg } : e))
        );
      }
    },
    [createSchedule, setEntries]
  );

  // Fire all pending uploads simultaneously.
  const startAll = useCallback(() => {
    const pending = entriesRef.current
      .map((e, i) => ({ e, i }))
      .filter(({ e }) => e.status === 'pending');
    pending.forEach(({ i }) => doUpload(i));
  }, [doUpload]);

  const pendingCount = entries.filter((e) => e.status === 'pending').length;
  const doneCount = entries.filter((e) => e.status === 'done').length;
  const allDone = entries.length > 0 && entries.every((e) => e.status === 'done' || e.status === 'error');

  return (
    <section className="rounded-2xl border border-slate-700 bg-slate-900/60 p-5">
      <h2 className="mb-4 flex items-center gap-2 text-sm font-semibold uppercase tracking-[0.22em] text-slate-400">
        <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
          />
        </svg>
        Import schedules
      </h2>

      {/* Drop zone */}
      <div
        onDragOver={(e) => {
          e.preventDefault();
          setIsDragOver(true);
        }}
        onDragLeave={() => setIsDragOver(false)}
        onDrop={(e) => {
          e.preventDefault();
          setIsDragOver(false);
          addFiles(e.dataTransfer.files);
        }}
        onClick={() => fileInputRef.current?.click()}
        className={`flex cursor-pointer flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed px-6 py-8 text-center transition-all ${
          isDragOver
            ? 'border-sky-400 bg-sky-500/10 text-sky-300'
            : 'border-slate-600 text-slate-400 hover:border-sky-500/50 hover:bg-slate-800/40 hover:text-slate-300'
        }`}
        role="button"
        tabIndex={0}
        aria-label="Drop JSON files here or click to browse"
        onKeyDown={(e) => e.key === 'Enter' && fileInputRef.current?.click()}
      >
        <svg className="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
          />
        </svg>
        <p className="text-sm font-medium">
          {isDragOver ? 'Drop JSON files here' : 'Drop JSON files here or click to browse'}
        </p>
        <p className="text-xs opacity-70">Multiple files supported</p>
      </div>

      <input
        ref={fileInputRef}
        type="file"
        accept=".json,application/json"
        multiple
        className="sr-only"
        onChange={(e) => {
          if (e.target.files) addFiles(e.target.files);
          e.target.value = '';
        }}
      />

      {entries.length > 0 && (
        <>
          {/* Observatory site selector */}
          <div className="mt-4">
            <label
              htmlFor="upload-site"
              className="mb-1.5 block text-xs font-medium text-slate-400"
            >
              Observatory site (applied to all files)
            </label>
            <select
              id="upload-site"
              value={siteIdx}
              onChange={(e) => setSiteIdx(e.target.value)}
              disabled={isRunning}
              className="w-full max-w-xs rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white outline-none transition focus:border-sky-500 disabled:opacity-50"
            >
              {OBSERVATORY_SITES.map((site, i) => (
                <option key={i} value={String(i)}>
                  {formatSiteLabel(site)}
                </option>
              ))}
              <option value={SITE_FROM_FILE}>Use location from file</option>
            </select>
          </div>

          {/* File queue */}
          <ul className="mt-4 space-y-2">
            {entries.map((entry, idx) => (
              <li
                key={entry.id}
                className="rounded-xl border border-slate-700/60 bg-slate-800/40 px-4 py-3"
              >
                <div className="flex items-center gap-3">
                  {/* Status icon */}
                  <span className="shrink-0">
                    {entry.status === 'pending' && (
                      <span className="h-4 w-4 rounded-full border-2 border-slate-600 bg-transparent inline-block" />
                    )}
                    {entry.status === 'uploading' && <LoadingSpinner size="sm" />}
                    {entry.status === 'done' && (
                      <svg
                        className="h-4 w-4 text-emerald-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2.5}
                          d="M5 13l4 4L19 7"
                        />
                      </svg>
                    )}
                    {entry.status === 'error' && (
                      <svg
                        className="h-4 w-4 text-red-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2.5}
                          d="M6 18L18 6M6 6l12 12"
                        />
                      </svg>
                    )}
                  </span>

                  {/* Editable name */}
                  <input
                    type="text"
                    value={entry.name}
                    onChange={(e) => updateName(entry.id, e.target.value)}
                    disabled={entry.status !== 'pending'}
                    placeholder={entry.file.name.replace(/\.json$/i, '')}
                    className="min-w-0 flex-1 rounded-lg border border-transparent bg-transparent px-2 py-0.5 text-sm text-white placeholder-slate-500 outline-none transition focus:border-slate-600 focus:bg-slate-900 disabled:opacity-60"
                  />

                  {/* Filename hint */}
                  <span className="hidden shrink-0 text-xs text-slate-500 sm:inline">
                    {entry.file.name}
                  </span>

                  {/* Remove button (only when not actively uploading) */}
                  {entry.status !== 'uploading' && (
                    <button
                      type="button"
                      onClick={() => removeEntry(entry.id)}
                      className="shrink-0 text-slate-500 hover:text-red-400 transition-colors"
                      aria-label={`Remove ${entry.file.name}`}
                    >
                      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M6 18L18 6M6 6l12 12"
                        />
                      </svg>
                    </button>
                  )}
                </div>

                {/* Error message */}
                {entry.status === 'error' && entry.error && (
                  <p className="mt-1.5 pl-7 text-xs text-red-400">{entry.error}</p>
                )}

                {/* Live log stream — shown for every actively uploading file */}
                {entry.status === 'uploading' && entry.jobId && (
                  <div className="mt-2 pl-7">
                    <LogStream
                      jobId={entry.jobId}
                      apiBaseUrl="/api"
                      maxHeight="160px"
                      onComplete={() => {
                        setEntries((prev) =>
                          prev.map((e, i) => (i === idx ? { ...e, status: 'done', jobId: null } : e))
                        );
                      }}
                      onError={(err) => {
                        setEntries((prev) =>
                          prev.map((e, i) =>
                            i === idx ? { ...e, status: 'error', error: err, jobId: null } : e
                          )
                        );
                      }}
                    />
                  </div>
                )}
              </li>
            ))}
          </ul>

          {/* Action row */}
          <div className="mt-4 flex items-center justify-between gap-4">
            {allDone ? (
              <p className="text-sm text-emerald-400">
                {doneCount} schedule{doneCount !== 1 ? 's' : ''} imported — they now appear in the
                groups below.
              </p>
            ) : (
              <p className="text-sm text-slate-400">
                {pendingCount} file{pendingCount !== 1 ? 's' : ''} ready to upload
              </p>
            )}

            <div className="flex items-center gap-2">
              {!isRunning && (
                <button
                  type="button"
                  onClick={() => setEntries(() => [])}
                  className="rounded-lg px-3 py-1.5 text-sm text-slate-400 hover:text-slate-200 transition-colors"
                >
                  Clear all
                </button>
              )}
              <button
                type="button"
                onClick={startAll}
                disabled={isRunning || pendingCount === 0}
                className="rounded-xl bg-sky-600 px-5 py-2 text-sm font-semibold text-white shadow transition hover:bg-sky-500 disabled:cursor-not-allowed disabled:opacity-40"
              >
                {isRunning ? (
                  <span className="flex items-center gap-2">
                    <LoadingSpinner size="sm" />
                    Uploading…
                  </span>
                ) : (
                  `Upload ${pendingCount} file${pendingCount !== 1 ? 's' : ''}`
                )}
              </button>
            </div>
          </div>
        </>
      )}
    </section>
  );
}



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

      {/* Upload section — always shown so users can import new files */}
      <UploadSection />

      {groups.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-slate-700 bg-slate-900/50 px-6 py-16 text-center text-slate-400">
          No schedules in the database yet. Use the import section above to get started.
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
