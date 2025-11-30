-- ============================================================================
-- TSI Analytics Summary Tables Migration - Phase 2
-- Version: 002
-- Description: Creates summary-level analytics tables for Insights and Trends
-- ============================================================================

-- ============================================================================
-- Table 1: schedule_summary_analytics
-- 
-- One row per schedule with pre-computed aggregate metrics.
-- Used by: Insights page for key metrics and analytics
-- ============================================================================

IF OBJECT_ID('analytics.schedule_summary_analytics', 'U') IS NOT NULL
BEGIN
    DROP TABLE analytics.schedule_summary_analytics;
END
GO

CREATE TABLE analytics.schedule_summary_analytics (
    -- Identity
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_id BIGINT NOT NULL UNIQUE,
    
    -- Block Counts
    total_blocks INT NOT NULL DEFAULT 0,
    scheduled_blocks INT NOT NULL DEFAULT 0,
    unscheduled_blocks INT NOT NULL DEFAULT 0,
    impossible_blocks INT NOT NULL DEFAULT 0,  -- zero visibility
    
    -- Scheduling Rate
    scheduling_rate FLOAT NOT NULL DEFAULT 0.0,
    
    -- Priority Statistics (all blocks)
    priority_min FLOAT NULL,
    priority_max FLOAT NULL,
    priority_mean FLOAT NULL,
    priority_median FLOAT NULL,
    priority_std FLOAT NULL,
    
    -- Priority Statistics (scheduled blocks only)
    priority_scheduled_mean FLOAT NULL,
    priority_scheduled_median FLOAT NULL,
    
    -- Priority Statistics (unscheduled blocks only)
    priority_unscheduled_mean FLOAT NULL,
    priority_unscheduled_median FLOAT NULL,
    
    -- Visibility Statistics (hours)
    visibility_total_hours FLOAT NOT NULL DEFAULT 0.0,
    visibility_mean_hours FLOAT NULL,
    visibility_median_hours FLOAT NULL,
    visibility_min_hours FLOAT NULL,
    visibility_max_hours FLOAT NULL,
    
    -- Requested Time Statistics (hours)
    requested_total_hours FLOAT NOT NULL DEFAULT 0.0,
    requested_mean_hours FLOAT NULL,
    requested_median_hours FLOAT NULL,
    requested_min_hours FLOAT NULL,
    requested_max_hours FLOAT NULL,
    
    -- Scheduled Time Statistics (hours)
    scheduled_total_hours FLOAT NOT NULL DEFAULT 0.0,
    scheduled_mean_hours FLOAT NULL,
    
    -- Elevation Statistics (degrees)
    elevation_mean_deg FLOAT NULL,
    elevation_min_deg FLOAT NULL,
    elevation_max_deg FLOAT NULL,
    
    -- Coordinate Ranges
    ra_min FLOAT NULL,
    ra_max FLOAT NULL,
    dec_min FLOAT NULL,
    dec_max FLOAT NULL,
    
    -- Time Range (MJD)
    scheduled_time_min_mjd FLOAT NULL,
    scheduled_time_max_mjd FLOAT NULL,
    
    -- Pre-computed Correlations (Spearman)
    corr_priority_visibility FLOAT NULL,
    corr_priority_requested FLOAT NULL,
    corr_visibility_requested FLOAT NULL,
    corr_priority_elevation FLOAT NULL,
    
    -- Conflict Count
    conflict_count INT NOT NULL DEFAULT 0,
    
    -- Metadata
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE(),
    updated_at DATETIME2 NOT NULL DEFAULT GETUTCDATE()
);
GO

-- Index for fast lookup by schedule_id
CREATE INDEX IX_schedule_summary_schedule_id 
    ON analytics.schedule_summary_analytics (schedule_id);
GO

-- ============================================================================
-- Table 2: schedule_priority_rates
-- 
-- Per-schedule, per-priority bin aggregates for scheduling rate analysis.
-- Used by: Trends page for empirical rate charts and priority analysis
-- ============================================================================

IF OBJECT_ID('analytics.schedule_priority_rates', 'U') IS NOT NULL
BEGIN
    DROP TABLE analytics.schedule_priority_rates;
END
GO

