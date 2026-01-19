/**
 * TypeScript types for the TSI REST API.
 * These types mirror the Rust DTOs from the backend.
 */

// =============================================================================
// Core Types
// =============================================================================

export interface ScheduleInfo {
  schedule_id: number;
  schedule_name: string;
}

export interface ScheduleListResponse {
  schedules: ScheduleInfo[];
  total: number;
}

export interface CreateScheduleRequest {
  name: string;
  schedule_json: unknown;
  visibility_json?: unknown;
  populate_analytics?: boolean;
}

export interface CreateScheduleResponse {
  job_id: string;
  message: string;
}

export interface LogEntry {
  timestamp: string;
  level: 'info' | 'success' | 'warning' | 'error';
  message: string;
}

export interface JobStatusResponse {
  job_id: string;
  status: string;
  logs: LogEntry[];
  result?: {
    schedule_id: number;
    schedule_name: string;
  };
}

export interface HealthResponse {
  status: string;
  version: string;
  database: string;
}

// =============================================================================
// Visualization Types
// =============================================================================

export interface Period {
  start: number;
  stop: number;
}

export interface PriorityBinInfo {
  label: string;
  min_priority: number;
  max_priority: number;
  color: string;
}

// Sky Map
export interface LightweightBlock {
  original_block_id: string;
  priority: number;
  priority_bin: string;
  requested_duration_seconds: number;
  target_ra_deg: number;
  target_dec_deg: number;
  scheduled_period: Period | null;
}

export interface SkyMapData {
  blocks: LightweightBlock[];
  priority_bins: PriorityBinInfo[];
  priority_min: number;
  priority_max: number;
  ra_min: number;
  ra_max: number;
  dec_min: number;
  dec_max: number;
  total_count: number;
  scheduled_count: number;
  scheduled_time_min: number | null;
  scheduled_time_max: number | null;
}

// Distributions
export interface DistributionBlock {
  priority: number;
  total_visibility_hours: number;
  requested_hours: number;
  elevation_range_deg: number;
  scheduled: boolean;
}

export interface DistributionStats {
  count: number;
  mean: number;
  median: number;
  std_dev: number;
  min: number;
  max: number;
  sum: number;
}

export interface DistributionData {
  blocks: DistributionBlock[];
  priority_stats: DistributionStats;
  visibility_stats: DistributionStats;
  requested_hours_stats: DistributionStats;
  total_count: number;
  scheduled_count: number;
  unscheduled_count: number;
  impossible_count: number;
}

// Timeline
export interface ScheduleTimelineBlock {
  scheduling_block_id: number;
  original_block_id: string;
  priority: number;
  scheduled_start_mjd: number;
  scheduled_stop_mjd: number;
  ra_deg: number;
  dec_deg: number;
  requested_hours: number;
  total_visibility_hours: number;
  num_visibility_periods: number;
}

export interface ScheduleTimelineData {
  blocks: ScheduleTimelineBlock[];
  priority_min: number;
  priority_max: number;
  total_count: number;
  scheduled_count: number;
  unique_months: string[];
  dark_periods: Period[];
}

// Insights
export interface InsightsBlock {
  scheduling_block_id: number;
  original_block_id: string;
  priority: number;
  total_visibility_hours: number;
  requested_hours: number;
  elevation_range_deg: number;
  scheduled: boolean;
  scheduled_start_mjd: number | null;
  scheduled_stop_mjd: number | null;
}

export interface AnalyticsMetrics {
  total_observations: number;
  scheduled_count: number;
  unscheduled_count: number;
  scheduling_rate: number;
  mean_priority: number;
  median_priority: number;
  mean_priority_scheduled: number;
  mean_priority_unscheduled: number;
  total_visibility_hours: number;
  mean_requested_hours: number;
}

export interface CorrelationEntry {
  variable1: string;
  variable2: string;
  correlation: number;
}

export interface ConflictRecord {
  block_id_1: string;
  block_id_2: string;
  start_time_1: number;
  stop_time_1: number;
  start_time_2: number;
  stop_time_2: number;
  overlap_hours: number;
}

export interface TopObservation {
  scheduling_block_id: number;
  original_block_id: string;
  priority: number;
  total_visibility_hours: number;
  requested_hours: number;
  scheduled: boolean;
}

