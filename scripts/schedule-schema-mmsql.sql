-- =========================================================
-- Cleanup for idempotent execution
-- =========================================================

IF OBJECT_ID('dbo.schedule_scheduling_blocks', 'U') IS NOT NULL DROP TABLE dbo.schedule_scheduling_blocks;
IF OBJECT_ID('dbo.visibility_periods', 'U') IS NOT NULL DROP TABLE dbo.visibility_periods;
IF OBJECT_ID('dbo.schedule_dark_periods', 'U') IS NOT NULL DROP TABLE dbo.schedule_dark_periods;

IF OBJECT_ID('dbo.scheduling_blocks', 'U') IS NOT NULL DROP TABLE dbo.scheduling_blocks;
IF OBJECT_ID('dbo.constraints', 'U') IS NOT NULL DROP TABLE dbo.constraints;
IF OBJECT_ID('dbo.azimuth_constraints', 'U') IS NOT NULL DROP TABLE dbo.azimuth_constraints;
IF OBJECT_ID('dbo.altitude_constraints', 'U') IS NOT NULL DROP TABLE dbo.altitude_constraints;

IF OBJECT_ID('dbo.targets', 'U') IS NOT NULL DROP TABLE dbo.targets;
IF OBJECT_ID('dbo.schedules', 'U') IS NOT NULL DROP TABLE dbo.schedules;
GO


-- =========================================================
-- Schedules
-- =========================================================

CREATE TABLE dbo.schedules (
    schedule_id      BIGINT IDENTITY(1,1) PRIMARY KEY,
    schedule_name    NVARCHAR(256) NOT NULL,  -- Human-readable name of the schedule upload
    upload_timestamp DATETIMEOFFSET(3) NOT NULL 
        CONSTRAINT DF_schedules_upload_timestamp_flat DEFAULT SYSUTCDATETIME(),
    checksum         NVARCHAR(64) NOT NULL,   -- Required + unique identifier of the schedule
    CONSTRAINT UQ_schedules_checksum UNIQUE (checksum)
);
INSERT INTO dbo.schedules (schedule_name, checksum)
SELECT @schedule_name, @checksum
WHERE NOT EXISTS (
    SELECT 1
    FROM dbo.schedules
    WHERE checksum = @checksum
);
-- A schedule upload (e.g. one run of the scheduler).


-- =========================================================
-- Astronomical targets
-- =========================================================

CREATE TABLE dbo.targets (
    target_id       BIGINT IDENTITY(1,1) PRIMARY KEY,
    name            NVARCHAR(MAX) NOT NULL,      -- Human-readable target name or identifier
    ra_deg          FLOAT NOT NULL,              -- Right ascension in degrees [0, 360)
    dec_deg         FLOAT NOT NULL,              -- Declination in degrees [-90, +90]
    ra_pm_masyr     FLOAT NOT NULL DEFAULT 0,    -- Proper motion in RA (mas/yr)
    dec_pm_masyr    FLOAT NOT NULL DEFAULT 0,    -- Proper motion in Dec (mas/yr)
    equinox         FLOAT NOT NULL DEFAULT 2000.0, -- Reference equinox (e.g. 2000.0)
    CONSTRAINT targets_unique_natural_flat
        UNIQUE (ra_deg, dec_deg, ra_pm_masyr, dec_pm_masyr, equinox),
    CONSTRAINT valid_ra_dec_flat CHECK (
        ra_deg >= 0 AND ra_deg < 360
        AND dec_deg >= -90 AND dec_deg <= 90
    )
);
-- Resolved sky positions with optional proper motion.


-- =========================================================
-- Atomic observing constraints
-- =========================================================

CREATE TABLE dbo.altitude_constraints (
    altitude_constraints_id BIGINT IDENTITY(1,1) PRIMARY KEY,
    min_alt_deg             FLOAT NOT NULL DEFAULT 0,   -- Minimum altitude (deg)
    max_alt_deg             FLOAT NOT NULL DEFAULT 90,  -- Maximum altitude (deg)
    CONSTRAINT altitude_constraints_range_chk_flat
        CHECK (min_alt_deg <= max_alt_deg),
    CONSTRAINT altitude_constraints_unique_flat
        UNIQUE (min_alt_deg, max_alt_deg)
);
-- Reusable minimum and maximum altitude constraints.


