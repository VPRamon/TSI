-- ============================================================================
-- TSI Application - Complete Azure SQL Database Setup
-- ============================================================================
-- This script creates ALL tables required for the TSI application from scratch.
-- It includes both the base normalized schema and the analytics ETL tables.
--
-- Target: Azure SQL Database (compatible with SQL Server 2019+)
-- 
-- Usage:
--   sqlcmd -S <server>.database.windows.net -d <database> -U <user> -P <password> -i azure-setup-complete.sql
--
-- Or run in Azure Data Studio / SSMS / Azure Portal Query Editor
-- ============================================================================

SET NOCOUNT ON;
GO

PRINT '';
PRINT '============================================================================';
PRINT ' TSI Database Setup';
PRINT ' Started: ' + CONVERT(VARCHAR(23), GETDATE(), 121);
PRINT '============================================================================';
PRINT '';
GO

-- ============================================================================
-- PART 1: BASE SCHEMA (Normalized Tables)
-- ============================================================================

PRINT '=== PART 1: Creating Base Schema ===';
PRINT '';
GO

-- Drop tables in correct order (respecting FK constraints)
-- Must drop analytics tables FIRST since they reference dbo.schedules
IF OBJECT_ID('analytics.schedule_validation_results', 'U') IS NOT NULL 
    DROP TABLE analytics.schedule_validation_results;
IF OBJECT_ID('analytics.schedule_blocks_analytics', 'U') IS NOT NULL 
    DROP TABLE analytics.schedule_blocks_analytics;

-- Now drop base schema tables
IF OBJECT_ID('dbo.schedule_scheduling_blocks', 'U') IS NOT NULL 
    DROP TABLE dbo.schedule_scheduling_blocks;
IF OBJECT_ID('dbo.visibility_periods', 'U') IS NOT NULL 
    DROP TABLE dbo.visibility_periods;
IF OBJECT_ID('dbo.schedule_dark_periods', 'U') IS NOT NULL 
    DROP TABLE dbo.schedule_dark_periods;
IF OBJECT_ID('dbo.scheduling_blocks', 'U') IS NOT NULL 
    DROP TABLE dbo.scheduling_blocks;
IF OBJECT_ID('dbo.constraints', 'U') IS NOT NULL 
    DROP TABLE dbo.constraints;
IF OBJECT_ID('dbo.azimuth_constraints', 'U') IS NOT NULL 
    DROP TABLE dbo.azimuth_constraints;
IF OBJECT_ID('dbo.altitude_constraints', 'U') IS NOT NULL 
    DROP TABLE dbo.altitude_constraints;
IF OBJECT_ID('dbo.targets', 'U') IS NOT NULL 
    DROP TABLE dbo.targets;
IF OBJECT_ID('dbo.schedules', 'U') IS NOT NULL 
    DROP TABLE dbo.schedules;
GO

PRINT '  Dropped existing tables (if any)';
GO

-- Create tables in correct order (respecting FK dependencies)
CREATE TABLE dbo.schedules (
    schedule_id      BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_name    NVARCHAR(256) NOT NULL,
    upload_timestamp DATETIMEOFFSET(3) NOT NULL DEFAULT SYSUTCDATETIME(),
    checksum         NVARCHAR(64) NOT NULL,
    dark_periods_json NVARCHAR(MAX) NULL,
    CONSTRAINT UQ_schedules_checksum UNIQUE (checksum)
);
PRINT '  ✓ Created table: dbo.schedules';

CREATE TABLE dbo.targets (
    target_id       BIGINT IDENTITY(1,1) PRIMARY KEY,
    name            NVARCHAR(MAX) NOT NULL,
    ra_deg          FLOAT NOT NULL,
    dec_deg         FLOAT NOT NULL,
    ra_pm_masyr     FLOAT NOT NULL DEFAULT 0,
    dec_pm_masyr    FLOAT NOT NULL DEFAULT 0,
    equinox         FLOAT NOT NULL DEFAULT 2000.0,
    CONSTRAINT targets_unique_natural_flat UNIQUE (ra_deg, dec_deg, ra_pm_masyr, dec_pm_masyr, equinox),
    CONSTRAINT valid_ra_dec_flat CHECK (ra_deg >= 0 AND ra_deg < 360 AND dec_deg >= -90 AND dec_deg <= 90)
);
PRINT '  ✓ Created table: dbo.targets';

