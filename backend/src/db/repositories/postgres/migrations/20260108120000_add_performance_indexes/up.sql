-- Add performance indexes for JSONB columns and common query patterns
-- Migration: 20260108120000_add_performance_indexes

-- GIN indexes for JSONB columns to speed up containment queries
-- These are useful when filtering by period values or checking array contents
CREATE INDEX IF NOT EXISTS schedule_blocks_visibility_periods_gin_idx 
    ON schedule_blocks USING gin (visibility_periods_json);

CREATE INDEX IF NOT EXISTS schedule_blocks_scheduled_periods_gin_idx 
    ON schedule_blocks USING gin (scheduled_periods_json);

CREATE INDEX IF NOT EXISTS schedules_dark_periods_gin_idx 
    ON schedules USING gin (dark_periods_json);

-- Composite indexes for common query patterns

-- Sky map queries: join blocks with analytics, filter by schedule
CREATE INDEX IF NOT EXISTS schedule_blocks_skymap_idx 
    ON schedule_blocks (schedule_id, scheduling_block_id, priority, target_ra_deg, target_dec_deg);

-- Distribution queries: analytics filtering
CREATE INDEX IF NOT EXISTS schedule_block_analytics_distribution_idx 
    ON schedule_block_analytics (schedule_id, priority_bucket, scheduled);

-- Timeline queries: scheduled blocks with time ordering
CREATE INDEX IF NOT EXISTS schedule_block_analytics_timeline_idx 
    ON schedule_block_analytics (schedule_id, scheduled, scheduled_start_mjd)
    WHERE scheduled = true;

-- Validation queries: filter by status and schedule
CREATE INDEX IF NOT EXISTS schedule_validation_results_composite_idx 
    ON schedule_validation_results (schedule_id, scheduling_block_id, status);

-- Priority-based filtering (common in histogram queries)
CREATE INDEX IF NOT EXISTS schedule_blocks_priority_idx 
    ON schedule_blocks (schedule_id, priority);

-- Add partial index for unscheduled blocks (common query pattern)
CREATE INDEX IF NOT EXISTS schedule_block_analytics_unscheduled_idx 
    ON schedule_block_analytics (schedule_id, priority_bucket)
    WHERE scheduled = false;

-- Add partial index for impossible blocks (validation filtering)
CREATE INDEX IF NOT EXISTS schedule_block_analytics_impossible_idx 
    ON schedule_block_analytics (schedule_id)
    WHERE validation_impossible = true;
