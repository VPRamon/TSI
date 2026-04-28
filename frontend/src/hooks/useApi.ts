/**
 * React Query hooks for the TSI API.
 */
import { useQuery, useMutation, useQueryClient, keepPreviousData } from '@tanstack/react-query';
import type { AxiosProgressEvent } from 'axios';
import { api } from '@/api';
import type {
  CreateScheduleRequest,
  TrendsQuery,
  CompareQuery,
  VisibilityHistogramQuery,
  UpdateScheduleRequest,
  AltAzRequest,
  CreateEnvironmentRequest,
  BulkImportRequest,
} from '@/api/types';

/**
 * Garbage-collection window for heavy per-schedule payloads (sky maps,
 * insights, fragmentation, …). 5 minutes balances responsiveness when a
 * user revisits a schedule against the memory cost of holding several
 * large payloads in the react-query cache.
 */
const HEAVY_SCHEDULE_GC_TIME_MS = 5 * 60_000;

// Query keys factory
export const queryKeys = {
  health: ['health'] as const,
  schedules: ['schedules'] as const,
  schedule: (id: number) => ['schedule', id] as const,
  skyMap: (id: number) => ['skyMap', id] as const,
  distributions: (id: number) => ['distributions', id] as const,
  visibilityMap: (id: number) => ['visibilityMap', id] as const,
  visibilityHistogram: (id: number, query?: VisibilityHistogramQuery) =>
    ['visibilityHistogram', id, query] as const,
  timeline: (id: number) => ['timeline', id] as const,
  insights: (id: number) => ['insights', id] as const,
  fragmentation: (id: number) => ['fragmentation', id] as const,
  algorithmTrace: (id: number) => ['algorithmTrace', id] as const,
  altAz: (id: number, request?: AltAzRequest) => ['altAz', id, request] as const,
  trends: (id: number, query?: TrendsQuery) => ['trends', id, query] as const,
  validationReport: (id: number) => ['validationReport', id] as const,
  compare: (id: number, otherId: number, query?: CompareQuery) =>
    ['compare', id, otherId, query] as const,
  environments: ['environments'] as const,
  environment: (id: number) => ['environment', id] as const,
  scheduleKpis: (id: number) => ['scheduleKpis', id] as const,
  environmentKpis: (id: number) => ['environmentKpis', id] as const,
};

// Health check
export function useHealth() {
  return useQuery({
    queryKey: queryKeys.health,
    queryFn: () => api.getHealth(),
    refetchInterval: 30000, // Refetch every 30 seconds
  });
}

// Schedule list
export function useSchedules(params?: { limit?: number; offset?: number }) {
  return useQuery({
    queryKey: [...queryKeys.schedules, params?.limit ?? null, params?.offset ?? null] as const,
    queryFn: () => api.listSchedules(params),
    staleTime: 30_000,
  });
}

// Create schedule mutation
export interface CreateScheduleVariables {
  request: CreateScheduleRequest;
  onUploadProgress?: (event: AxiosProgressEvent) => void;
}

export function useCreateSchedule() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (vars: CreateScheduleVariables) =>
      api.createSchedule(vars.request, vars.onUploadProgress),
    onSuccess: () => {
      // Invalidate schedules list to refetch
      queryClient.invalidateQueries({ queryKey: queryKeys.schedules });
    },
  });
}

// Delete schedule mutation
export function useDeleteSchedule() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (scheduleId: number) => api.deleteSchedule(scheduleId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.schedules });
    },
  });
}

// Bulk-delete schedules. The backend currently exposes only single-schedule
// delete, so this hook sequences calls client-side and reports the count of
// successfully deleted schedules. The shape mirrors the planned bulk endpoint
// (`{ deleted_count, message }`) so callers can switch transparently when it
// ships.
export function useDeleteSchedules() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (scheduleIds: number[]) => {
      let deleted = 0;
      for (const id of scheduleIds) {
        await api.deleteSchedule(id);
        deleted += 1;
      }
      return {
        deleted_count: deleted,
        message: `Deleted ${deleted} schedule${deleted === 1 ? '' : 's'}`,
      };
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.schedules });
    },
  });
}

// Update schedule mutation
export function useUpdateSchedule() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ scheduleId, request }: { scheduleId: number; request: UpdateScheduleRequest }) =>
      api.updateSchedule(scheduleId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.schedules });
    },
  });
}

// Visualization hooks
export function useSkyMap(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.skyMap(scheduleId),
    queryFn: ({ signal }) => api.getSkyMap(scheduleId, { signal }),
    enabled: scheduleId > 0,
    gcTime: HEAVY_SCHEDULE_GC_TIME_MS,
  });
}

export function useDistributions(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.distributions(scheduleId),
    queryFn: ({ signal }) => api.getDistributions(scheduleId, { signal }),
    enabled: scheduleId > 0,
    gcTime: HEAVY_SCHEDULE_GC_TIME_MS,
  });
}