CREATE TABLE analytics.schedule_priority_rates (
    -- Identity
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_id BIGINT NOT NULL,
    
    -- Priority Bin (integer priority value)
    priority_value INT NOT NULL,
    
    -- Counts
    total_count INT NOT NULL DEFAULT 0,
    scheduled_count INT NOT NULL DEFAULT 0,
    unscheduled_count INT NOT NULL DEFAULT 0,
    impossible_count INT NOT NULL DEFAULT 0,
    
    -- Scheduling Rate for this priority
    scheduling_rate FLOAT NOT NULL DEFAULT 0.0,
    
    -- Visibility Statistics for this priority (hours)
    visibility_mean_hours FLOAT NULL,
    visibility_total_hours FLOAT NOT NULL DEFAULT 0.0,
    
    -- Requested Time Statistics for this priority (hours)
    requested_mean_hours FLOAT NULL,
    requested_total_hours FLOAT NOT NULL DEFAULT 0.0,
    
    -- Metadata
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE(),
    
    -- Composite unique constraint
    CONSTRAINT UQ_schedule_priority UNIQUE (schedule_id, priority_value)
);
GO

-- Index for fast lookup by schedule_id
CREATE INDEX IX_priority_rates_schedule_id 
    ON analytics.schedule_priority_rates (schedule_id);

-- Index for priority analysis
CREATE INDEX IX_priority_rates_priority 
    ON analytics.schedule_priority_rates (schedule_id, priority_value)
    INCLUDE (scheduling_rate, total_count, scheduled_count);
GO

-- ============================================================================
-- Table 3: schedule_visibility_bins
-- 
-- Per-schedule, visibility hour bins for trend analysis.
-- Used by: Trends page for visibility-based rate charts
-- ============================================================================

IF OBJECT_ID('analytics.schedule_visibility_bins', 'U') IS NOT NULL
BEGIN
    DROP TABLE analytics.schedule_visibility_bins;
END
GO

CREATE TABLE analytics.schedule_visibility_bins (
    -- Identity
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_id BIGINT NOT NULL,
    
    -- Bin Definition
    bin_index INT NOT NULL,  -- 0-based bin index
    bin_min_hours FLOAT NOT NULL,
    bin_max_hours FLOAT NOT NULL,
    bin_mid_hours FLOAT NOT NULL,  -- midpoint for plotting
    
    -- Counts
    total_count INT NOT NULL DEFAULT 0,
    scheduled_count INT NOT NULL DEFAULT 0,
    
    -- Scheduling Rate for this bin
    scheduling_rate FLOAT NOT NULL DEFAULT 0.0,
    
    -- Mean priority in this bin
    priority_mean FLOAT NULL,
    
    -- Metadata
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE(),
    
    -- Composite unique constraint
    CONSTRAINT UQ_schedule_visibility_bin UNIQUE (schedule_id, bin_index)
);
GO

CREATE INDEX IX_visibility_bins_schedule_id 
    ON analytics.schedule_visibility_bins (schedule_id);
GO

-- ============================================================================
-- Table 4: schedule_heatmap_bins
-- 
-- 2D bins for visibility vs requested time heatmap.
-- Used by: Trends page for heatmap visualization
-- ============================================================================

IF OBJECT_ID('analytics.schedule_heatmap_bins', 'U') IS NOT NULL
BEGIN
    DROP TABLE analytics.schedule_heatmap_bins;
END
GO

CREATE TABLE analytics.schedule_heatmap_bins (
    -- Identity
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_id BIGINT NOT NULL,
    
    -- Bin Indices
    visibility_bin_index INT NOT NULL,
    time_bin_index INT NOT NULL,
    
    -- Bin Midpoints (for plotting)
    visibility_mid_hours FLOAT NOT NULL,
    time_mid_hours FLOAT NOT NULL,
    
    -- Counts
    total_count INT NOT NULL DEFAULT 0,
    scheduled_count INT NOT NULL DEFAULT 0,
    
    -- Scheduling Rate
    scheduling_rate FLOAT NOT NULL DEFAULT 0.0,
    
    -- Metadata
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE(),
    
    -- Composite unique constraint
    CONSTRAINT UQ_schedule_heatmap_bin UNIQUE (schedule_id, visibility_bin_index, time_bin_index)
);
GO

CREATE INDEX IX_heatmap_bins_schedule_id 
    ON analytics.schedule_heatmap_bins (schedule_id);
GO

-- ============================================================================
-- Stored Procedure: Populate all summary analytics for a schedule
-- ============================================================================

CREATE OR ALTER PROCEDURE analytics.sp_populate_summary_analytics
    @schedule_id BIGINT
