import { useQueries } from '@tanstack/react-query';
import { api } from '@/api';
import { queryKeys } from '@/hooks/useApi';
import type { FragmentationData, InsightsData, ScheduleInfo } from '@/api/types';

export interface ScheduleAnalysisData {
  id: number;
  name: string;
  insights: InsightsData | undefined;
  fragmentation: FragmentationData | undefined;
  isLoading: boolean;
  error: Error | null;
  /** Algorithm name from `schedule_metadata.algorithm`, when known. */
  algorithm?: string;
  /**
   * Algorithm configuration blob from `schedule_metadata.algorithm_config`,
   * when populated. Fed to {@link useConfigFilters} so the comparison
   * panels can filter / group by configuration knobs (e/k/b/…).
   */
  algorithmConfig?: Record<string, unknown>;
}

/**
 * Source of supplemental schedule metadata. Either a name lookup map
 * (legacy) or a richer ScheduleInfo lookup (preferred — surfaces the
 * algorithm config to consumers).
 */
type ScheduleLookup =
  | ReadonlyMap<number, string>
  | ReadonlyMap<number, ScheduleInfo>
  | undefined;

function resolveLookup(
  lookup: ScheduleLookup,
  id: number,
): { name: string | undefined; info: ScheduleInfo | undefined } {
  if (!lookup) return { name: undefined, info: undefined };
  const entry = lookup.get(id);
  if (entry == null) return { name: undefined, info: undefined };
  if (typeof entry === 'string') return { name: entry, info: undefined };
  return { name: entry.schedule_name, info: entry };
}

export function useScheduleAnalysisData(
  ids: number[],
  scheduleLookup?: ScheduleLookup
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
    const { name, info } = resolveLookup(scheduleLookup, id);
    const fallbackName = name ?? `Schedule #${id}`;

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
      algorithm: info?.schedule_metadata?.algorithm,
      algorithmConfig: info?.schedule_metadata?.algorithm_config,
    };
  });
}
