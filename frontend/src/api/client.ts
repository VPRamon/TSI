/**
 * API client for the TSI backend.
 */
import axios, { AxiosInstance, AxiosError, AxiosProgressEvent } from 'axios';
import type {
  ScheduleListResponse,
  ScheduleListEnvelope,
  ListSchedulesParams,
  CreateScheduleRequest,
  CreateScheduleResponse,
  JobStatusResponse,
  HealthResponse,
  SkyMapData,
  DistributionData,
  ScheduleTimelineData,
  InsightsData,
  FragmentationData,
  AlgorithmTraceResponse,
  AltAzData,
  AltAzRequest,
  TrendsData,
  TrendsQuery,
  ValidationReport,
  CompareData,
  CompareQuery,
  VisibilityMapData,
  VisibilityBin,
  VisibilityHistogramQuery,
  ApiError as ApiErrorResponse,
  UpdateScheduleRequest,
  DeleteScheduleResponse,
  BulkDeleteSchedulesRequest,
  BulkDeleteSchedulesResponse,
  ScheduleInfo,
  EnvironmentInfo,
  EnvironmentListResponse,
  CreateEnvironmentRequest,
  BulkImportRequest,
  BulkImportResponse,
  DeleteEnvironmentResponse,
  ScheduleKpi,
  EnvironmentKpisResponse,
} from './types';
import {
  ApiRequestError,
  NotFoundError,
  ValidationError,
  NetworkError,
  ServerError,
  RateLimitError,
} from './errors';

// Base URL - use /api prefix for both dev (proxy) and prod (creates consistency with nginx)
const BASE_URL = '/api';

class ApiClient {
  private client: AxiosInstance;

  constructor() {
    this.client = axios.create({
      baseURL: BASE_URL,
      headers: {
        'Content-Type': 'application/json',
      },
      paramsSerializer: (params) => {
        const searchParams = new URLSearchParams();

        Object.entries(params ?? {}).forEach(([key, value]) => {
          if (value === undefined || value === null) {
            return;
          }

          if (Array.isArray(value)) {
            if (value.length > 0) {
              searchParams.set(key, value.map(String).join(','));
            }
            return;
          }

          searchParams.append(key, String(value));
        });

        return searchParams.toString();
      },
    });

    // Response interceptor for error handling with typed errors
    this.client.interceptors.response.use(
      (response) => response,
      (error: AxiosError<ApiErrorResponse>) => {
        // Network error (no response)
        if (!error.response) {
          throw new NetworkError();
        }

        const { status, data } = error.response;
        const message = data?.message || 'An error occurred';

        // Map status codes to typed errors
        switch (status) {
          case 400:
            throw new ValidationError(message);
          case 404: {
            // Extract resource info from URL if available
            const urlParts = error.config?.url?.split('/') ?? [];
            const resourceType = urlParts[urlParts.length - 2] || 'Resource';
            const resourceId = urlParts[urlParts.length - 1] || 'unknown';
            throw new NotFoundError(resourceType, resourceId);
          }
          case 429: {
            const retryAfter = parseInt(error.response.headers['retry-after'] ?? '60', 10);
            throw new RateLimitError(retryAfter);
          }
          case 500:
          case 502:
          case 503:
          case 504:
            throw new ServerError(message);
          default:
            throw new ApiRequestError(message, status, status >= 500);
        }
      }
    );
  }

  // Health check
  async getHealth(): Promise<HealthResponse> {
    const { data } = await this.client.get<HealthResponse>('/health');
    return data;
  }

  // Schedule CRUD
  async listSchedules(params?: ListSchedulesParams): Promise<ScheduleListResponse> {
    const { data } = await this.client.get<ScheduleListEnvelope>('/v1/schedules', {
      params: {
        ...(params?.limit !== undefined ? { limit: params.limit } : {}),
        ...(params?.offset !== undefined ? { offset: params.offset } : {}),
      },
    });
    return {
      schedules: data.items,
      total: data.total,
      limit: data.limit,
      offset: data.offset,
    };
  }

  async getSchedule(scheduleId: number, init?: { signal?: AbortSignal }): Promise<unknown> {
    const { data } = await this.client.get<unknown>(`/v1/schedules/${scheduleId}`, {
      signal: init?.signal,
    });
    return data;
  }

  async createSchedule(
    request: CreateScheduleRequest,
    onUploadProgress?: (event: AxiosProgressEvent) => void
  ): Promise<CreateScheduleResponse> {
    const { data } = await this.client.post<CreateScheduleResponse>('/v1/schedules', request, {
      onUploadProgress,
    });
    return data;
  }

  async deleteSchedule(scheduleId: number): Promise<DeleteScheduleResponse> {
    const { data } = await this.client.delete<DeleteScheduleResponse>(
      `/v1/schedules/${scheduleId}`
    );
    return data;
  }

  async bulkDeleteSchedules(scheduleIds: number[]): Promise<BulkDeleteSchedulesResponse> {
    const { data } = await this.client.post<BulkDeleteSchedulesResponse>(
      '/v1/schedules/bulk-delete',
      { schedule_ids: scheduleIds } satisfies BulkDeleteSchedulesRequest
    );
    return data;
  }

  async updateSchedule(scheduleId: number, request: UpdateScheduleRequest): Promise<ScheduleInfo> {
    const { data } = await this.client.patch<ScheduleInfo>(`/v1/schedules/${scheduleId}`, request);
    return data;
  }

  // Visualization endpoints
  async getSkyMap(scheduleId: number, init?: { signal?: AbortSignal }): Promise<SkyMapData> {
    const { data } = await this.client.get<SkyMapData>(`/v1/schedules/${scheduleId}/sky-map`, {
      signal: init?.signal,
    });
    return data;
  }

