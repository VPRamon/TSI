/**
 * Algorithm Analysis — environment-scoped, algorithm-agnostic shell.
 *
 * Always nested under an environment so the schedule selection comes
 * from the environment members instead of a free picker. The active
 * algorithm is auto-detected from the members'
 * `schedule_metadata.algorithm` value; when several algorithms are
 * present in the environment a small switcher is shown.
 *
 * The right pane is provided by the matching `TsiAlgorithm` registered
 * via the extension API; tabs from that registration are mounted under
 * `/environments/:envId/algorithm/:algoId/:tabId`. Tab components consume
 * the schedule selection via {@link useAlgorithm} so they don't need any
 * props.
 */
import { createContext, Suspense, useContext, useMemo } from 'react';
import { Link, Navigate, useNavigate, useParams } from 'react-router-dom';
import {
  ErrorMessage,
  LoadingSpinner,
  PageContainer,
  PageHeader,
} from '@/components';
import { useEnvironment, useSchedules } from '@/hooks';
import type { ScheduleInfo } from '@/api/types';
import { extensions } from '@/extensions';
import type { TsiAlgorithm } from '@/extensions';
import { formatStructureSummary } from '@/features/environments';

// ---------------------------------------------------------------------------
// Context shared with algorithm-specific tab components
// ---------------------------------------------------------------------------

export interface AlgorithmContextValue {
  /** Active algorithm (matched from `schedule_metadata.algorithm`). */
  algorithm: TsiAlgorithm;
  /** Member schedules of the environment that match the active algorithm. */
  selectedSchedules: ScheduleInfo[];
  /** Owning environment id (for deep-links from inside tabs). */
  environmentId: number;
}

const AlgorithmCtx = createContext<AlgorithmContextValue | null>(null);

