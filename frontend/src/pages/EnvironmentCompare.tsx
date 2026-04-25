/**
 * EnvironmentCompare page — multi-schedule comparison scoped to a single
 * environment. All members are peers (no reference baseline).
 *
 * Route: /environments/:envId/compare
 */
import { useMemo } from 'react';
import { useParams } from 'react-router-dom';
import { useEnvironment, useSchedules } from '@/hooks';
import {
  BlockStatusTable,
  ComparisonCharts,
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
  const scheduleNames = useMemo(() => {
    const map = new Map<number, string>();
    schedulesData?.schedules.forEach((schedule) => {
      map.set(schedule.schedule_id, schedule.schedule_name);
    });
    return map;
  }, [schedulesData]);

  const memberIds = useMemo(() => environment?.schedule_ids ?? [], [environment]);
  const schedules = useScheduleAnalysisData(memberIds, scheduleNames);

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
      <PageHeader title={environment.name} description={description} />

      {anyError ? (
        <ErrorMessage message={`Error loading data: ${anyError.error?.message}`} />
      ) : null}

      {anyLoading ? (
        <div className="flex justify-center py-8">
          <LoadingSpinner size="lg" />
        </div>
      ) : (
        <>
          <SummaryTable schedules={schedules} />
          <ComparisonCharts schedules={schedules} />
          <BlockStatusTable schedules={schedules} />
        </>
      )}
    </PageContainer>
  );
}

export default EnvironmentComparePage;
