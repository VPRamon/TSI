/**
 * Compare page — multi-schedule field table with reference baseline.
 *
 * Route: /schedules/:scheduleId/compare/:otherIds
 */
import { useMemo, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useSchedules } from '@/hooks';
import {
  BlockStatusTable,
  SchedulePicker,
  SummaryTable,
  useScheduleAnalysisData,
} from '@/features/schedules';
import { ErrorMessage, LoadingSpinner, PageContainer, PageHeader } from '@/components';
import type { ScheduleInfo } from '@/api/types';

function ScheduleChips({
  comparisonIds,
  refId,
  scheduleInfoMap,
  onRemove,
  onAdd,
}: {
  comparisonIds: number[];
  refId: number;
  scheduleInfoMap: Map<number, ScheduleInfo>;
  onRemove: (id: number) => void;
  onAdd: (schedules: ScheduleInfo[]) => void;
}) {
  const [showPicker, setShowPicker] = useState(false);

  return (
    <div className="flex flex-wrap items-center gap-2">
      {comparisonIds.map((id) => {
        const info = scheduleInfoMap.get(id);
        const label = info?.schedule_name ?? `#${id}`;

        return (
          <span
            key={id}
            className="flex items-center gap-1 rounded-full border border-slate-600 bg-slate-700 px-3 py-1 text-sm text-white"
          >
            {label}
            <button
              type="button"
              onClick={() => onRemove(id)}
              className="ml-1 text-slate-400 hover:text-red-400"
              aria-label={`Remove ${label}`}
            >
              ×
            </button>
          </span>
        );
      })}

      <div className="relative">
        <button
          type="button"
          onClick={() => setShowPicker((value) => !value)}
          className="flex items-center gap-1 rounded-full border border-dashed border-slate-600 px-3 py-1 text-sm text-slate-400 hover:border-slate-400 hover:text-slate-300"
        >
          + Add schedule
        </button>
        {showPicker ? (
          <div className="absolute left-0 top-full z-50 mt-2 w-72">
            <SchedulePicker
              multiSelect
              excludeId={refId}
              initialSelectedIds={comparisonIds}
              placeholder="Search schedules to compare..."
              onConfirm={(schedules) => {
                setShowPicker(false);
                onAdd(schedules);
              }}
            />
          </div>
        ) : null}
      </div>
    </div>
  );
}

function ComparePage() {
  const { scheduleId: scheduleIdParam, otherIds: otherIdsParam } = useParams<{
    scheduleId: string;
    otherIds?: string;
  }>();
  const navigate = useNavigate();
  const [showEmptyPicker, setShowEmptyPicker] = useState(false);

  const refId = useMemo(() => Number.parseInt(scheduleIdParam ?? '0', 10), [scheduleIdParam]);

  const comparisonIds = useMemo(() => {
    const seen = new Set<number>();
    const ids: number[] = [];

    for (const part of (otherIdsParam ?? '').split(',')) {
      const value = Number.parseInt(part.trim(), 10);
      if (!Number.isFinite(value) || value <= 0 || value === refId || seen.has(value)) {
        continue;
      }
      seen.add(value);
      ids.push(value);
    }

    return ids;
  }, [otherIdsParam, refId]);

  const orderedIds = useMemo(
    () => (refId > 0 ? [refId, ...comparisonIds] : comparisonIds),
    [comparisonIds, refId]
  );

  const { data: schedulesData } = useSchedules();
  const scheduleInfoMap = useMemo(() => {
    const map = new Map<number, ScheduleInfo>();
    schedulesData?.schedules.forEach((schedule) => {
      map.set(schedule.schedule_id, schedule);
    });
    return map;
  }, [schedulesData]);
  const scheduleNames = useMemo(() => {
    const map = new Map<number, string>();
    schedulesData?.schedules.forEach((schedule) => {
      map.set(schedule.schedule_id, schedule.schedule_name);
    });
    return map;
  }, [schedulesData]);

  const schedules = useScheduleAnalysisData(orderedIds, scheduleNames);

  const handleRemove = (id: number) => {
    const next = comparisonIds.filter((comparisonId) => comparisonId !== id);
    if (next.length === 0) {
      navigate(`/schedules/${refId}/compare`);
      return;
    }
    navigate(`/schedules/${refId}/compare/${next.join(',')}`);
  };

  const handleAdd = (selected: ScheduleInfo[]) => {
    const next = [
      ...new Set(selected.map((schedule) => schedule.schedule_id).filter((id) => id !== refId)),
    ];

    if (next.length === 0) {
      navigate(`/schedules/${refId}/compare`);
      return;
    }

    navigate(`/schedules/${refId}/compare/${next.join(',')}`);
  };

  const anyLoading = schedules.some((schedule) => schedule.isLoading);
  const anyError = schedules.find((schedule) => schedule.error);

  if (comparisonIds.length === 0) {
    return (
      <PageContainer>
        <PageHeader
          title="Compare Schedules"
          description="Select one or more schedules to compare against the loaded schedule."
        />
        <div className="flex flex-col items-center gap-4 py-12">
          <p className="text-slate-400">Add schedules to compare</p>
          <div className="w-80">
            {showEmptyPicker ? (
              <SchedulePicker
                multiSelect
                excludeId={refId}
                initialSelectedIds={comparisonIds}
                placeholder="Search schedules to compare..."
                onConfirm={(selected) => {
                  setShowEmptyPicker(false);
                  const next = [
                    ...new Set(
                      selected
                        .map((schedule) => schedule.schedule_id)
                        .filter((scheduleId) => scheduleId !== refId)
                    ),
                  ];
                  navigate(`/schedules/${refId}/compare/${next.join(',')}`);
                }}
              />
            ) : (
              <button
                type="button"
                onClick={() => setShowEmptyPicker(true)}
                className="w-full rounded-lg border border-dashed border-slate-600 px-4 py-3 text-sm text-slate-400 hover:border-slate-400 hover:text-slate-300"
              >
                + Select schedules to compare
              </button>
            )}
          </div>
        </div>
      </PageContainer>
    );
  }

  return (
    <PageContainer>
      <PageHeader
        title="Compare Schedules"
        description="The loaded schedule is used as the reference baseline."
      />

      <ScheduleChips
        comparisonIds={comparisonIds}
        refId={refId}
        scheduleInfoMap={scheduleInfoMap}
        onRemove={handleRemove}
        onAdd={handleAdd}
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
          <SummaryTable schedules={schedules} />
          <BlockStatusTable schedules={schedules} />
        </>
      )}
    </PageContainer>
  );
}

export default ComparePage;