/** Read the active algorithm and selected schedules from inside a tab. */
export function useAlgorithm(): AlgorithmContextValue {
  const value = useContext(AlgorithmCtx);
  if (!value) {
    throw new Error('useAlgorithm must be used inside <AlgorithmAnalysisPage>');
  }
  return value;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function scheduleAlgorithm(s: ScheduleInfo): string {
  return s.schedule_metadata?.algorithm?.trim() || 'unknown';
}

function findAlgorithm(algorithms: TsiAlgorithm[], id: string | undefined) {
  if (!id) return undefined;
  return algorithms.find((a) => a.id === id);
}

// ---------------------------------------------------------------------------
// Main page
// ---------------------------------------------------------------------------

export default function AlgorithmAnalysisPage() {
  const algorithms = useMemo(() => extensions.algorithms ?? [], []);
  const { envId: envIdParam, algoId: algoParam, tabId: tabParam } = useParams<{
    envId: string;
    algoId?: string;
    tabId?: string;
  }>();
  const navigate = useNavigate();

  const envId = useMemo(() => Number.parseInt(envIdParam ?? '0', 10), [envIdParam]);
  const validEnvId = Number.isFinite(envId) && envId > 0;

  const {
    data: environment,
    isLoading: envLoading,
    error: envError,
  } = useEnvironment(validEnvId ? envId : 0);

  const { data: schedulesResp, isLoading: schedulesLoading } = useSchedules();
  const allSchedules = useMemo(() => schedulesResp?.schedules ?? [], [schedulesResp]);

  const memberSchedules = useMemo(() => {
    if (!environment) return [] as ScheduleInfo[];
    const memberIds = new Set(environment.schedule_ids);
    return allSchedules.filter((s) => memberIds.has(s.schedule_id));
  }, [environment, allSchedules]);

  const knownAlgorithmIds = useMemo(() => new Set(algorithms.map((a) => a.id)), [algorithms]);
  const candidateSchedules = useMemo(
    () => memberSchedules.filter((s) => knownAlgorithmIds.has(scheduleAlgorithm(s))),
    [memberSchedules, knownAlgorithmIds],
  );

  const presentAlgoIds = useMemo(() => {
    const ids = new Set<string>();
    for (const s of candidateSchedules) ids.add(scheduleAlgorithm(s));
    return ids;
  }, [candidateSchedules]);

  // ---------------- early returns ----------------
  if (!validEnvId) {
    return (
      <PageContainer>
        <ErrorMessage message="Invalid environment id" />
      </PageContainer>
    );
  }
  if (envLoading || schedulesLoading) {
    return (
      <PageContainer>
        <div className="flex justify-center py-8">
          <LoadingSpinner size="lg" />
        </div>
      </PageContainer>
    );
  }
  if (envError || !environment) {
    return (
      <PageContainer>
        <ErrorMessage
          message={`Error loading environment: ${envError?.message ?? 'not found'}`}
        />
      </PageContainer>
    );
  }

  const description = `${formatStructureSummary(environment.structure)} · ${
    environment.schedule_ids.length
  } ${environment.schedule_ids.length === 1 ? 'schedule' : 'schedules'}`;

  if (algorithms.length === 0) {
    return (
      <PageContainer>
        <PageHeader title={environment.name} description={description} />
        <div className="rounded-lg border border-dashed border-slate-600 py-16 text-center text-sm text-slate-400">
          No algorithm extensions are currently registered.
        </div>
      </PageContainer>
    );
  }
  if (candidateSchedules.length === 0) {
    return (
      <PageContainer>
        <PageHeader title={environment.name} description={description} />
        <div className="rounded-lg border border-dashed border-slate-600 py-16 text-center text-sm text-slate-400">
          None of this environment&apos;s schedules carry a recognised algorithm
          trace. Re-upload a schedule alongside its <code>*.&lt;algorithm&gt;_trace.jsonl</code>
          file to enable algorithm analysis.
        </div>
      </PageContainer>
    );
  }

  // ---------------- pick algorithm + tab ----------------
  const explicitAlgo = findAlgorithm(algorithms, algoParam);
  const autoAlgo =
    presentAlgoIds.size === 1
      ? findAlgorithm(algorithms, [...presentAlgoIds][0])
      : undefined;
  const activeAlgo =
    explicitAlgo ??
    autoAlgo ??
    algorithms.find((a) => presentAlgoIds.has(a.id)) ??
    algorithms[0];

  const activeTabId = tabParam ?? activeAlgo.tabs[0]?.id;
  const activeTab = activeAlgo.tabs.find((t) => t.id === activeTabId) ?? activeAlgo.tabs[0];

  if (
    activeTab &&
    (!algoParam ||
      !tabParam ||
      algoParam !== activeAlgo.id ||
      tabParam !== activeTab.id)
  ) {
    return (
      <Navigate
        to={`/environments/${envId}/algorithm/${activeAlgo.id}/${activeTab.id}`}
        replace
      />
    );
  }

  // ---------------- switcher + tabs ----------------
  const showSwitcher = algorithms.length > 1 || presentAlgoIds.size > 1;
  const algoSwitcher = showSwitcher && (
    <div className="flex flex-wrap items-center gap-2">
      <span className="text-xs uppercase tracking-wider text-slate-500">Algorithm</span>
      {algorithms
        .filter((a) => presentAlgoIds.has(a.id))
        .map((a) => {
          const active = a.id === activeAlgo.id;
          return (
            <button
              key={a.id}
              type="button"
              onClick={() =>
                navigate(
                  `/environments/${envId}/algorithm/${a.id}/${a.tabs[0]?.id ?? ''}`,
                )
              }
              className={`rounded border px-3 py-1 text-xs transition ${
                active
                  ? 'border-primary-500 bg-primary-600/30 text-white'
                  : 'border-slate-600 bg-slate-800 text-slate-300 hover:bg-slate-700'
              }`}
            >
              {a.label}
            </button>
          );
        })}
    </div>
  );

  const tabBar = (
    <div className="flex flex-wrap items-center gap-2 border-b border-slate-700 pb-2">
      {activeAlgo.tabs.map((t) => (
        <Link
          key={t.id}
          to={`/environments/${envId}/algorithm/${activeAlgo.id}/${t.id}`}
          className={`rounded-t px-3 py-1.5 text-sm transition ${
            t.id === activeTab?.id
              ? 'bg-primary-600 text-white'
              : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
          }`}
        >
          {t.label}
        </Link>
      ))}
      <div className="ml-auto text-xs text-slate-500">
        {candidateSchedules.filter((s) => scheduleAlgorithm(s) === activeAlgo.id).length}{' '}
        schedule(s) for {activeAlgo.label}
      </div>
    </div>
  );

  const matching = candidateSchedules.filter(
    (s) => scheduleAlgorithm(s) === activeAlgo.id,
  );
  const TabComponent = activeTab?.component;
  const tabBody = !TabComponent ? (
    <div className="rounded-lg border border-dashed border-slate-600 py-16 text-center text-sm text-slate-400">
      This algorithm has no panels registered.
    </div>
  ) : matching.length === 0 ? (
    <div className="rounded-lg border border-dashed border-slate-600 py-16 text-center text-sm text-slate-400">
      None of this environment&apos;s schedules were produced by {activeAlgo.label}.
    </div>
  ) : (
    <Suspense
      fallback={
        <div className="flex items-center justify-center py-16">
          <LoadingSpinner />
        </div>
      }
    >
      <TabComponent />
    </Suspense>
  );

  const ctxValue: AlgorithmContextValue = {
    algorithm: activeAlgo,
    selectedSchedules: matching,
    environmentId: envId,
  };

  return (
    <PageContainer>
      <PageHeader
        title={`${environment.name} — Algorithm Analysis`}
        description={description}
        actions={
          <div className="flex items-center gap-3">
            <Link
              to={`/environments/${envId}/compare`}
              className="rounded-lg border border-slate-600 px-3 py-1.5 text-xs text-slate-200 hover:bg-slate-800"
            >
              Compare ↗
            </Link>
            <Link
              to="/workspace"
              className="text-xs text-slate-400 hover:text-slate-200"
            >
              ← Workspace
            </Link>
            {algoSwitcher || null}
          </div>
        }
      />

      <div className="space-y-5">
        {tabBar}
        <AlgorithmCtx.Provider value={ctxValue}>{tabBody}</AlgorithmCtx.Provider>
      </div>
    </PageContainer>
  );
}
