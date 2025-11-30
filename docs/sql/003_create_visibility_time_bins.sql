-- ============================================================================
-- TSI Analytics Visibility Time Bins Migration - Phase 3
-- Version: 003
-- Description: Creates pre-computed visibility time bins for fast histogram rendering
-- ============================================================================

-- ============================================================================
-- Table: schedule_visibility_time_bins
-- 
-- Pre-computed visibility counts per time bin.
-- Each row represents one time bin containing the set of block IDs visible in that bin.
-- 
-- Design rationale:
-- - Uses fixed 15-minute base bins to balance storage vs. aggregation flexibility
-- - Stores visible_block_ids as a comma-separated list to enable re-aggregation
-- - Stores priority-filtered counts to support common query patterns
-- - Can aggregate multiple base bins for larger display bins at query time
-- 
-- Used by: Visibility Map page for fast histogram rendering
-- ============================================================================

IF OBJECT_ID('analytics.schedule_visibility_time_bins', 'U') IS NOT NULL
BEGIN
    DROP TABLE analytics.schedule_visibility_time_bins;
END
GO

CREATE TABLE analytics.schedule_visibility_time_bins (
    -- Identity
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_id BIGINT NOT NULL,
    
    -- Time Bin Definition (Unix timestamps)
    bin_start_unix BIGINT NOT NULL,      -- Start of bin (inclusive)
    bin_end_unix BIGINT NOT NULL,        -- End of bin (exclusive)
    bin_index INT NOT NULL,              -- 0-based index within schedule
    
    -- Pre-computed Counts (for ALL blocks, regardless of priority)
    total_visible_count INT NOT NULL DEFAULT 0,
    
    -- Pre-computed Counts by Priority Quartile (for fast filtering)
    -- Priority quartiles are computed during ETL based on schedule's priority range
    priority_q1_count INT NOT NULL DEFAULT 0,  -- Low priority (bottom 25%)
    priority_q2_count INT NOT NULL DEFAULT 0,  -- Medium-Low (25-50%)
    priority_q3_count INT NOT NULL DEFAULT 0,  -- Medium-High (50-75%)
    priority_q4_count INT NOT NULL DEFAULT 0,  -- High priority (top 25%)
    
    -- Scheduled vs Unscheduled counts
    scheduled_visible_count INT NOT NULL DEFAULT 0,
    unscheduled_visible_count INT NOT NULL DEFAULT 0,
    
    -- Block IDs visible in this bin (comma-separated for re-aggregation)
    -- NULL if count is 0 to save space
    visible_block_ids VARCHAR(MAX) NULL,
    
    -- Metadata
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE(),
    
    -- Composite unique constraint
    CONSTRAINT UQ_visibility_time_bin UNIQUE (schedule_id, bin_index)
);
GO

-- Primary index for schedule-based queries
CREATE INDEX IX_visibility_time_bins_schedule 
    ON analytics.schedule_visibility_time_bins (schedule_id, bin_start_unix)
    INCLUDE (total_visible_count, scheduled_visible_count);
GO

-- Index for time-range queries
CREATE INDEX IX_visibility_time_bins_time_range
    ON analytics.schedule_visibility_time_bins (schedule_id, bin_start_unix, bin_end_unix)
    INCLUDE (total_visible_count, priority_q1_count, priority_q2_count, priority_q3_count, priority_q4_count);
GO

-- ============================================================================
-- Table: schedule_visibility_metadata
-- 
-- Metadata about the visibility time bins for each schedule.
-- Stores parameters used during ETL for validation and queries.
-- ============================================================================

IF OBJECT_ID('analytics.schedule_visibility_metadata', 'U') IS NOT NULL
BEGIN
    DROP TABLE analytics.schedule_visibility_metadata;
END
GO

CREATE TABLE analytics.schedule_visibility_metadata (
    -- Identity
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_id BIGINT NOT NULL UNIQUE,
    
    -- Time Range (Unix timestamps)
    time_range_start_unix BIGINT NOT NULL,
    time_range_end_unix BIGINT NOT NULL,
    
    -- Bin Configuration
    bin_duration_seconds INT NOT NULL DEFAULT 900,  -- 15 minutes default
    total_bins INT NOT NULL DEFAULT 0,
    
    -- Block Counts
    total_blocks INT NOT NULL DEFAULT 0,
    blocks_with_visibility INT NOT NULL DEFAULT 0,
    
    -- Priority Range (for quartile calculations)
    priority_min FLOAT NULL,
    priority_max FLOAT NULL,
    
    -- Statistics
    max_visible_in_bin INT NOT NULL DEFAULT 0,
    mean_visible_per_bin FLOAT NULL,
    
    -- Metadata
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE(),
    etl_duration_ms INT NULL
);
GO