CREATE TABLE dbo.altitude_constraints (
    altitude_constraints_id BIGINT IDENTITY(1,1) PRIMARY KEY,
    min_alt_deg             FLOAT NOT NULL DEFAULT 0,
    max_alt_deg             FLOAT NOT NULL DEFAULT 90,
    CONSTRAINT altitude_constraints_range_chk_flat CHECK (min_alt_deg <= max_alt_deg),
    CONSTRAINT altitude_constraints_unique_flat UNIQUE (min_alt_deg, max_alt_deg)
);
PRINT '  ✓ Created table: dbo.altitude_constraints';

CREATE TABLE dbo.azimuth_constraints (
    azimuth_constraints_id BIGINT IDENTITY(1,1) PRIMARY KEY,
    min_az_deg             FLOAT NOT NULL DEFAULT 0,
    max_az_deg             FLOAT NOT NULL DEFAULT 360,
    CONSTRAINT azimuth_constraints_range_chk_flat CHECK (min_az_deg <= max_az_deg),
    CONSTRAINT azimuth_constraints_unique_flat UNIQUE (min_az_deg, max_az_deg)
);
PRINT '  ✓ Created table: dbo.azimuth_constraints';

CREATE TABLE dbo.constraints (
    constraints_id          BIGINT IDENTITY(1,1) PRIMARY KEY,
    start_time_mjd          FLOAT NULL,
    stop_time_mjd           FLOAT NULL,
    altitude_constraints_id BIGINT NULL REFERENCES dbo.altitude_constraints(altitude_constraints_id) ON DELETE SET NULL,
    azimuth_constraints_id  BIGINT NULL REFERENCES dbo.azimuth_constraints(azimuth_constraints_id) ON DELETE SET NULL,
    CONSTRAINT at_least_one_constraint_flat CHECK (start_time_mjd IS NOT NULL OR altitude_constraints_id IS NOT NULL OR azimuth_constraints_id IS NOT NULL),
    CONSTRAINT constraints_time_range_chk_flat CHECK (start_time_mjd IS NULL OR stop_time_mjd IS NULL OR start_time_mjd < stop_time_mjd),
    CONSTRAINT constraints_unique_combo_flat UNIQUE (start_time_mjd, stop_time_mjd, altitude_constraints_id, azimuth_constraints_id)
);
PRINT '  ✓ Created table: dbo.constraints';

CREATE TABLE dbo.scheduling_blocks (
    scheduling_block_id    BIGINT IDENTITY(1,1) PRIMARY KEY,
    original_block_id      NVARCHAR(256) NULL,  -- Original schedulingBlockId from JSON
    target_id              BIGINT NOT NULL REFERENCES dbo.targets(target_id),
    constraints_id         BIGINT NULL REFERENCES dbo.constraints(constraints_id),
    priority               FLOAT NOT NULL,
    min_observation_sec    INT NOT NULL,
    requested_duration_sec INT NOT NULL,
    visibility_periods_json NVARCHAR(MAX) NULL,
    CONSTRAINT valid_min_obs_req_dur_flat CHECK (min_observation_sec >= 0 AND requested_duration_sec >= 0 AND min_observation_sec <= requested_duration_sec)
);
PRINT '  ✓ Created table: dbo.scheduling_blocks';

