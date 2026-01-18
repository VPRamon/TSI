/**
 * React Query hooks for the TSI API.
 */
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from '@/api';
import type { CreateScheduleRequest, TrendsQuery, CompareQuery } from '@/api/types';

// Query keys factory
export const queryKeys = {
  health: ['health'] as const,
  schedules: ['schedules'] as const,
  schedule: (id: number) => ['schedule', id] as const,
  skyMap: (id: number) => ['skyMap', id] as const,
  distributions: (id: number) => ['distributions', id] as const,
  visibilityMap: (id: number) => ['visibilityMap', id] as const,
  timeline: (id: number) => ['timeline', id] as const,
  insights: (id: number) => ['insights', id] as const,
  trends: (id: number, query?: TrendsQuery) => ['trends', id, query] as const,
  validationReport: (id: number) => ['validationReport', id] as const,
  compare: (id: number, otherId: number, query?: CompareQuery) =>
    ['compare', id, otherId, query] as const,
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