CREATE INDEX IX_visibility_metadata_schedule 
    ON analytics.schedule_visibility_metadata (schedule_id);
GO

-- ============================================================================
-- Stored Procedure: Aggregate visibility bins to larger intervals
-- 
-- This procedure takes pre-computed 15-minute bins and aggregates them
-- into larger bins for display. Used by the Visibility Map page.
-- ============================================================================

CREATE OR ALTER PROCEDURE analytics.sp_aggregate_visibility_bins
    @schedule_id BIGINT,
    @start_unix BIGINT,
    @end_unix BIGINT,
    @target_bin_duration_seconds INT = 3600,  -- Default 1 hour
    @priority_min INT = NULL,                  -- Optional priority filter (inclusive)
    @priority_max INT = NULL                   -- Optional priority filter (inclusive)
AS
BEGIN
    SET NOCOUNT ON;
    
    -- Get metadata for this schedule
    DECLARE @base_bin_duration INT;
    DECLARE @priority_range_min FLOAT;
    DECLARE @priority_range_max FLOAT;
    
    SELECT 
        @base_bin_duration = bin_duration_seconds,
        @priority_range_min = priority_min,
        @priority_range_max = priority_max
    FROM analytics.schedule_visibility_metadata
    WHERE schedule_id = @schedule_id;
    
    IF @base_bin_duration IS NULL
    BEGIN
        -- No pre-computed data, return empty result
        SELECT 
            CAST(0 AS BIGINT) AS bin_start_unix,
            CAST(0 AS BIGINT) AS bin_end_unix,
            CAST(0 AS INT) AS visible_count
        WHERE 1 = 0;
        RETURN;
    END
    
    -- Calculate aggregation factor (how many base bins per target bin)
    DECLARE @agg_factor INT = @target_bin_duration_seconds / @base_bin_duration;
    IF @agg_factor < 1 SET @agg_factor = 1;
    
    -- Generate target bins and aggregate
    WITH TargetBins AS (
        SELECT 
            bin_start_unix,
            bin_end_unix,
            -- Calculate which target bin this base bin belongs to
            (bin_start_unix - @start_unix) / @target_bin_duration_seconds AS target_bin_index,
            -- Select count based on priority filter
            CASE 
                WHEN @priority_min IS NULL AND @priority_max IS NULL THEN total_visible_count
                -- For simplicity, if filtering by priority, sum the quartile counts that fall within range
                -- This is an approximation; for exact counts, would need to parse visible_block_ids
                ELSE total_visible_count  -- TODO: Implement exact priority filtering if needed
            END AS visible_count
        FROM analytics.schedule_visibility_time_bins
        WHERE schedule_id = @schedule_id
          AND bin_start_unix >= @start_unix
          AND bin_end_unix <= @end_unix
    )
    SELECT 
        @start_unix + (target_bin_index * @target_bin_duration_seconds) AS bin_start_unix,
        @start_unix + ((target_bin_index + 1) * @target_bin_duration_seconds) AS bin_end_unix,
        -- Use MAX instead of SUM because blocks can span multiple base bins
        -- and we want unique blocks per target bin
        MAX(visible_count) AS visible_count
    FROM TargetBins
    GROUP BY target_bin_index
    ORDER BY target_bin_index;
END
GO

-- ============================================================================
-- View: Fast visibility histogram query (alternative to stored procedure)
-- ============================================================================

CREATE OR ALTER VIEW analytics.vw_visibility_time_bins_summary AS
SELECT 
    vtb.schedule_id,
    vtb.bin_start_unix,
    vtb.bin_end_unix,
    vtb.total_visible_count,
    vtb.scheduled_visible_count,
    vtb.unscheduled_visible_count,
    vtb.priority_q1_count,
    vtb.priority_q2_count,
    vtb.priority_q3_count,
    vtb.priority_q4_count,
    vm.priority_min,
    vm.priority_max,
    vm.bin_duration_seconds
FROM analytics.schedule_visibility_time_bins vtb
JOIN analytics.schedule_visibility_metadata vm ON vtb.schedule_id = vm.schedule_id;
GO

PRINT 'Phase 3 visibility time bins tables created successfully';
GO
