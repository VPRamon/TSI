-- Rollback performance indexes
-- Migration: 20260108120000_add_performance_indexes

-- Drop partial indexes
DROP INDEX IF EXISTS schedule_block_analytics_impossible_idx;
DROP INDEX IF EXISTS schedule_block_analytics_unscheduled_idx;

-- Drop composite indexes
DROP INDEX IF EXISTS schedule_blocks_priority_idx;
DROP INDEX IF EXISTS schedule_validation_results_composite_idx;
DROP INDEX IF EXISTS schedule_block_analytics_timeline_idx;
DROP INDEX IF EXISTS schedule_block_analytics_distribution_idx;
DROP INDEX IF EXISTS schedule_blocks_skymap_idx;

-- Drop GIN indexes
DROP INDEX IF EXISTS schedules_dark_periods_gin_idx;
DROP INDEX IF EXISTS schedule_blocks_scheduled_periods_gin_idx;
DROP INDEX IF EXISTS schedule_blocks_visibility_periods_gin_idx;