CREATE TABLE dbo.schedule_scheduling_blocks (
    schedule_id          BIGINT NOT NULL,
    scheduling_block_id  BIGINT NOT NULL,
    start_time_mjd       FLOAT NULL,
    stop_time_mjd        FLOAT NULL,
    duration_sec         AS (CASE WHEN start_time_mjd IS NOT NULL AND stop_time_mjd IS NOT NULL AND stop_time_mjd > start_time_mjd THEN (stop_time_mjd - start_time_mjd) * 86400.0 ELSE 0 END) PERSISTED,
    CONSTRAINT PK_schedule_scheduling_blocks_flat PRIMARY KEY (schedule_id, scheduling_block_id),
    CONSTRAINT FK_ssb_schedules_flat FOREIGN KEY (schedule_id) REFERENCES dbo.schedules(schedule_id) ON DELETE CASCADE,
    CONSTRAINT FK_ssb_scheduling_blocks_flat FOREIGN KEY (scheduling_block_id) REFERENCES dbo.scheduling_blocks(scheduling_block_id) ON DELETE CASCADE,
    CONSTRAINT ssb_time_range_chk_flat CHECK (start_time_mjd IS NULL OR stop_time_mjd IS NULL OR start_time_mjd < stop_time_mjd)
);
PRINT '  ✓ Created table: dbo.schedule_scheduling_blocks';

CREATE TABLE dbo.visibility_periods (
    target_id       BIGINT NOT NULL REFERENCES dbo.targets(target_id) ON DELETE CASCADE,
    constraints_id  BIGINT NOT NULL REFERENCES dbo.constraints(constraints_id) ON DELETE CASCADE,
    start_time_mjd  FLOAT NOT NULL,
    stop_time_mjd   FLOAT NOT NULL,
    duration_sec    AS (CASE WHEN stop_time_mjd > start_time_mjd THEN (stop_time_mjd - start_time_mjd) * 86400.0 ELSE 0 END) PERSISTED,
    CONSTRAINT visibility_periods_pk_flat PRIMARY KEY (target_id, constraints_id, start_time_mjd, stop_time_mjd),
    CONSTRAINT visibility_periods_range_chk_flat CHECK (start_time_mjd < stop_time_mjd)
);
PRINT '  ✓ Created table: dbo.visibility_periods';

CREATE TABLE dbo.schedule_dark_periods (
    schedule_id     BIGINT NOT NULL REFERENCES dbo.schedules(schedule_id) ON DELETE CASCADE,
    start_time_mjd  FLOAT NOT NULL,
    stop_time_mjd   FLOAT NOT NULL,
    duration_sec    AS (CASE WHEN stop_time_mjd > start_time_mjd THEN (stop_time_mjd - start_time_mjd) * 86400.0 ELSE 0 END) PERSISTED,
    CONSTRAINT schedule_dark_periods_pk_flat PRIMARY KEY (schedule_id, start_time_mjd, stop_time_mjd),
    CONSTRAINT schedule_dark_periods_range_chk_flat CHECK (start_time_mjd < stop_time_mjd)
);
PRINT '  ✓ Created table: dbo.schedule_dark_periods';
GO

PRINT '';
PRINT '=== PART 2: Creating Analytics Schema (ETL) ===';
PRINT '';
GO

-- ============================================================================
-- PART 2: ANALYTICS SCHEMA (ETL Tables)
-- ============================================================================

-- Create analytics schema if it doesn't exist
IF NOT EXISTS (SELECT * FROM sys.schemas WHERE name = 'analytics')
BEGIN
    EXEC('CREATE SCHEMA analytics');
    PRINT '  ✓ Created schema: analytics';
END
ELSE
    PRINT '  ✓ Schema exists: analytics';
GO

-- Note: Analytics tables already dropped in Part 1 to respect FK dependencies

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
    original_block_id NVARCHAR(256) NULL,  -- Original schedulingBlockId from JSON
    
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
    
    -- Validation Results (set during ETL Phase 4 based on validation rules)
    validation_impossible BIT NULL DEFAULT NULL,  -- Set to 1 if validation marks block as impossible
    
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
PRINT '  ✓ Created table: analytics.schedule_blocks_analytics';
GO

-- ============================================================================
-- Indexes for common query patterns
-- ============================================================================

-- Primary access pattern: all blocks for a schedule
CREATE INDEX IX_analytics_schedule_id 
    ON analytics.schedule_blocks_analytics (schedule_id) 
    INCLUDE (scheduling_block_id, priority, target_ra_deg, target_dec_deg, is_scheduled);
PRINT '  ✓ Created index: IX_analytics_schedule_id';

-- Sky Map queries: by schedule and coordinates
CREATE INDEX IX_analytics_sky_map 
    ON analytics.schedule_blocks_analytics (schedule_id, priority_bucket)
    INCLUDE (target_ra_deg, target_dec_deg, scheduled_start_mjd, scheduled_stop_mjd);