AS
BEGIN
    SET NOCOUNT ON;
    
    -- Delete existing summary for this schedule
    DELETE FROM analytics.schedule_summary_analytics WHERE schedule_id = @schedule_id;
    DELETE FROM analytics.schedule_priority_rates WHERE schedule_id = @schedule_id;
    DELETE FROM analytics.schedule_visibility_bins WHERE schedule_id = @schedule_id;
    DELETE FROM analytics.schedule_heatmap_bins WHERE schedule_id = @schedule_id;
    
    -- Check if we have analytics data for this schedule
    IF NOT EXISTS (SELECT 1 FROM analytics.schedule_blocks_analytics WHERE schedule_id = @schedule_id)
    BEGIN
        PRINT 'No block-level analytics found for schedule_id=' + CAST(@schedule_id AS VARCHAR);
        RETURN;
    END
    
    -- Insert schedule summary
    INSERT INTO analytics.schedule_summary_analytics (
        schedule_id,
        total_blocks,
        scheduled_blocks,
        unscheduled_blocks,
        impossible_blocks,
        scheduling_rate,
        priority_min,
        priority_max,
        priority_mean,
        visibility_total_hours,
        visibility_mean_hours,
        visibility_min_hours,
        visibility_max_hours,
        requested_total_hours,
        requested_mean_hours,
        requested_min_hours,
        requested_max_hours,
        scheduled_total_hours,
        ra_min,
        ra_max,
        dec_min,
        dec_max,
        scheduled_time_min_mjd,
        scheduled_time_max_mjd
    )
    SELECT
        @schedule_id,
        COUNT(*) as total_blocks,
        SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END) as scheduled_blocks,
        SUM(CASE WHEN is_scheduled = 0 THEN 1 ELSE 0 END) as unscheduled_blocks,
        SUM(CASE WHEN is_impossible = 1 THEN 1 ELSE 0 END) as impossible_blocks,
        CAST(SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END) AS FLOAT) / NULLIF(COUNT(*), 0) as scheduling_rate,
        MIN(priority),
        MAX(priority),
        AVG(priority),
        SUM(total_visibility_hours),
        AVG(total_visibility_hours),
        MIN(total_visibility_hours),
        MAX(total_visibility_hours),
        SUM(requested_hours),
        AVG(requested_hours),
        MIN(requested_hours),
        MAX(requested_hours),
        SUM(CASE WHEN is_scheduled = 1 THEN scheduled_duration_sec / 3600.0 ELSE 0 END),
        MIN(target_ra_deg),
        MAX(target_ra_deg),
        MIN(target_dec_deg),
        MAX(target_dec_deg),
        MIN(scheduled_start_mjd),
        MAX(scheduled_stop_mjd)
    FROM analytics.schedule_blocks_analytics
    WHERE schedule_id = @schedule_id;
    
    -- Update priority statistics for scheduled/unscheduled
    UPDATE s SET
        priority_scheduled_mean = sub.scheduled_mean,
        priority_unscheduled_mean = sub.unscheduled_mean
    FROM analytics.schedule_summary_analytics s
    CROSS APPLY (
        SELECT
            AVG(CASE WHEN is_scheduled = 1 THEN priority END) as scheduled_mean,
            AVG(CASE WHEN is_scheduled = 0 THEN priority END) as unscheduled_mean
        FROM analytics.schedule_blocks_analytics
        WHERE schedule_id = @schedule_id
    ) sub
    WHERE s.schedule_id = @schedule_id;
    
    -- Insert priority rates
    INSERT INTO analytics.schedule_priority_rates (
        schedule_id,
        priority_value,
        total_count,
        scheduled_count,
        unscheduled_count,
        impossible_count,
        scheduling_rate,
        visibility_mean_hours,
        visibility_total_hours,
        requested_mean_hours,
        requested_total_hours
    )
    SELECT
        @schedule_id,
        CAST(ROUND(priority, 0) AS INT) as priority_value,
        COUNT(*) as total_count,
        SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END) as scheduled_count,
        SUM(CASE WHEN is_scheduled = 0 THEN 1 ELSE 0 END) as unscheduled_count,
        SUM(CASE WHEN is_impossible = 1 THEN 1 ELSE 0 END) as impossible_count,
        CAST(SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END) AS FLOAT) / NULLIF(COUNT(*), 0) as scheduling_rate,
        AVG(total_visibility_hours),
        SUM(total_visibility_hours),
        AVG(requested_hours),
        SUM(requested_hours)
    FROM analytics.schedule_blocks_analytics
    WHERE schedule_id = @schedule_id
    GROUP BY CAST(ROUND(priority, 0) AS INT);
    
    SELECT @@ROWCOUNT AS priority_rates_inserted;
END
GO

PRINT 'Phase 2 summary tables migration completed successfully.';
GO
