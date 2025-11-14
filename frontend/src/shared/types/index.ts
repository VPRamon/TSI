/**
 * Shared type definitions for the TSI application
 */

export interface SchedulingBlock {
  scheduling_block_id: string
  right_ascension_deg: number
  declination_deg: number
  priority: number
  priority_bin: string
  scheduled_flag: boolean
  requested_hours: number
  total_visibility_hours: number
  elevation_range_deg?: number
  [key: string]: any
}

export interface DatasetMetadata {
  filename: string
  num_blocks: number
  num_scheduled: number
  num_unscheduled: number
  uploaded_at?: string
}

export interface DistributionStats {
  column: string
  count: number
  mean: number
  median: number
  std: number
  min: number
  max: number
  q25: number
  q50: number
  q75: number
  p10: number
  p90: number
  p95: number
  p99: number
}

export interface HistogramBin {
  bin_start: number
  bin_end: number
  count: number
  frequency: number
}

export interface Histogram {
  column: string
  bins: HistogramBin[]
  total_count: number
  min: number
  max: number
}

export interface Metrics {
  total_blocks: number
  scheduled_blocks: number
  unscheduled_blocks: number
  scheduling_rate: number
  total_requested_hours: number
  total_scheduled_hours: number
  total_visibility_hours: number
  utilization_rate: number
  priority_stats: DistributionStats
  priority_bin_counts: Record<string, number>
}

export interface Conflict {
  scheduling_block_id: string
  conflict_type: 'impossible_observation' | 'insufficient_visibility' | 'scheduling_anomaly'
  description: string
  severity: 'low' | 'medium' | 'high'
}

export interface ConflictReport {
  total_conflicts: number
  impossible_observations: number
  insufficient_visibility: number
  scheduling_anomalies: number
  conflicts: Conflict[]
}

export interface CorrelationData {
  columns: string[]
  matrix: number[][]
  correlations: Array<{
    col1: string
    col2: string
    correlation: number
    insight: string
  }>
}

export interface ApiError {
  error: string
  message?: string
}

export type LoadingState = 'idle' | 'loading' | 'success' | 'error'

export interface AsyncState<T> {
  data: T | null
  loading: boolean
  error: string | null
  state: LoadingState
}