CREATE TABLE dbo.azimuth_constraints (
    azimuth_constraints_id BIGINT IDENTITY(1,1) PRIMARY KEY,
    min_az_deg             FLOAT NOT NULL DEFAULT 0,    -- Minimum azimuth (deg)
    max_az_deg             FLOAT NOT NULL DEFAULT 360,  -- Maximum azimuth (deg)
    CONSTRAINT azimuth_constraints_range_chk_flat
        CHECK (min_az_deg <= max_az_deg),
    CONSTRAINT azimuth_constraints_unique_flat
        UNIQUE (min_az_deg, max_az_deg)
);
-- Reusable azimuth constraints, typically to avoid obstructions or mechanical limits.

-- =========================================================
-- Composite constraints (time + altitude + azimuth) - flat time window
-- =========================================================

CREATE TABLE dbo.constraints (
    constraints_id          BIGINT IDENTITY(1,1) PRIMARY KEY,
    -- Optional time window directly on the constraint (MJD)
    start_time_mjd          FLOAT NULL,
    stop_time_mjd           FLOAT NULL,
    altitude_constraints_id BIGINT NULL
        REFERENCES dbo.altitude_constraints(altitude_constraints_id)
        ON DELETE SET NULL,
    azimuth_constraints_id  BIGINT NULL
        REFERENCES dbo.azimuth_constraints(azimuth_constraints_id)
        ON DELETE SET NULL,

    -- At least one component must be present
    CONSTRAINT at_least_one_constraint_flat CHECK (
        start_time_mjd IS NOT NULL
        OR altitude_constraints_id IS NOT NULL
        OR azimuth_constraints_id IS NOT NULL
    ),

    -- If time window is present, enforce ordering
    CONSTRAINT constraints_time_range_chk_flat CHECK (
        start_time_mjd IS NULL
        OR stop_time_mjd  IS NULL
        OR start_time_mjd < stop_time_mjd
    ),

    -- Prevent duplicate composite constraints
    CONSTRAINT constraints_unique_combo_flat
        UNIQUE (start_time_mjd, stop_time_mjd, altitude_constraints_id, azimuth_constraints_id)
);
-- Composite constraint objects that may combine time, altitude, and azimuth constraints.
-- Time is now stored directly as MJD start/stop (no FK to a periods table).


-- =========================================================
-- Visibility periods per (target, constraints) - flat
-- =========================================================

CREATE TABLE dbo.visibility_periods (
    target_id       BIGINT NOT NULL
        REFERENCES dbo.targets(target_id) ON DELETE CASCADE,
    constraints_id  BIGINT NOT NULL
        REFERENCES dbo.constraints(constraints_id) ON DELETE CASCADE,
    start_time_mjd  FLOAT NOT NULL,   -- Start of visibility interval in MJD
    stop_time_mjd   FLOAT NOT NULL,   -- End of visibility interval in MJD,
    duration_sec    AS (
        CASE 
            WHEN stop_time_mjd > start_time_mjd
                THEN (stop_time_mjd - start_time_mjd) * 86400.0
            ELSE 0
        END
    ) PERSISTED,
    CONSTRAINT visibility_periods_pk_flat
        PRIMARY KEY (target_id, constraints_id, start_time_mjd, stop_time_mjd),
    CONSTRAINT visibility_periods_range_chk_flat
        CHECK (start_time_mjd < stop_time_mjd)
);
-- Each (target, constraints) pair has zero or more visibility intervals (no period sets).


-- =========================================================
-- Scheduling blocks (atomic observing requests)
-- =========================================================

