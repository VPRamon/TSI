/**
 * One environment rendered as a card on the /workspace page.
 */
import { useNavigate } from 'react-router-dom';
import { useSchedules } from '@/hooks';
import type { EnvironmentInfo } from '@/api/types';
import { formatCreatedAt, formatStructureSummary } from '../utils';

interface EnvironmentCardProps {
  environment: EnvironmentInfo;
  onAddSchedules: () => void;
  onRemoveSchedule: (scheduleId: number) => void;
  onDelete: () => void;
  isMutating: boolean;
}

export function EnvironmentCard({
  environment,
  onAddSchedules,
  onRemoveSchedule,
  onDelete,
  isMutating,
}: EnvironmentCardProps) {
  const navigate = useNavigate();
  const { data: schedulesData } = useSchedules();
  const nameById = new Map<number, string>();
  const algorithmById = new Map<number, string>();
  schedulesData?.schedules.forEach((s) => {
    nameById.set(s.schedule_id, s.schedule_name);
    const algo = s.schedule_metadata?.algorithm?.trim();
    if (algo) algorithmById.set(s.schedule_id, algo);
  });

  const memberCount = environment.schedule_ids.length;
  const canCompare = memberCount >= 2;
  const hasAlgorithmTrace = environment.schedule_ids.some((id) => {
    const algo = algorithmById.get(id);
    return algo && algo !== 'unknown';
  });

  return (
    <section
      className="flex flex-col gap-4 rounded-2xl border border-slate-700 bg-slate-900/60 p-5"
      data-testid="environment-card"
    >
      <header className="flex flex-col gap-1">
        <div className="flex items-center justify-between gap-3">
          <h3 className="truncate text-base font-semibold text-white" title={environment.name}>
            {environment.name}
          </h3>
          <span className="shrink-0 rounded-full border border-slate-600 px-2 py-0.5 text-xs text-slate-300">
            {memberCount} schedule{memberCount === 1 ? '' : 's'}
          </span>
        </div>
        <p className="text-xs text-slate-500">Created {formatCreatedAt(environment.created_at)}</p>
        <p className="text-xs text-slate-400">{formatStructureSummary(environment.structure)}</p>
      </header>

      {memberCount > 0 && (
        <ul className="space-y-1.5">
          {environment.schedule_ids.map((id) => (
            <li
              key={id}
              className="flex items-center justify-between gap-2 rounded-lg border border-slate-700/60 bg-slate-800/40 px-3 py-1.5 text-sm"
            >
              <span className="truncate text-slate-200">
                {nameById.get(id) ?? `Schedule #${id}`}
              </span>
              <button
                type="button"
                onClick={() => onRemoveSchedule(id)}
                disabled={isMutating}
                className="text-xs text-slate-400 hover:text-red-400 disabled:opacity-40"
                aria-label={`Remove schedule ${nameById.get(id) ?? id} from environment`}
              >
                Unassign
              </button>
            </li>
          ))}
        </ul>
      )}

      <div className="mt-auto flex flex-wrap items-center justify-between gap-2">
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={onAddSchedules}
            disabled={isMutating}
            className="rounded-lg border border-slate-600 px-3 py-1.5 text-xs text-slate-200 hover:bg-slate-800 disabled:opacity-40"
          >
            Add schedules
          </button>
          <button
            type="button"
            onClick={() => navigate(`/environments/${environment.environment_id}/compare`)}
            disabled={!canCompare}
            className="rounded-lg bg-sky-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-sky-500 disabled:cursor-not-allowed disabled:opacity-40"
            title={canCompare ? 'Open compare view' : 'Need at least 2 schedules to compare'}
          >
            Open compare
          </button>
          <button
            type="button"
            onClick={() => navigate(`/environments/${environment.environment_id}/algorithm`)}
            disabled={!hasAlgorithmTrace}
            className="rounded-lg border border-violet-500/60 bg-violet-600/30 px-3 py-1.5 text-xs font-semibold text-violet-100 hover:bg-violet-600/50 disabled:cursor-not-allowed disabled:opacity-40"
            title={
              hasAlgorithmTrace
                ? 'Open algorithm-specific dashboards'
                : 'No member schedule has an algorithm trace yet'
            }
          >
            Algorithm analysis →
          </button>
        </div>
        <button
          type="button"
          onClick={onDelete}
          disabled={isMutating}
          className="text-xs text-slate-500 hover:text-red-400 disabled:opacity-40"
        >
          Delete env
        </button>
      </div>
    </section>
  );
}
