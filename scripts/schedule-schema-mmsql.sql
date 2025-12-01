USE [db-schedules];
GO

-- Drop tables in correct order (respecting FK constraints)
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

-- Create tables in correct order (respecting FK dependencies)
CREATE TABLE dbo.schedules (
    schedule_id      BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_name    NVARCHAR(256) NOT NULL,
    upload_timestamp DATETIMEOFFSET(3) NOT NULL DEFAULT SYSUTCDATETIME(),
    checksum         NVARCHAR(64) NOT NULL,
    dark_periods_json NVARCHAR(MAX) NULL,
    CONSTRAINT UQ_schedules_checksum UNIQUE (checksum)
);

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

CREATE TABLE dbo.altitude_constraints (
    altitude_constraints_id BIGINT IDENTITY(1,1) PRIMARY KEY,
    min_alt_deg             FLOAT NOT NULL DEFAULT 0,
    max_alt_deg             FLOAT NOT NULL DEFAULT 90,
    CONSTRAINT altitude_constraints_range_chk_flat CHECK (min_alt_deg <= max_alt_deg),
    CONSTRAINT altitude_constraints_unique_flat UNIQUE (min_alt_deg, max_alt_deg)
);

CREATE TABLE dbo.azimuth_constraints (
    azimuth_constraints_id BIGINT IDENTITY(1,1) PRIMARY KEY,
    min_az_deg             FLOAT NOT NULL DEFAULT 0,
    max_az_deg             FLOAT NOT NULL DEFAULT 360,
    CONSTRAINT azimuth_constraints_range_chk_flat CHECK (min_az_deg <= max_az_deg),
    CONSTRAINT azimuth_constraints_unique_flat UNIQUE (min_az_deg, max_az_deg)
);

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

CREATE TABLE dbo.scheduling_blocks (
    scheduling_block_id    BIGINT IDENTITY(1,1) PRIMARY KEY,
    target_id              BIGINT NOT NULL REFERENCES dbo.targets(target_id),
    constraints_id         BIGINT NULL REFERENCES dbo.constraints(constraints_id),
    priority               FLOAT NOT NULL,
    min_observation_sec    INT NOT NULL,
    requested_duration_sec INT NOT NULL,
    visibility_periods_json NVARCHAR(MAX) NULL,
    CONSTRAINT valid_min_obs_req_dur_flat CHECK (min_observation_sec >= 0 AND requested_duration_sec >= 0 AND min_observation_sec <= requested_duration_sec)
);

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

CREATE TABLE dbo.visibility_periods (
    target_id       BIGINT NOT NULL REFERENCES dbo.targets(target_id) ON DELETE CASCADE,
    constraints_id  BIGINT NOT NULL REFERENCES dbo.constraints(constraints_id) ON DELETE CASCADE,
    start_time_mjd  FLOAT NOT NULL,
    stop_time_mjd   FLOAT NOT NULL,
    duration_sec    AS (CASE WHEN stop_time_mjd > start_time_mjd THEN (stop_time_mjd - start_time_mjd) * 86400.0 ELSE 0 END) PERSISTED,
    CONSTRAINT visibility_periods_pk_flat PRIMARY KEY (target_id, constraints_id, start_time_mjd, stop_time_mjd),
    CONSTRAINT visibility_periods_range_chk_flat CHECK (start_time_mjd < stop_time_mjd)
);

CREATE TABLE dbo.schedule_dark_periods (
    schedule_id     BIGINT NOT NULL REFERENCES dbo.schedules(schedule_id) ON DELETE CASCADE,
    start_time_mjd  FLOAT NOT NULL,
    stop_time_mjd   FLOAT NOT NULL,
    duration_sec    AS (CASE WHEN stop_time_mjd > start_time_mjd THEN (stop_time_mjd - start_time_mjd) * 86400.0 ELSE 0 END) PERSISTED,
    CONSTRAINT schedule_dark_periods_pk_flat PRIMARY KEY (schedule_id, start_time_mjd, stop_time_mjd),
    CONSTRAINT schedule_dark_periods_range_chk_flat CHECK (start_time_mjd < stop_time_mjd)
);

PRINT '✅ All tables created successfully';
GO

-- Verify creation
SELECT TABLE_NAME FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA = 'dbo' ORDER BY TABLE_NAME;
GO