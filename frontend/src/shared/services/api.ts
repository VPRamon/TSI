/**
 * Centralized API client service
 */

import axios, { AxiosInstance, AxiosRequestConfig } from 'axios'
import { API_BASE_URL } from '@/shared/utils/constants'
import type {
  SchedulingBlock,
  DatasetMetadata,
  DistributionStats,
  Histogram,
  Metrics,
  ConflictReport,
  CorrelationData
} from '@/shared/types'

class ApiService {
  private client: AxiosInstance

  constructor(baseURL: string = API_BASE_URL) {
    this.client = axios.create({
      baseURL,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json'
      }
    })

    this.setupInterceptors()
  }

  private setupInterceptors() {
    // Request interceptor
    this.client.interceptors.request.use(
      (config) => {
        // Add any auth tokens or custom headers here
        return config
      },
      (error) => Promise.reject(error)
    )

    // Response interceptor
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        const message = error.response?.data?.error || error.message || 'An error occurred'
        return Promise.reject(new Error(message))
      }
    )
  }

  // Dataset endpoints
  async getCurrentDataset(): Promise<{ blocks: SchedulingBlock[] }> {
    const response = await this.client.get('/datasets/current')
    return response.data
  }

  async getDatasetMetadata(): Promise<DatasetMetadata> {
    const response = await this.client.get('/datasets/current/metadata')
    return response.data
  }

  async uploadCSV(file: File): Promise<{ metadata: DatasetMetadata }> {
    const formData = new FormData()
    formData.append('file', file)
    
    const response = await this.client.post('/datasets/upload/csv', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
    return response.data
  }

  async uploadJSON(scheduleFile: File, visibilityFile?: File): Promise<{ metadata: DatasetMetadata }> {
    const formData = new FormData()
    formData.append('schedule', scheduleFile)
    if (visibilityFile) {
      formData.append('visibility', visibilityFile)
    }
    
    const response = await this.client.post('/datasets/upload/json', formData, {
      headers: { 'Content-Type': 'multipart/form-data' }
    })
    return response.data
  }

  async loadSampleDataset(): Promise<{ metadata: DatasetMetadata }> {
    const response = await this.client.post('/datasets/sample')
    return response.data
  }

  // Analytics endpoints
  async getDistributionStats(column: string): Promise<DistributionStats> {
    const response = await this.client.get('/analytics/distribution', {
      params: { column, stats: true }
    })
    return response.data
  }

  async getHistogram(column: string, bins: number = 20): Promise<Histogram> {
    const response = await this.client.get('/analytics/distribution', {
      params: { column, bins }
    })
    return response.data
  }

  async getMetrics(): Promise<Metrics> {
    const response = await this.client.get('/analytics/metrics')
    return response.data
  }

  async getConflicts(): Promise<ConflictReport> {
    const response = await this.client.get('/analytics/conflicts')
    return response.data
  }

  async getCorrelations(columns: string): Promise<CorrelationData> {
    const response = await this.client.get('/analytics/correlations', {
      params: { columns }
    })
    return response.data
  }

  async getTopObservations(params: {
    by: string
    order?: 'ascending' | 'descending'
    n?: number
  }): Promise<SchedulingBlock[]> {
    const response = await this.client.get('/analytics/top', { params })
    return response.data
  }

  // Generic request method for custom endpoints
  async request<T>(config: AxiosRequestConfig): Promise<T> {
    const response = await this.client.request<T>(config)
    return response.data
  }
}

// Export singleton instance
export const apiService = new ApiService()

// Also export the class for testing or custom instances
export default ApiService
