-- ============================================================================
-- TSI Analytics Table Migration
-- Version: 001
-- Description: Creates the analytics schema and schedule_blocks_analytics table
-- ============================================================================

-- Create analytics schema if it doesn't exist
IF NOT EXISTS (SELECT * FROM sys.schemas WHERE name = 'analytics')
BEGIN
    EXEC('CREATE SCHEMA analytics');
END
GO

-- Drop existing table if it exists (for clean migration)
IF OBJECT_ID('analytics.schedule_blocks_analytics', 'U') IS NOT NULL
BEGIN
    DROP TABLE analytics.schedule_blocks_analytics;
END
GO

-- ============================================================================
-- Main Analytics Table: schedule_blocks_analytics
-- 
-- This table denormalizes and pre-computes data from:
-- - dbo.schedules
-- - dbo.schedule_scheduling_blocks
-- - dbo.scheduling_blocks
-- - dbo.targets
-- - dbo.constraints
-- - dbo.altitude_constraints
-- - dbo.azimuth_constraints
-- ============================================================================

CREATE TABLE analytics.schedule_blocks_analytics (
    -- Identity
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    
    -- Foreign Keys (not enforced to allow flexible ETL)
    schedule_id BIGINT NOT NULL,
    scheduling_block_id BIGINT NOT NULL,
    
    -- Target Information (denormalized from dbo.targets)
    target_ra_deg FLOAT NOT NULL,
    target_dec_deg FLOAT NOT NULL,
    
    -- Block Core Fields (from dbo.scheduling_blocks)
    priority FLOAT NOT NULL,
    priority_bucket TINYINT NOT NULL,  -- Pre-computed: 1=Low, 2=Med-Low, 3=Med-High, 4=High
    requested_duration_sec INT NOT NULL,
    min_observation_sec INT NOT NULL,
    
    -- Constraints (denormalized from dbo.constraints/altitude_constraints)
    min_altitude_deg FLOAT NULL,
    max_altitude_deg FLOAT NULL,
    min_azimuth_deg FLOAT NULL,
    max_azimuth_deg FLOAT NULL,
    
    -- Constraint Time Window (from dbo.constraints)
    constraint_start_mjd FLOAT NULL,
    constraint_stop_mjd FLOAT NULL,
    
    -- Scheduling Status (from dbo.schedule_scheduling_blocks)
    is_scheduled BIT NOT NULL DEFAULT 0,
    scheduled_start_mjd FLOAT NULL,
    scheduled_stop_mjd FLOAT NULL,
    
    -- Pre-computed Visibility Metrics (extracted from visibility_periods_json)
    total_visibility_hours FLOAT NOT NULL DEFAULT 0.0,
    visibility_period_count INT NOT NULL DEFAULT 0,
    
    -- Metadata
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE(),
    
    -- Computed columns (for convenience)
    requested_hours AS (CAST(requested_duration_sec AS FLOAT) / 3600.0) PERSISTED,
    elevation_range_deg AS (COALESCE(max_altitude_deg, 90.0) - COALESCE(min_altitude_deg, 0.0)) PERSISTED,
    scheduled_duration_sec AS (
        CASE WHEN scheduled_start_mjd IS NOT NULL AND scheduled_stop_mjd IS NOT NULL 
        THEN (scheduled_stop_mjd - scheduled_start_mjd) * 86400.0 
        ELSE NULL END
    ) PERSISTED,
    is_impossible AS (CASE WHEN total_visibility_hours = 0 THEN 1 ELSE 0 END) PERSISTED
);
GO

-- ============================================================================
-- Indexes for common query patterns
-- ============================================================================

-- Primary access pattern: all blocks for a schedule
CREATE INDEX IX_analytics_schedule_id 
    ON analytics.schedule_blocks_analytics (schedule_id) 
    INCLUDE (scheduling_block_id, priority, target_ra_deg, target_dec_deg, is_scheduled);

-- Sky Map queries: by schedule and coordinates
CREATE INDEX IX_analytics_sky_map 
    ON analytics.schedule_blocks_analytics (schedule_id, priority_bucket)
    INCLUDE (target_ra_deg, target_dec_deg, scheduled_start_mjd, scheduled_stop_mjd);

-- Distribution queries: by schedule and priority
CREATE INDEX IX_analytics_distribution 
    ON analytics.schedule_blocks_analytics (schedule_id, is_impossible)
    INCLUDE (priority, total_visibility_hours, requested_hours, elevation_range_deg, is_scheduled);

-- Scheduled blocks: for timeline and filtering
CREATE INDEX IX_analytics_scheduled 
    ON analytics.schedule_blocks_analytics (schedule_id, is_scheduled)
    WHERE is_scheduled = 1;

-- Impossible blocks: for filtering
CREATE INDEX IX_analytics_impossible 
    ON analytics.schedule_blocks_analytics (schedule_id, is_impossible)
    WHERE is_impossible = 1;

GO

-- ============================================================================
-- Stored Procedure: Populate analytics for a single schedule
-- This procedure is called by the Rust backend after schedule upload
-- ============================================================================