  async getDistributions(
    scheduleId: number,
    init?: { signal?: AbortSignal }
  ): Promise<DistributionData> {
    const { data } = await this.client.get<DistributionData>(
      `/v1/schedules/${scheduleId}/distributions`,
      { signal: init?.signal }
    );
    return data;
  }

  async getVisibilityMap(
    scheduleId: number,
    init?: { signal?: AbortSignal }
  ): Promise<VisibilityMapData> {
    const { data } = await this.client.get<VisibilityMapData>(
      `/v1/schedules/${scheduleId}/visibility-map`,
      { signal: init?.signal }
    );
    return data;
  }

  async getVisibilityHistogram(
    scheduleId: number,
    query?: VisibilityHistogramQuery,
    init?: { signal?: AbortSignal }
  ): Promise<VisibilityBin[]> {
    const { data } = await this.client.get<VisibilityBin[]>(
      `/v1/schedules/${scheduleId}/visibility-histogram`,
      { params: query, signal: init?.signal }
    );
    return data;
  }

  async getTimeline(
    scheduleId: number,
    init?: { signal?: AbortSignal }
  ): Promise<ScheduleTimelineData> {
    const { data } = await this.client.get<ScheduleTimelineData>(
      `/v1/schedules/${scheduleId}/timeline`,
      { signal: init?.signal }
    );
    return data;
  }

  async getInsights(
    scheduleId: number,
    init?: { signal?: AbortSignal }
  ): Promise<InsightsData> {
    const { data } = await this.client.get<InsightsData>(
      `/v1/schedules/${scheduleId}/insights`,
      { signal: init?.signal }
    );
    return data;
  }

  async getFragmentation(
    scheduleId: number,
    init?: { signal?: AbortSignal }
  ): Promise<FragmentationData> {
    const { data } = await this.client.get<FragmentationData>(
      `/v1/schedules/${scheduleId}/fragmentation`,
      { signal: init?.signal }
    );
    return data;
  }

  async getScheduleKpis(scheduleId: number): Promise<ScheduleKpi> {
    const { data } = await this.client.get<ScheduleKpi>(`/v1/schedules/${scheduleId}/kpis`);
    return data;
  }

  async getEnvironmentKpis(environmentId: number): Promise<EnvironmentKpisResponse> {
    const { data } = await this.client.get<EnvironmentKpisResponse>(
      `/v1/environments/${environmentId}/kpis`
    );
    return data;
  }

  async getAlgorithmTrace(
    scheduleId: number,
    init?: { signal?: AbortSignal }
  ): Promise<AlgorithmTraceResponse> {
    const { data } = await this.client.get<AlgorithmTraceResponse>(
      `/v1/schedules/${scheduleId}/algorithm_trace`,
      { signal: init?.signal }
    );
    return data;
  }

  async computeAltAz(
    scheduleId: number,
    request: AltAzRequest,
    init?: { signal?: AbortSignal }
  ): Promise<AltAzData> {
    const { data } = await this.client.post<AltAzData>(
      `/v1/schedules/${scheduleId}/alt-az`,
      request,
      { signal: init?.signal }
    );
    return data;
  }

  async getTrends(
    scheduleId: number,
    query?: TrendsQuery,
    init?: { signal?: AbortSignal }
  ): Promise<TrendsData> {
    const { data } = await this.client.get<TrendsData>(`/v1/schedules/${scheduleId}/trends`, {
      params: query,
      signal: init?.signal,
    });
    return data;
  }

  async getValidationReport(
    scheduleId: number,
    init?: { signal?: AbortSignal }
  ): Promise<ValidationReport> {
    const { data } = await this.client.get<ValidationReport>(
      `/v1/schedules/${scheduleId}/validation-report`,
      { signal: init?.signal }
    );
    return data;
  }

  async compareSchedules(
    scheduleId: number,
    otherId: number,
    query?: CompareQuery,
    init?: { signal?: AbortSignal }
  ): Promise<CompareData> {
    const { data } = await this.client.get<CompareData>(
      `/v1/schedules/${scheduleId}/compare/${otherId}`,
      { params: query, signal: init?.signal }
    );
    return data;
  }

  // Job management
  async getJobStatus(jobId: string): Promise<JobStatusResponse> {
    const { data } = await this.client.get<JobStatusResponse>(`/v1/jobs/${jobId}`);
    return data;
  }

  // Environments
  async listEnvironments(): Promise<EnvironmentListResponse> {
    const { data } = await this.client.get<EnvironmentListResponse>('/v1/environments');
    return data;
  }

  async getEnvironment(environmentId: number): Promise<EnvironmentInfo> {
    const { data } = await this.client.get<EnvironmentInfo>(
      `/v1/environments/${environmentId}`
    );
    return data;
  }

  async createEnvironment(request: CreateEnvironmentRequest): Promise<EnvironmentInfo> {
    const { data } = await this.client.post<EnvironmentInfo>('/v1/environments', request);
    return data;
  }

  async deleteEnvironment(environmentId: number): Promise<DeleteEnvironmentResponse> {
    const { data } = await this.client.delete<DeleteEnvironmentResponse>(
      `/v1/environments/${environmentId}`
    );
    return data;
  }

  async bulkImportToEnvironment(
    environmentId: number,
    request: BulkImportRequest
  ): Promise<BulkImportResponse> {
    const { data } = await this.client.post<BulkImportResponse>(
      `/v1/environments/${environmentId}/schedules`,
      request
    );
    return data;
  }

  async removeScheduleFromEnvironment(scheduleId: number): Promise<DeleteEnvironmentResponse> {
    const { data } = await this.client.delete<DeleteEnvironmentResponse>(
      `/v1/schedules/${scheduleId}/environment`
    );
    return data;
  }
}

// Export a singleton instance
export const api = new ApiClient();
export default api;