PRINT '  ✓ Created index: IX_analytics_sky_map';

-- Distribution queries: by schedule and priority
CREATE INDEX IX_analytics_distribution 
    ON analytics.schedule_blocks_analytics (schedule_id, is_impossible)
    INCLUDE (priority, total_visibility_hours, requested_hours, elevation_range_deg, is_scheduled);
PRINT '  ✓ Created index: IX_analytics_distribution';

-- Scheduled blocks: for timeline and filtering
CREATE INDEX IX_analytics_scheduled 
    ON analytics.schedule_blocks_analytics (schedule_id, is_scheduled)
    WHERE scheduled_start_mjd IS NOT NULL;
PRINT '  ✓ Created index: IX_analytics_scheduled';

-- Impossible blocks: for filtering (where no visibility)
CREATE INDEX IX_analytics_impossible 
    ON analytics.schedule_blocks_analytics (schedule_id, total_visibility_hours)
    WHERE total_visibility_hours = 0;
PRINT '  ✓ Created index: IX_analytics_impossible';
GO

-- ============================================================================
-- Table: analytics.schedule_validation_results
-- Stores validation results for each scheduling block after ETL processing
-- ============================================================================

CREATE TABLE analytics.schedule_validation_results (
    id BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_id BIGINT NOT NULL,
    scheduling_block_id BIGINT NOT NULL,
    
    -- Validation status
    validation_status VARCHAR(20) NOT NULL,  -- 'valid', 'impossible', 'error', 'warning'
    
    -- Issue details
    issue_type VARCHAR(100) NULL,
    issue_category VARCHAR(50) NULL,  -- 'visibility', 'constraint', 'coordinate', 'priority', 'duration'
    criticality VARCHAR(20) NULL,     -- 'Critical', 'High', 'Medium', 'Low'
    
    -- Field information
    field_name VARCHAR(50) NULL,
    current_value NVARCHAR(200) NULL,
    expected_value NVARCHAR(200) NULL,
    
    -- Description
    description NVARCHAR(MAX) NULL,
    
    -- Metadata
    created_at DATETIME2 NOT NULL DEFAULT GETUTCDATE(),
    
    -- Foreign key to schedules
    CONSTRAINT FK_validation_schedule FOREIGN KEY (schedule_id) 
        REFERENCES dbo.schedules(schedule_id) ON DELETE CASCADE,
    
    -- Constraints
    CONSTRAINT CHK_validation_status CHECK (validation_status IN ('valid', 'impossible', 'error', 'warning')),
    CONSTRAINT CHK_criticality CHECK (criticality IN ('Critical', 'High', 'Medium', 'Low') OR criticality IS NULL)
);
PRINT '  ✓ Created table: analytics.schedule_validation_results';
GO

-- Index for primary access pattern: all validation results for a schedule
CREATE INDEX IX_validation_schedule_id 
    ON analytics.schedule_validation_results (schedule_id, validation_status)
    INCLUDE (scheduling_block_id, issue_type, criticality);
PRINT '  ✓ Created index: IX_validation_schedule_id';

-- Index for querying by status
CREATE INDEX IX_validation_status 
    ON analytics.schedule_validation_results (schedule_id, validation_status, criticality);
PRINT '  ✓ Created index: IX_validation_status';

-- Index for querying by block
CREATE INDEX IX_validation_block_id 
    ON analytics.schedule_validation_results (scheduling_block_id, validation_status);
PRINT '  ✓ Created index: IX_validation_block_id';
GO

-- ============================================================================
-- Stored Procedure: Populate analytics for a single schedule
-- This procedure is called by the Rust backend after schedule upload
-- ============================================================================

IF OBJECT_ID('analytics.sp_populate_schedule_analytics', 'P') IS NOT NULL
    DROP PROCEDURE analytics.sp_populate_schedule_analytics;
GO