CREATE OR ALTER PROCEDURE analytics.sp_populate_schedule_analytics
    @schedule_id BIGINT
AS
BEGIN
    SET NOCOUNT ON;
    
    -- Delete existing analytics for this schedule (idempotent operation)
    DELETE FROM analytics.schedule_blocks_analytics 
    WHERE schedule_id = @schedule_id;
    
    -- Compute priority range for bucket calculation
    DECLARE @priority_min FLOAT, @priority_max FLOAT, @priority_range FLOAT;
    
    SELECT 
        @priority_min = MIN(sb.priority),
        @priority_max = MAX(sb.priority)
    FROM dbo.schedule_scheduling_blocks ssb
    JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
    WHERE ssb.schedule_id = @schedule_id;
    
    SET @priority_range = NULLIF(@priority_max - @priority_min, 0);
    
    -- Insert denormalized data with pre-computed fields
    INSERT INTO analytics.schedule_blocks_analytics (
        schedule_id,
        scheduling_block_id,
        target_ra_deg,
        target_dec_deg,
        priority,
        priority_bucket,
        requested_duration_sec,
        min_observation_sec,
        min_altitude_deg,
        max_altitude_deg,
        min_azimuth_deg,
        max_azimuth_deg,
        constraint_start_mjd,
        constraint_stop_mjd,
        is_scheduled,
        scheduled_start_mjd,
        scheduled_stop_mjd,
        total_visibility_hours,
        visibility_period_count
    )
    SELECT 
        ssb.schedule_id,
        sb.scheduling_block_id,
        t.ra_deg,
        t.dec_deg,
        sb.priority,
        -- Priority bucket: 1-4 based on quartiles
        CASE 
            WHEN @priority_range IS NULL THEN 2  -- Single value = medium
            WHEN sb.priority >= @priority_min + 0.75 * @priority_range THEN 4  -- High
            WHEN sb.priority >= @priority_min + 0.50 * @priority_range THEN 3  -- Medium-High
            WHEN sb.priority >= @priority_min + 0.25 * @priority_range THEN 2  -- Medium-Low
            ELSE 1  -- Low
        END,
        sb.requested_duration_sec,
        sb.min_observation_sec,
        ac.min_alt_deg,
        ac.max_alt_deg,
        azc.min_az_deg,
        azc.max_az_deg,
        c.start_time_mjd as constraint_start_mjd,
        c.stop_time_mjd as constraint_stop_mjd,
        CASE WHEN ssb.start_time_mjd IS NOT NULL THEN 1 ELSE 0 END,
        ssb.start_time_mjd,
        ssb.stop_time_mjd,
        -- Total visibility hours: computed from JSON
        COALESCE(vis.total_hours, 0.0),
        COALESCE(vis.period_count, 0)
    FROM dbo.schedule_scheduling_blocks ssb
    JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
    JOIN dbo.targets t ON sb.target_id = t.target_id
    LEFT JOIN dbo.constraints c ON sb.constraints_id = c.constraints_id
    LEFT JOIN dbo.altitude_constraints ac ON c.altitude_constraints_id = ac.altitude_constraints_id
    LEFT JOIN dbo.azimuth_constraints azc ON c.azimuth_constraints_id = azc.azimuth_constraints_id
    -- Cross apply to parse visibility JSON
    CROSS APPLY (
        SELECT 
            SUM(
                CAST(
                    JSON_VALUE(period.value, '$.stop') AS FLOAT
                ) - CAST(
                    JSON_VALUE(period.value, '$.start') AS FLOAT
                )
            ) * 24.0 AS total_hours,
            COUNT(*) AS period_count
        FROM OPENJSON(sb.visibility_periods_json) AS period
        WHERE sb.visibility_periods_json IS NOT NULL
    ) AS vis
    WHERE ssb.schedule_id = @schedule_id;
    
    SELECT @@ROWCOUNT AS rows_inserted;
END
GO

-- ============================================================================
-- Stored Procedure: Delete analytics for a schedule (cleanup)
-- ============================================================================

CREATE OR ALTER PROCEDURE analytics.sp_delete_schedule_analytics
    @schedule_id BIGINT
AS
BEGIN
    SET NOCOUNT ON;
    
    DELETE FROM analytics.schedule_blocks_analytics 
    WHERE schedule_id = @schedule_id;
    
    SELECT @@ROWCOUNT AS rows_deleted;
END
GO

-- ============================================================================
-- View: Convenient access to analytics data with additional computed fields
-- ============================================================================

CREATE OR ALTER VIEW analytics.v_schedule_blocks_analytics AS
SELECT 
    a.*,
    -- Priority bucket labels
    CASE a.priority_bucket
        WHEN 1 THEN 'Low'
        WHEN 2 THEN 'Medium-Low'
        WHEN 3 THEN 'Medium-High'
        WHEN 4 THEN 'High'
    END AS priority_bucket_label
FROM analytics.schedule_blocks_analytics a;
GO

PRINT 'Analytics table migration completed successfully.';
GO