export function useVisibilityMap(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.visibilityMap(scheduleId),
    queryFn: ({ signal }) => api.getVisibilityMap(scheduleId, { signal }),
    enabled: scheduleId > 0,
    gcTime: HEAVY_SCHEDULE_GC_TIME_MS,
  });
}

export function useVisibilityHistogram(scheduleId: number, query?: VisibilityHistogramQuery) {
  return useQuery({
    queryKey: queryKeys.visibilityHistogram(scheduleId, query),
    queryFn: ({ signal }) => api.getVisibilityHistogram(scheduleId, query, { signal }),
    enabled: scheduleId > 0,
    placeholderData: keepPreviousData,
  });
}

export function useTimeline(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.timeline(scheduleId),
    queryFn: ({ signal }) => api.getTimeline(scheduleId, { signal }),
    enabled: scheduleId > 0,
    gcTime: HEAVY_SCHEDULE_GC_TIME_MS,
  });
}

export function useInsights(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.insights(scheduleId),
    queryFn: ({ signal }) => api.getInsights(scheduleId, { signal }),
    enabled: scheduleId > 0,
    gcTime: HEAVY_SCHEDULE_GC_TIME_MS,
  });
}

export function useFragmentation(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.fragmentation(scheduleId),
    queryFn: ({ signal }) => api.getFragmentation(scheduleId, { signal }),
    enabled: scheduleId > 0,
    gcTime: HEAVY_SCHEDULE_GC_TIME_MS,
  });
}

/**
 * Fetch the algorithm trace for a schedule.  Returns `undefined` data
 * when the schedule was not produced by an algorithm that emits a trace
 * or when the trace was not uploaded; the underlying 404 is surfaced via
 * `error`.
 */
export function useAlgorithmTrace(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.algorithmTrace(scheduleId),
    queryFn: ({ signal }) => api.getAlgorithmTrace(scheduleId, { signal }),
    enabled: scheduleId > 0,
    retry: false,
    gcTime: HEAVY_SCHEDULE_GC_TIME_MS,
  });
}

export function useAltAz(scheduleId: number, request?: AltAzRequest) {
  return useQuery({
    queryKey: queryKeys.altAz(scheduleId, request),
    queryFn: ({ signal }) => api.computeAltAz(scheduleId, request as AltAzRequest, { signal }),
    enabled: scheduleId > 0 && !!request,
  });
}

export function useTrends(scheduleId: number, query?: TrendsQuery) {
  return useQuery({
    queryKey: queryKeys.trends(scheduleId, query),
    queryFn: ({ signal }) => api.getTrends(scheduleId, query, { signal }),
    enabled: scheduleId > 0,
  });
}

export function useValidationReport(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.validationReport(scheduleId),
    queryFn: ({ signal }) => api.getValidationReport(scheduleId, { signal }),
    enabled: scheduleId > 0,
    gcTime: HEAVY_SCHEDULE_GC_TIME_MS,
  });
}

export function useCompare(scheduleId: number, otherId: number, query?: CompareQuery) {
  return useQuery({
    queryKey: queryKeys.compare(scheduleId, otherId, query),
    queryFn: ({ signal }) => api.compareSchedules(scheduleId, otherId, query, { signal }),
    enabled: scheduleId > 0 && otherId > 0,
    gcTime: HEAVY_SCHEDULE_GC_TIME_MS,
  });
}

// Environments

export function useEnvironments() {
  return useQuery({
    queryKey: queryKeys.environments,
    queryFn: () => api.listEnvironments(),
    staleTime: 30_000,
  });
}

export function useEnvironment(environmentId: number) {
  return useQuery({
    queryKey: queryKeys.environment(environmentId),
    queryFn: () => api.getEnvironment(environmentId),
    enabled: environmentId > 0,
  });
}

export function useEnvironmentKpis(environmentId: number) {
  return useQuery({
    queryKey: queryKeys.environmentKpis(environmentId),
    queryFn: () => api.getEnvironmentKpis(environmentId),
    enabled: environmentId > 0,
  });
}

export function useCreateEnvironment() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: CreateEnvironmentRequest) => api.createEnvironment(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.environments });
    },
  });
}

export function useDeleteEnvironment() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (environmentId: number) => api.deleteEnvironment(environmentId),
    onSuccess: (_data, environmentId) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.environments });
      queryClient.invalidateQueries({ queryKey: queryKeys.environment(environmentId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.schedules });
    },
  });
}

export function useBulkImportToEnvironment() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      environmentId,
      request,
    }: {
      environmentId: number;
      request: BulkImportRequest;
    }) => api.bulkImportToEnvironment(environmentId, request),
    onSuccess: (_data, { environmentId }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.environments });
      queryClient.invalidateQueries({ queryKey: queryKeys.environment(environmentId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.schedules });
    },
  });
}

export function useRemoveScheduleFromEnvironment() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (scheduleId: number) => api.removeScheduleFromEnvironment(scheduleId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.environments });
      queryClient.invalidateQueries({ queryKey: queryKeys.schedules });
    },
  });
}
