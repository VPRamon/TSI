import { useQueries } from '@tanstack/react-query';
import { api } from '@/api';
import { queryKeys } from '@/hooks/useApi';
import type { FragmentationData, InsightsData } from '@/api/types';

export interface ScheduleAnalysisData {
  id: number;
  name: string;
  insights: InsightsData | undefined;
  fragmentation: FragmentationData | undefined;
  isLoading: boolean;
  error: Error | null;
}

export function useScheduleAnalysisData(
  ids: number[],
  scheduleNames?: ReadonlyMap<number, string>
): ScheduleAnalysisData[] {
  const insightQueries = useQueries({
    queries: ids.map((id) => ({
      queryKey: queryKeys.insights(id),
      queryFn: () => api.getInsights(id),
      enabled: id > 0,
    })),
  });

  const fragmentationQueries = useQueries({
    queries: ids.map((id) => ({
      queryKey: queryKeys.fragmentation(id),
      queryFn: () => api.getFragmentation(id),
      enabled: id > 0,
    })),
  });

  return ids.map((id, idx) => {
    const insights = insightQueries[idx]?.data;
    const fragmentation = fragmentationQueries[idx]?.data;
    const fallbackName = scheduleNames?.get(id) ?? `Schedule #${id}`;

    return {
      id,
      name: fragmentation?.schedule_name ?? fallbackName,
      insights,
      fragmentation,
      isLoading:
        Boolean(insightQueries[idx]?.isLoading) || Boolean(fragmentationQueries[idx]?.isLoading),
      error:
        (insightQueries[idx]?.error as Error | null | undefined) ??
        (fragmentationQueries[idx]?.error as Error | null | undefined) ??
        null,
    };
  });
}
