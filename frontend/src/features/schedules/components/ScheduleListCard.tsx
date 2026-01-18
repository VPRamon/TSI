/**
 * Schedule list card component.
 * Displays list of existing schedules from the database.
 */
import type { ScheduleInfo } from '@/api/types';

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

export interface ScheduleListCardProps {
  /** List of schedules to display */
  schedules: ScheduleInfo[];
  /** Total count of schedules */
  total: number;
  /** Callback when a schedule is clicked */
  onScheduleClick: (scheduleId: number) => void;
}

function ScheduleListCard({ schedules, total, onScheduleClick }: ScheduleListCardProps) {
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
            <h2 className="mb-2 text-2xl font-semibold text-white">Load from Database</h2>
            <p className="text-sm leading-relaxed text-slate-400">
              Access previously uploaded observation schedules
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
                />
              ))}
            </div>
          )}
        </div>

        {/* Info Footer */}
        {!isEmpty && (
          <div className="border-t border-slate-700/50 pt-4">
            <p className="text-center text-xs text-slate-500">
              {total} {total === 1 ? 'schedule' : 'schedules'} available
            </p>
          </div>
        )}
      </div>
    </article>
  );
}

function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-12 text-center">
      <div
        className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-slate-700/30 text-3xl"
        aria-hidden="true"
      >
        📭
      </div>
      <p className="text-sm text-slate-400">No schedules yet</p>
      <p className="mt-1 text-xs text-slate-500">Upload one to get started</p>
    </div>
  );
}

interface ScheduleListItemProps {
  schedule: ScheduleInfo;
  onClick: () => void;
}

function ScheduleListItem({ schedule, onClick }: ScheduleListItemProps) {
  return (
    <button
      onClick={onClick}
      className="group/item flex w-full items-center justify-between rounded-lg border border-slate-700/30 bg-slate-900/40 p-4 text-left transition-all duration-200 hover:border-indigo-500/30 hover:bg-slate-900/70 focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
    >
      <div className="min-w-0 flex-1">
        <p className="truncate font-medium text-white transition-colors group-hover/item:text-indigo-300">
          {schedule.schedule_name}
        </p>
        <p className="mt-0.5 text-xs text-slate-500">ID: {schedule.schedule_id}</p>
      </div>
      <ChevronRightIcon />
    </button>
  );
}

export default ScheduleListCard;