export interface InsightsData {
  blocks: InsightsBlock[];
  metrics: AnalyticsMetrics;
  correlations: CorrelationEntry[];
  top_priority: TopObservation[];
  top_visibility: TopObservation[];
  conflicts: ConflictRecord[];
  total_count: number;
  scheduled_count: number;
  impossible_count: number;
}

// Trends
export interface TrendsBlock {
  scheduling_block_id: number;
  original_block_id: string;
  priority: number;
  total_visibility_hours: number;
  requested_hours: number;
  scheduled: boolean;
}

export interface EmpiricalRatePoint {
  bin_label: string;
  mid_value: number;
  scheduled_rate: number;
  count: number;
}

export interface SmoothedPoint {
  x: number;
  y_smoothed: number;
  n_samples: number;
}

export interface HeatmapBin {
  visibility_mid: number;
  time_mid: number;
  scheduled_rate: number;
  count: number;
}

export interface TrendsMetrics {
  total_count: number;
  scheduled_count: number;
  scheduling_rate: number;
  zero_visibility_count: number;
  priority_min: number;
  priority_max: number;
  priority_mean: number;
  visibility_min: number;
  visibility_max: number;
  visibility_mean: number;
  time_min: number;
  time_max: number;
  time_mean: number;
}

export interface TrendsData {
  blocks: TrendsBlock[];
  metrics: TrendsMetrics;
  by_priority: EmpiricalRatePoint[];
  by_visibility: EmpiricalRatePoint[];
  by_time: EmpiricalRatePoint[];
  smoothed_visibility: SmoothedPoint[];
  smoothed_time: SmoothedPoint[];
  heatmap_bins: HeatmapBin[];
  priority_values: number[];
}

// Validation
export interface ValidationIssue {
  block_id: number;
  original_block_id: string | null;
  issue_type: string;
  category: string;
  criticality: string;
  field_name: string | null;
  current_value: string | null;
  expected_value: string | null;
  description: string;
}

export interface ValidationReport {
  schedule_id: number;
  total_blocks: number;
  valid_blocks: number;
  impossible_blocks: ValidationIssue[];
  validation_errors: ValidationIssue[];
  validation_warnings: ValidationIssue[];
}

// Compare
export interface CompareBlock {
  scheduling_block_id: string;
  priority: number;
  scheduled: boolean;
  requested_hours: number;
}

export interface CompareStats {
  scheduled_count: number;
  unscheduled_count: number;
  total_priority: number;
  mean_priority: number;
  median_priority: number;
  total_hours: number;
  gap_count: number | null;
  gap_mean_hours: number | null;
  gap_median_hours: number | null;
}

export interface SchedulingChange {
  scheduling_block_id: string;
  priority: number;
  change_type: string;
}

export interface CompareData {
  current_blocks: CompareBlock[];
  comparison_blocks: CompareBlock[];
  current_stats: CompareStats;
  comparison_stats: CompareStats;
  common_ids: string[];
  only_in_current: string[];
  only_in_comparison: string[];
  scheduling_changes: SchedulingChange[];
  current_name: string;
  comparison_name: string;
}

// Visibility Map
export interface VisibilityBlockSummary {
  scheduling_block_id: number;
  original_block_id: string;
  priority: number;
  num_visibility_periods: number;
  scheduled: boolean;
}

export interface VisibilityMapData {
  blocks: VisibilityBlockSummary[];
  priority_min: number;
  priority_max: number;
  total_count: number;
  scheduled_count: number;
}

export interface VisibilityBin {
  bin_start_unix: number;
  bin_end_unix: number;
  visible_count: number;
}

// =============================================================================
// Query Parameters
// =============================================================================

export interface TrendsQuery {
  bins?: number;
  bandwidth?: number;
  points?: number;
}

export interface CompareQuery {
  current_name?: string;
  comparison_name?: string;
}

export interface VisibilityHistogramQuery {
  bin_duration_minutes?: number;
  num_bins?: number;
  priority_min?: number;
  priority_max?: number;
  block_ids?: number[];
}

// =============================================================================
// Error Types
// =============================================================================

export interface ApiError {
  code: string;
  message: string;
  details?: string;
}
