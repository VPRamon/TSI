/**
 * API client for the TSI backend.
 */
import axios, { AxiosInstance, AxiosError } from 'axios';
import type {
  ScheduleListResponse,
  CreateScheduleRequest,
  CreateScheduleResponse,
  JobStatusResponse,
  HealthResponse,
  SkyMapData,
  DistributionData,
  ScheduleTimelineData,
  InsightsData,
  FragmentationData,
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
  ScheduleInfo,
  EnvironmentInfo,
  EnvironmentListResponse,
  CreateEnvironmentRequest,
  BulkImportRequest,
  BulkImportResponse,
  DeleteEnvironmentResponse,
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
  async listSchedules(): Promise<ScheduleListResponse> {
    const { data } = await this.client.get<ScheduleListResponse>('/v1/schedules');
    return data;
  }

  async getSchedule(scheduleId: number): Promise<unknown> {
    const { data } = await this.client.get<unknown>(`/v1/schedules/${scheduleId}`);
    return data;
  }

  async createSchedule(request: CreateScheduleRequest): Promise<CreateScheduleResponse> {
    const { data } = await this.client.post<CreateScheduleResponse>('/v1/schedules', request);
    return data;
  }

  async deleteSchedule(scheduleId: number): Promise<DeleteScheduleResponse> {
    const { data } = await this.client.delete<DeleteScheduleResponse>(
      `/v1/schedules/${scheduleId}`
    );
    return data;
  }

  async updateSchedule(scheduleId: number, request: UpdateScheduleRequest): Promise<ScheduleInfo> {
    const { data } = await this.client.patch<ScheduleInfo>(`/v1/schedules/${scheduleId}`, request);
    return data;
  }

  // Visualization endpoints
  async getSkyMap(scheduleId: number): Promise<SkyMapData> {
    const { data } = await this.client.get<SkyMapData>(`/v1/schedules/${scheduleId}/sky-map`);
    return data;
  }

  async getDistributions(scheduleId: number): Promise<DistributionData> {
    const { data } = await this.client.get<DistributionData>(
      `/v1/schedules/${scheduleId}/distributions`
    );
    return data;
  }

  async getVisibilityMap(scheduleId: number): Promise<VisibilityMapData> {
    const { data } = await this.client.get<VisibilityMapData>(
      `/v1/schedules/${scheduleId}/visibility-map`
    );
    return data;
  }

  async getVisibilityHistogram(
    scheduleId: number,
    query?: VisibilityHistogramQuery
  ): Promise<VisibilityBin[]> {
    const { data } = await this.client.get<VisibilityBin[]>(
      `/v1/schedules/${scheduleId}/visibility-histogram`,
      { params: query }
    );
    return data;
  }

  async getTimeline(scheduleId: number): Promise<ScheduleTimelineData> {
    const { data } = await this.client.get<ScheduleTimelineData>(
      `/v1/schedules/${scheduleId}/timeline`
    );
    return data;
  }

  async getInsights(scheduleId: number): Promise<InsightsData> {
    const { data } = await this.client.get<InsightsData>(`/v1/schedules/${scheduleId}/insights`);
    return data;
  }

  async getFragmentation(scheduleId: number): Promise<FragmentationData> {
    const { data } = await this.client.get<FragmentationData>(
      `/v1/schedules/${scheduleId}/fragmentation`
    );
    return data;
  }

  async computeAltAz(scheduleId: number, request: AltAzRequest): Promise<AltAzData> {
    const { data } = await this.client.post<AltAzData>(`/v1/schedules/${scheduleId}/alt-az`, request);
    return data;
  }

  async getTrends(scheduleId: number, query?: TrendsQuery): Promise<TrendsData> {
    const { data } = await this.client.get<TrendsData>(`/v1/schedules/${scheduleId}/trends`, {
      params: query,
    });
    return data;
  }

  async getValidationReport(scheduleId: number): Promise<ValidationReport> {
    const { data } = await this.client.get<ValidationReport>(
      `/v1/schedules/${scheduleId}/validation-report`
    );
    return data;
  }

  async compareSchedules(
    scheduleId: number,
    otherId: number,
    query?: CompareQuery
  ): Promise<CompareData> {
    const { data } = await this.client.get<CompareData>(
      `/v1/schedules/${scheduleId}/compare/${otherId}`,
      { params: query }
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
