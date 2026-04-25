/**
 * React Query hooks for the TSI API.
 */
import { useQuery, useMutation, useQueryClient, keepPreviousData } from '@tanstack/react-query';
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
  altAz: (id: number, request?: AltAzRequest) => ['altAz', id, request] as const,
  trends: (id: number, query?: TrendsQuery) => ['trends', id, query] as const,
  validationReport: (id: number) => ['validationReport', id] as const,
  compare: (id: number, otherId: number, query?: CompareQuery) =>
    ['compare', id, otherId, query] as const,
  environments: ['environments'] as const,
  environment: (id: number) => ['environment', id] as const,
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
export function useSchedules() {
  return useQuery({
    queryKey: queryKeys.schedules,
    queryFn: () => api.listSchedules(),
  });
}

// Create schedule mutation
export function useCreateSchedule() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: CreateScheduleRequest) => api.createSchedule(request),
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
    queryFn: () => api.getSkyMap(scheduleId),
    enabled: scheduleId > 0,
  });
}

export function useDistributions(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.distributions(scheduleId),
    queryFn: () => api.getDistributions(scheduleId),
    enabled: scheduleId > 0,
  });
}

export function useVisibilityMap(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.visibilityMap(scheduleId),
    queryFn: () => api.getVisibilityMap(scheduleId),
    enabled: scheduleId > 0,
  });
}

export function useVisibilityHistogram(scheduleId: number, query?: VisibilityHistogramQuery) {
  return useQuery({
    queryKey: queryKeys.visibilityHistogram(scheduleId, query),
    queryFn: () => api.getVisibilityHistogram(scheduleId, query),
    enabled: scheduleId > 0,
    placeholderData: keepPreviousData,
  });
}

export function useTimeline(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.timeline(scheduleId),
    queryFn: () => api.getTimeline(scheduleId),
    enabled: scheduleId > 0,
  });
}

export function useInsights(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.insights(scheduleId),
    queryFn: () => api.getInsights(scheduleId),
    enabled: scheduleId > 0,
  });
}

export function useFragmentation(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.fragmentation(scheduleId),
    queryFn: () => api.getFragmentation(scheduleId),
    enabled: scheduleId > 0,
  });
}

export function useAltAz(scheduleId: number, request?: AltAzRequest) {
  return useQuery({
    queryKey: queryKeys.altAz(scheduleId, request),
    queryFn: () => api.computeAltAz(scheduleId, request as AltAzRequest),
    enabled: scheduleId > 0 && !!request,
  });
}

export function useTrends(scheduleId: number, query?: TrendsQuery) {
  return useQuery({
    queryKey: queryKeys.trends(scheduleId, query),
    queryFn: () => api.getTrends(scheduleId, query),
    enabled: scheduleId > 0,
  });
}

export function useValidationReport(scheduleId: number) {
  return useQuery({
    queryKey: queryKeys.validationReport(scheduleId),
    queryFn: () => api.getValidationReport(scheduleId),
    enabled: scheduleId > 0,
  });
}

export function useCompare(scheduleId: number, otherId: number, query?: CompareQuery) {
  return useQuery({
    queryKey: queryKeys.compare(scheduleId, otherId, query),
    queryFn: () => api.compareSchedules(scheduleId, otherId, query),
    enabled: scheduleId > 0 && otherId > 0,
  });
}

// Environments

export function useEnvironments() {
  return useQuery({
    queryKey: queryKeys.environments,
    queryFn: () => api.listEnvironments(),
  });
}

export function useEnvironment(environmentId: number) {
  return useQuery({
    queryKey: queryKeys.environment(environmentId),
    queryFn: () => api.getEnvironment(environmentId),
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