CREATE PROCEDURE analytics.sp_populate_schedule_analytics
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
        original_block_id,
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
        sb.original_block_id,
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
        c.start_time_mjd,
        c.stop_time_mjd,
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
    OUTER APPLY (
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
          AND ISJSON(sb.visibility_periods_json) = 1
    ) AS vis
    WHERE ssb.schedule_id = @schedule_id;
    
    SELECT @@ROWCOUNT AS rows_inserted;
END
GO
PRINT '  ✓ Created procedure: analytics.sp_populate_schedule_analytics';
GO

-- ============================================================================
-- Stored Procedure: Delete analytics for a schedule (cleanup)
-- ============================================================================

IF OBJECT_ID('analytics.sp_delete_schedule_analytics', 'P') IS NOT NULL
    DROP PROCEDURE analytics.sp_delete_schedule_analytics;
GO

CREATE PROCEDURE analytics.sp_delete_schedule_analytics
    @schedule_id BIGINT
AS
BEGIN
    SET NOCOUNT ON;
    
    DELETE FROM analytics.schedule_blocks_analytics 
    WHERE schedule_id = @schedule_id;
    
    SELECT @@ROWCOUNT AS rows_deleted;
END
GO
PRINT '  ✓ Created procedure: analytics.sp_delete_schedule_analytics';
GO

-- ============================================================================
-- View: Convenient access to analytics data with additional computed fields
-- ============================================================================

IF OBJECT_ID('analytics.v_schedule_blocks_analytics', 'V') IS NOT NULL
    DROP VIEW analytics.v_schedule_blocks_analytics;
GO

CREATE VIEW analytics.v_schedule_blocks_analytics AS
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
PRINT '  ✓ Created view: analytics.v_schedule_blocks_analytics';
GO

-- ============================================================================
-- PART 3: VERIFICATION
-- ============================================================================

PRINT '';
PRINT '=== PART 3: Verification ===';
PRINT '';
GO

-- Count tables
DECLARE @base_tables INT, @analytics_tables INT;

SELECT @base_tables = COUNT(*)
FROM sys.tables t
JOIN sys.schemas s ON t.schema_id = s.schema_id
WHERE s.name = 'dbo' AND t.name IN (
    'schedules', 'targets', 'altitude_constraints', 'azimuth_constraints',
    'constraints', 'scheduling_blocks', 'schedule_scheduling_blocks',
    'visibility_periods', 'schedule_dark_periods'
);

SELECT @analytics_tables = COUNT(*)
FROM sys.tables t
JOIN sys.schemas s ON t.schema_id = s.schema_id
WHERE s.name = 'analytics' AND t.name IN ('schedule_blocks_analytics', 'schedule_validation_results');

PRINT '  Base tables created: ' + CAST(@base_tables AS NVARCHAR(10)) + '/9';
PRINT '  Analytics tables created: ' + CAST(@analytics_tables AS NVARCHAR(10)) + '/2';

IF @base_tables = 9 AND @analytics_tables = 2
BEGIN
    PRINT '';
    PRINT '============================================================================';
    PRINT ' ✅ DATABASE SETUP COMPLETE';
    PRINT '============================================================================';
    PRINT '';
    PRINT ' Tables:';
    PRINT '   - dbo.schedules';
    PRINT '   - dbo.targets';
    PRINT '   - dbo.altitude_constraints';
    PRINT '   - dbo.azimuth_constraints';
    PRINT '   - dbo.constraints';
    PRINT '   - dbo.scheduling_blocks';
    PRINT '   - dbo.schedule_scheduling_blocks';
    PRINT '   - dbo.visibility_periods';
    PRINT '   - dbo.schedule_dark_periods';
    PRINT '   - analytics.schedule_blocks_analytics';
    PRINT '   - analytics.schedule_validation_results';
    PRINT '';
    PRINT ' Next Steps:';
    PRINT '   1. Configure connection in .env file';
    PRINT '   2. Grant permissions to application user';
    PRINT '   3. Start the TSI application';
    PRINT '';
END
ELSE
BEGIN
    PRINT '';
    PRINT '⚠️  SETUP INCOMPLETE - Check error messages above';
    PRINT '';
END

PRINT ' Completed: ' + CONVERT(VARCHAR(23), GETDATE(), 121);
PRINT '============================================================================';
GO
