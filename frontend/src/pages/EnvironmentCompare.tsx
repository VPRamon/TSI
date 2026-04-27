/**
 * EnvironmentCompare page — multi-schedule comparison scoped to a single
 * environment. All members are peers (no reference baseline).
 *
 * Route: /environments/:envId/compare
 */
import { useMemo } from 'react';
import { Link, useParams } from 'react-router-dom';
import { useEnvironment, useEnvironmentKpis, useSchedules } from '@/hooks';
import {
  BlockStatusTable,
  ComparisonCharts,
  EnvironmentVerdict,
  KpiEvolutionChart,
  SummaryTable,
  useScheduleAnalysisData,
} from '@/features/schedules';
import { formatStructureSummary } from '@/features/environments';
import { ErrorMessage, LoadingSpinner, PageContainer, PageHeader } from '@/components';

function EnvironmentComparePage() {
  const { envId: envIdParam } = useParams<{ envId: string }>();
  const envId = useMemo(() => Number.parseInt(envIdParam ?? '0', 10), [envIdParam]);

  const {
    data: environment,
    isLoading: envLoading,
    error: envError,
  } = useEnvironment(Number.isFinite(envId) && envId > 0 ? envId : 0);

  const { data: schedulesData } = useSchedules();
  const scheduleInfoMap = useMemo(() => {
    const map = new Map<number, import('@/api/types').ScheduleInfo>();
    schedulesData?.schedules.forEach((schedule) => {
      map.set(schedule.schedule_id, schedule);
    });
    return map;
  }, [schedulesData]);

  const memberIds = useMemo(() => environment?.schedule_ids ?? [], [environment]);
  const schedules = useScheduleAnalysisData(memberIds, scheduleInfoMap);
  const { data: envKpis } = useEnvironmentKpis(Number.isFinite(envId) && envId > 0 ? envId : 0);

  const anyLoading = schedules.some((schedule) => schedule.isLoading);
  const anyError = schedules.find((schedule) => schedule.error);

  if (!Number.isFinite(envId) || envId <= 0) {
    return (
      <PageContainer>
        <ErrorMessage message="Invalid environment id" />
      </PageContainer>
    );
  }

  if (envLoading) {
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

  const description = `${formatStructureSummary(environment.structure)} · ${memberIds.length} ${
    memberIds.length === 1 ? 'schedule' : 'schedules'
  }`;

  if (memberIds.length < 2) {
    return (
      <PageContainer>
        <PageHeader title={environment.name} description={description} />
        <div className="rounded-lg border border-slate-700 bg-slate-800 p-8 text-center text-slate-400">
          This environment needs at least two schedules to compare. Add more schedules
          on the Advanced page.
        </div>
      </PageContainer>
    );
  }

  return (
    <PageContainer>
      <PageHeader
        title={environment.name}
        description={description}
        actions={
          <Link
            to={`/environments/${envId}/algorithm`}
            className="rounded-lg border border-violet-500/60 bg-violet-600/30 px-3 py-1.5 text-xs font-semibold text-violet-100 hover:bg-violet-600/50"
          >
            Algorithm analysis →
          </Link>
        }
      />

      {anyError ? (
        <ErrorMessage message={`Error loading data: ${anyError.error?.message}`} />
      ) : null}

      {anyLoading ? (
        <div className="flex justify-center py-8">
          <LoadingSpinner size="lg" />
        </div>
      ) : (
        <>
          {envKpis && envKpis.kpis.length >= 2 ? (
            <>
              <EnvironmentVerdict kpis={envKpis.kpis} />
              <KpiEvolutionChart kpis={envKpis.kpis} />
            </>
          ) : null}
          <SummaryTable schedules={schedules} />
          <ComparisonCharts schedules={schedules} />
          <BlockStatusTable schedules={schedules} />
        </>
      )}
    </PageContainer>
  );
}

export default EnvironmentComparePage;
