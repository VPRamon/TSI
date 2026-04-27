/**
 * Schedule list card component.
 * Displays list of existing schedules from the database.
 */
import type { ScheduleInfo } from '@/api/types';
import { Icon } from '@/components';

// SVG Icons
const DatabaseIcon = () => (
  <svg className="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.5}
      d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"
    />
  </svg>
);

const ChevronRightIcon = () => (
  <svg
    className="h-5 w-5 flex-shrink-0 text-slate-500 transition-all group-hover/item:translate-x-1 group-hover/item:text-indigo-400"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
    aria-hidden="true"
  >
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
  </svg>
);

const DownloadIcon = () => (
  <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.75}
      d="M12 4v10m0 0 4-4m-4 4-4-4M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2"
    />
  </svg>
);

const ManageIcon = () => (
  <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.75}
      d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
    />
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.75}
      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
    />
  </svg>
);

const SpinnerIcon = () => (
  <svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none" aria-hidden="true">
    <circle cx="12" cy="12" r="10" stroke="currentColor" strokeOpacity="0.25" strokeWidth="3" />
    <path
      d="M22 12a10 10 0 0 0-10-10"
      stroke="currentColor"
      strokeWidth="3"
      strokeLinecap="round"
    />
  </svg>
);

export interface ScheduleListCardProps {
  /** List of schedules to display */
  schedules: ScheduleInfo[];
  /** Total count of schedules */
  total: number;
  /** Callback when a schedule is clicked */
  onScheduleClick: (scheduleId: number) => void;
  /** Callback when a schedule JSON download is requested */
  onScheduleDownload: (schedule: ScheduleInfo) => void;
  /** Callback when schedule management page is requested */
  onManageSchedules: () => void;
  /** Callback when workspace is requested */
  onOpenWorkspace?: () => void;
  /** IDs currently downloading */
  downloadingScheduleIds?: ReadonlySet<number>;
}

function ScheduleListCard({
  schedules,
  total,
  onScheduleClick,
  onScheduleDownload,
  onManageSchedules,
  onOpenWorkspace,
  downloadingScheduleIds,
}: ScheduleListCardProps) {
  const isEmpty = schedules.length === 0;

  return (
    <article className="group relative rounded-2xl border border-slate-700/50 bg-slate-800/50 p-8 backdrop-blur-sm transition-all duration-300 focus-within:ring-2 focus-within:ring-indigo-500/50 hover:border-slate-600/50 hover:bg-slate-800/70 hover:shadow-xl hover:shadow-indigo-500/10">
      <div className="flex h-full flex-col">
        {/* Icon & Title */}
        <div className="mb-6 flex items-start gap-4">
          <div
            className="rounded-xl bg-indigo-500/10 p-3 text-indigo-400 transition-colors group-hover:bg-indigo-500/20"
            aria-hidden="true"
          >
            <DatabaseIcon />
          </div>
          <div className="flex-1">
            <h2 className="mb-2 text-2xl font-semibold text-white">Open from Database</h2>
            <p className="text-sm leading-relaxed text-slate-400">
              Access previously imported schedule datasets
            </p>
          </div>
        </div>

        {/* Schedule List */}
        <div className="mb-6 flex-1">
          {isEmpty ? (
            <EmptyState />
          ) : (
            <div className="scrollbar-thin max-h-64 space-y-2 overflow-y-auto pr-2">
              {schedules.map((schedule) => (
                <ScheduleListItem
                  key={schedule.schedule_id}
                  schedule={schedule}
                  onClick={() => onScheduleClick(schedule.schedule_id)}
                  onDownload={() => onScheduleDownload(schedule)}
                  isDownloading={downloadingScheduleIds?.has(schedule.schedule_id) ?? false}
                />
              ))}
            </div>
          )}
        </div>

        {/* Info Footer — always visible so navigation is reachable even on empty DB */}
        <div className="border-t border-slate-700/50 pt-4">
          <div className="flex flex-wrap items-center justify-between gap-3">
            {!isEmpty && <p className="text-xs text-slate-500">{total} available</p>}
            <div className="ml-auto flex items-center gap-2">
              {onOpenWorkspace ? (
                <button
                  type="button"
                  onClick={onOpenWorkspace}
                  className="inline-flex items-center gap-2 rounded-lg border border-sky-500/40 bg-sky-500/10 px-3 py-2 text-xs font-medium text-sky-200 transition-all duration-200 hover:border-sky-400/60 hover:bg-sky-500/20 hover:text-white focus:outline-none focus:ring-2 focus:ring-sky-500/50"
                  title="Open workspace"
                  aria-label="Open workspace"
                >
                  <DatabaseIcon />
                  <span>Workspace</span>
                </button>
              ) : null}
              <button
                type="button"
                onClick={onManageSchedules}
                className="inline-flex items-center gap-2 rounded-lg border border-slate-600/60 bg-slate-800/70 px-3 py-2 text-xs font-medium text-slate-200 transition-all duration-200 hover:border-slate-500/70 hover:bg-slate-700/80 hover:text-white focus:outline-none focus:ring-2 focus:ring-slate-500/50"
                title="Manage schedules"
                aria-label="Manage schedules"
              >
                <ManageIcon />
                <span>Manage Schedules</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </article>
  );
}

function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-12 text-center">
      <div
        className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-slate-700/30 text-slate-400"
        aria-hidden="true"
      >
        <Icon name="inbox" className="h-8 w-8" />
      </div>
      <p className="text-sm text-slate-400">No schedules yet</p>
      <p className="mt-1 text-xs text-slate-500">Upload one to get started</p>
    </div>
  );
}

interface ScheduleListItemProps {
  schedule: ScheduleInfo;
  onClick: () => void;
  onDownload: () => void;
  isDownloading: boolean;
}

function ScheduleListItem({ schedule, onClick, onDownload, isDownloading }: ScheduleListItemProps) {
  return (
    <div className="flex items-stretch gap-2">
      <button
        type="button"
        onClick={onClick}
        className="group/item flex min-h-[56px] min-w-0 flex-1 items-center justify-between rounded-lg border border-slate-700/30 bg-slate-900/40 p-4 text-left transition-all duration-200 hover:border-indigo-500/30 hover:bg-slate-900/70 focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
      >
        <div className="min-w-0 flex-1">
          <p className="truncate font-medium text-white transition-colors group-hover/item:text-indigo-300">
            {schedule.schedule_name}
          </p>
        </div>
        <ChevronRightIcon />
      </button>

      <button
        type="button"
        onClick={onDownload}
        disabled={isDownloading}
        className="inline-flex min-h-[56px] items-center gap-1.5 rounded-lg border border-slate-700/50 bg-slate-900/40 px-3 py-3 text-xs font-medium text-slate-300 transition-all duration-200 hover:border-emerald-500/50 hover:bg-slate-900/70 hover:text-emerald-300 focus:outline-none focus:ring-2 focus:ring-emerald-500/50 disabled:cursor-not-allowed disabled:opacity-60"
        title={isDownloading ? 'Downloading JSON...' : 'Download JSON'}
        aria-label={
          isDownloading
            ? `Downloading ${schedule.schedule_name} JSON`
            : `Download ${schedule.schedule_name} as JSON`
        }
      >
        {isDownloading ? <SpinnerIcon /> : <DownloadIcon />}
        <span>JSON</span>
      </button>
    </div>
  );
}

export default ScheduleListCard;
