/**
 * API client for the TSI backend.
 */
import axios, { AxiosInstance, AxiosError } from 'axios';
import type {
  ScheduleListResponse,
  CreateScheduleRequest,
  CreateScheduleResponse,
  HealthResponse,
  SkyMapData,
  DistributionData,
  ScheduleTimelineData,
  InsightsData,
  TrendsData,
  TrendsQuery,
  ValidationReport,
  CompareData,
  CompareQuery,
  VisibilityMapData,
  ApiError,
} from './types';

// Base URL - use proxy in development
const BASE_URL = import.meta.env.PROD ? '' : '/api';

class ApiClient {
  private client: AxiosInstance;

  constructor() {
    this.client = axios.create({
      baseURL: BASE_URL,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error: AxiosError<ApiError>) => {
        if (error.response?.data) {
          throw new Error(error.response.data.message || 'An error occurred');
        }
        throw error;
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

  async createSchedule(request: CreateScheduleRequest): Promise<CreateScheduleResponse> {
    const { data } = await this.client.post<CreateScheduleResponse>('/v1/schedules', request);
    return data;
  }

  // Visualization endpoints
  async getSkyMap(scheduleId: number): Promise<SkyMapData> {
    const { data } = await this.client.get<SkyMapData>(`/v1/schedules/${scheduleId}/sky-map`);
    return data;
  }

  async getDistributions(scheduleId: number): Promise<DistributionData> {
    const { data } = await this.client.get<DistributionData>(`/v1/schedules/${scheduleId}/distributions`);
    return data;
  }

  async getVisibilityMap(scheduleId: number): Promise<VisibilityMapData> {
    const { data } = await this.client.get<VisibilityMapData>(`/v1/schedules/${scheduleId}/visibility-map`);
    return data;
  }

  async getTimeline(scheduleId: number): Promise<ScheduleTimelineData> {
    const { data } = await this.client.get<ScheduleTimelineData>(`/v1/schedules/${scheduleId}/timeline`);
    return data;
  }

  async getInsights(scheduleId: number): Promise<InsightsData> {
    const { data } = await this.client.get<InsightsData>(`/v1/schedules/${scheduleId}/insights`);
    return data;
  }

  async getTrends(scheduleId: number, query?: TrendsQuery): Promise<TrendsData> {
    const { data } = await this.client.get<TrendsData>(`/v1/schedules/${scheduleId}/trends`, {
      params: query,
    });
    return data;
  }

  async getValidationReport(scheduleId: number): Promise<ValidationReport> {
    const { data } = await this.client.get<ValidationReport>(`/v1/schedules/${scheduleId}/validation-report`);
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
}

// Export a singleton instance
export const api = new ApiClient();
export default api;
