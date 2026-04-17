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

export interface GeographicLocation {
  lon_deg: number;
  lat_deg: number;
  height: number;
}

export interface CreateScheduleRequest {
  name: string;
  schedule_json: unknown;
  visibility_json?: unknown;
  populate_analytics?: boolean;
  /** Optional geographic location override. When set, replaces any location
   *  embedded in the schedule JSON. Use for selecting a known observatory
   *  site (e.g. OBS-N, OBS-S) at import time. */
  location_override?: GeographicLocation;
  /** Optional schedule period override (in MJD). When set, replaces the
   *  period inferred from the payload. Use when the file has no scheduled
   *  blocks or does not define the schedule window explicitly. */
  schedule_period_override?: SchedulePeriodOverride;
}

export interface SchedulePeriodOverride {
  start_mjd: number;
  end_mjd: number;
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

export interface UpdateScheduleRequest {
  name?: string;
  location?: GeographicLocation;
}

export interface DeleteScheduleResponse {
  message: string;
}

export interface AltAzTargetRequest {
  original_block_id: string;
  block_name: string;
  priority: number;
  target_ra_deg: number;
  target_dec_deg: number;
}

export interface AltAzObservatoryRequest {
  lon_deg: number;
  lat_deg: number;
  height: number;
}

export interface AltAzRequest {
  observatory: AltAzObservatoryRequest;
  start_mjd: number;
  end_mjd: number;
  targets: AltAzTargetRequest[];
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
  block_name: string;
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
  block_name: string;
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
  block_name: string;
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
  block_name: string;
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

// Fragmentation
export type FragmentationSegmentKind =
  | 'non_operable'
  | 'scheduled'
  | 'no_target_visible'
  | 'visible_but_no_task_fits'
  | 'feasible_but_unused';

export type UnscheduledReason =
  | 'no_visibility'
  | 'no_contiguous_fit'
  | 'requested_exceeds_total_visibility'
  | 'other_validation_issue'
  | 'feasible_but_unscheduled';

export interface FragmentationMjdPeriod {
  start_mjd: number;
  end_mjd: number;
}

export interface FragmentationSegment {
  start_mjd: number;
  stop_mjd: number;
  duration_hours: number;
  kind: FragmentationSegmentKind;
}

export interface FragmentationGap {
  start_mjd: number;
  stop_mjd: number;
  duration_hours: number;
  cause: FragmentationSegmentKind;
}

export interface ReasonBreakdownEntry {
  kind: FragmentationSegmentKind;
  total_hours: number;
  fraction_of_operable: number;
}

export interface UnscheduledReasonSummary {
  reason: UnscheduledReason;
  block_count: number;
  example_block_ids: string[];
  example_block_names: string[];
}

export interface FragmentationMetrics {
  schedule_hours: number;
  requested_hours: number;
  operable_hours: number;
  scheduled_hours: number;
  idle_operable_hours: number;
  raw_visibility_coverage_hours: number;
  fit_visibility_coverage_hours: number;
  gap_count: number;
  gap_mean_hours: number;
  gap_median_hours: number;
  gap_std_dev_hours: number;
  gap_p90_hours: number;
  largest_gap_hours: number;
  scheduled_fraction_of_operable: number;
  idle_fraction_of_operable: number;
  raw_visibility_fraction_of_operable: number;
  fit_visibility_fraction_of_operable: number;
}

export interface AltAzCurve {
  original_block_id: string;
  block_name: string;
  priority: number;
  altitudes_deg: number[];
  azimuths_deg: number[];
}

export interface AltAzData {
  schedule_id: number;
  sample_times_mjd: number[];
  curves: AltAzCurve[];
}

export interface FragmentationData {
  schedule_id: number;
  schedule_name: string;
  schedule_window: FragmentationMjdPeriod;
  operable_periods: FragmentationMjdPeriod[];
  operable_source: string;
  segments: FragmentationSegment[];
  largest_gaps: FragmentationGap[];
  reason_breakdown: ReasonBreakdownEntry[];
  unscheduled_reasons: UnscheduledReasonSummary[];
  metrics: FragmentationMetrics;
}

// Trends
export interface TrendsBlock {
  scheduling_block_id: number;
  original_block_id: string;
  block_name: string;
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
  block_name: string | null;
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
  original_block_id: string;
  block_name: string;
  priority: number;
  scheduled: boolean;
  requested_hours: number;
  scheduled_start_mjd: number | null;
  scheduled_stop_mjd: number | null;
}

export interface CompareStats {
  scheduled_count: number;
  unscheduled_count: number;
  /** Sum of scheduled priorities. Rendered in the UI as "Cumulative priority". */
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

export interface CompareDiffBlock {
  original_block_id: string;
  block_name: string;
  priority: number;
  requested_hours: number;
  current_scheduling_block_id: string | null;
  comparison_scheduling_block_id: string | null;
  current_scheduled_start_mjd: number | null;
  current_scheduled_stop_mjd: number | null;
  comparison_scheduled_start_mjd: number | null;
  comparison_scheduled_stop_mjd: number | null;
}

export interface RetimedBlockChange {
  original_block_id: string;
  block_name: string;
  priority: number;
  requested_hours: number;
  current_scheduling_block_id: string | null;
  comparison_scheduling_block_id: string | null;
  current_scheduled_start_mjd: number | null;
  current_scheduled_stop_mjd: number | null;
  comparison_scheduled_start_mjd: number | null;
  comparison_scheduled_stop_mjd: number | null;
  start_shift_hours: number;
  stop_shift_hours: number;
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
  scheduled_only_current: CompareDiffBlock[];
  scheduled_only_comparison: CompareDiffBlock[];
  only_in_current_blocks: CompareDiffBlock[];
  only_in_comparison_blocks: CompareDiffBlock[];
  retimed_blocks: RetimedBlockChange[];
  current_name: string;
  comparison_name: string;
}

// Visibility Map
export interface VisibilityBlockSummary {
  scheduling_block_id: number;
  original_block_id: string;
  block_name: string;
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
  scheduled?: boolean;
}

// =============================================================================
// Error Types
// =============================================================================

export interface ApiError {
  code: string;
  message: string;
  details?: string;
}
