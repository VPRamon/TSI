/**
 * Shared analytics primitives for comparison & algorithm-analysis panels.
 */
export {
  extractDimensions,
  readDimension,
  type Dimension,
  type DimensionKind,
  type DimensionSet,
} from './dimensions';
export {
  useConfigFilters,
  type ConfigFilterState,
  type CategoricalSelection,
  type UseConfigFiltersOptions,
} from './useConfigFilters';
export {
  fingerprintInsights,
  groupEquivalentSchedules,
  collapseToRepresentatives,
  type EquivalenceGroup,
  type EquivalenceResult,
} from './equivalence';
export {
  ALL_METRICS,
  DEFAULT_COMPARISON_METRIC_KEYS,
  METRICS_BY_KEY,
  METRIC_CUMULATIVE_PRIORITY,
  METRIC_GAP_COUNT,
  METRIC_IDLE_FRACTION,
  METRIC_MEAN_PRIORITY,
  METRIC_PRIORITY_CAPTURE,
  METRIC_SCHEDULED_COUNT,
  METRIC_SCHEDULED_HOURS,
  METRIC_SCHEDULING_RATE,
  getMetric,
  type MetricDirection,
  type MetricSpec,
} from './metrics';
export { CategoricalFilterGroup } from './CategoricalFilterGroup';