CREATE TABLE dbo.scheduling_blocks (
    scheduling_block_id    BIGINT IDENTITY(1,1) PRIMARY KEY,
    target_id              BIGINT NOT NULL 
        REFERENCES dbo.targets(target_id),
    constraints_id         BIGINT NULL
        REFERENCES dbo.constraints(constraints_id),
    priority               NUMERIC(4,1) NOT NULL,  -- Relative scheduling priority
    min_observation_sec    INT NOT NULL,          -- Minimum viable observation time
    requested_duration_sec INT NOT NULL,          -- Ideal requested duration
    CONSTRAINT valid_min_obs_req_dur_flat CHECK (
        min_observation_sec >= 0
        AND requested_duration_sec >= 0
        AND min_observation_sec <= requested_duration_sec
    )
);
-- Atomic observing requests for a single target with associated constraints and durations.


-- =========================================================
-- Dark periods per schedule - flat
-- =========================================================

CREATE TABLE dbo.schedule_dark_periods (
    schedule_id     BIGINT NOT NULL
        REFERENCES dbo.schedules(schedule_id) ON DELETE CASCADE,
    start_time_mjd  FLOAT NOT NULL,
    stop_time_mjd   FLOAT NOT NULL,
    duration_sec    AS (
        CASE 
            WHEN stop_time_mjd > start_time_mjd
                THEN (stop_time_mjd - start_time_mjd) * 86400.0
            ELSE 0
        END
    ) PERSISTED,
    CONSTRAINT schedule_dark_periods_pk_flat
        PRIMARY KEY (schedule_id, start_time_mjd, stop_time_mjd),
    CONSTRAINT schedule_dark_periods_range_chk_flat
        CHECK (start_time_mjd < stop_time_mjd)
);
-- Dark time windows are now directly stored per schedule.


-- =========================================================
-- Relationship between schedules and scheduling blocks
-- =========================================================

CREATE TABLE dbo.schedule_scheduling_blocks (
    schedule_id          BIGINT NOT NULL,
    scheduling_block_id  BIGINT NOT NULL,
    -- Optional specific execution window (flat, independent of dark/visibility tables)
    start_time_mjd       FLOAT NULL,
    stop_time_mjd        FLOAT NULL,
    duration_sec         AS (
        CASE 
            WHEN start_time_mjd IS NOT NULL
             AND stop_time_mjd  IS NOT NULL
             AND stop_time_mjd > start_time_mjd
                THEN (stop_time_mjd - start_time_mjd) * 86400.0
            ELSE 0
        END
    ) PERSISTED,
    CONSTRAINT PK_schedule_scheduling_blocks_flat
        PRIMARY KEY (schedule_id, scheduling_block_id),
    CONSTRAINT FK_ssb_schedules_flat
        FOREIGN KEY (schedule_id)
        REFERENCES dbo.schedules(schedule_id)
        ON DELETE CASCADE,
    CONSTRAINT FK_ssb_scheduling_blocks_flat
        FOREIGN KEY (scheduling_block_id)
        REFERENCES dbo.scheduling_blocks(scheduling_block_id)
        ON DELETE CASCADE,
    CONSTRAINT ssb_time_range_chk_flat CHECK (
        start_time_mjd IS NULL
        OR stop_time_mjd  IS NULL
        OR start_time_mjd < stop_time_mjd
    )
);
-- Associates scheduling blocks with schedules and optionally with a concrete execution interval.


-- =========================================================
-- Indexes
-- =========================================================

-- Search scheduling blocks by target
CREATE INDEX idx_scheduling_blocks_target_flat
    ON dbo.scheduling_blocks (target_id);

-- Search scheduling blocks by constraints
CREATE INDEX idx_scheduling_blocks_constraints_flat
    ON dbo.scheduling_blocks (constraints_id);

-- Search visibility periods by time
CREATE INDEX idx_visibility_periods_time_flat
    ON dbo.visibility_periods (start_time_mjd);

-- Search dark periods by time
CREATE INDEX idx_schedule_dark_periods_time_flat
    ON dbo.schedule_dark_periods (start_time_mjd);

-- Search scheduled executions by time
CREATE INDEX idx_ssb_time_flat
    ON dbo.schedule_scheduling_blocks (start_time_mjd);
GO
