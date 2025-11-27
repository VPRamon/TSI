-- =====================
-- Cleanup for idempotent execution
-- =====================
IF OBJECT_ID('dbo.visibility_periods', 'U') IS NOT NULL DROP TABLE dbo.visibility_periods;
IF OBJECT_ID('dbo.dark_periods', 'U') IS NOT NULL DROP TABLE dbo.dark_periods;
IF OBJECT_ID('dbo.schedule_scheduling_blocks', 'U') IS NOT NULL DROP TABLE dbo.schedule_scheduling_blocks;
IF OBJECT_ID('dbo.scheduling_blocks', 'U') IS NOT NULL DROP TABLE dbo.scheduling_blocks;
IF OBJECT_ID('dbo.constraints', 'U') IS NOT NULL DROP TABLE dbo.constraints;
IF OBJECT_ID('dbo.azimuth_constraints', 'U') IS NOT NULL DROP TABLE dbo.azimuth_constraints;
IF OBJECT_ID('dbo.altitude_constraints', 'U') IS NOT NULL DROP TABLE dbo.altitude_constraints;
IF OBJECT_ID('dbo.periods', 'U') IS NOT NULL DROP TABLE dbo.periods;
IF OBJECT_ID('dbo.targets', 'U') IS NOT NULL DROP TABLE dbo.targets;
IF OBJECT_ID('dbo.schedules', 'U') IS NOT NULL DROP TABLE dbo.schedules;

-- =====================
-- Observing schedules metadata
-- =====================
CREATE TABLE dbo.schedules (
    schedule_id      BIGINT IDENTITY(1,1) PRIMARY KEY,
    upload_timestamp DATETIMEOFFSET(3) NOT NULL 
        CONSTRAINT DF_schedules_upload_timestamp DEFAULT SYSUTCDATETIME(),
    checksum         NVARCHAR(MAX) NULL
);
-- Logical group of scheduling blocks that were uploaded together.
-- Surrogate key for each schedule upload.
-- UTC timestamp recorded automatically on insertion.
-- Optional hash to track duplicates or tampering.

-- =====================
-- Astronomical targets referenced by scheduling blocks
-- =====================
CREATE TABLE dbo.targets (
    target_id       BIGINT IDENTITY(1,1) PRIMARY KEY,
    name            NVARCHAR(MAX) NOT NULL,
    ra_deg          FLOAT NOT NULL,
    dec_deg         FLOAT NOT NULL,
    ra_pm_masyr     FLOAT NOT NULL DEFAULT 0,
    dec_pm_masyr    FLOAT NOT NULL DEFAULT 0,
    equinox         FLOAT NOT NULL DEFAULT 2000.0,
    CONSTRAINT targets_unique_natural
        UNIQUE (ra_deg, dec_deg, ra_pm_masyr, dec_pm_masyr, equinox),
    CONSTRAINT valid_ra_dec CHECK (
        ra_deg >= 0 AND ra_deg < 360
        AND dec_deg >= -90 AND dec_deg <= 90
    )
);
-- Resolved sky positions with optional proper motion values.
-- Surrogate key for each target.
-- Human readable target identifier.
-- Right ascension at the provided equinox, expressed in degrees.
-- Declination at the provided equinox, expressed in degrees.
-- Proper motion along right ascension in milliarcseconds per year.
-- Proper motion along declination in milliarcseconds per year.
-- Reference equinox for the coordinates and proper motions.

-- =====================
-- Time periods that can be reused across schedules
-- =====================
CREATE TABLE dbo.periods (
    period_id      BIGINT IDENTITY(1,1) PRIMARY KEY,
    start_time_mjd FLOAT NOT NULL,
    stop_time_mjd  FLOAT NOT NULL,
    duration_sec   AS (
        CASE 
            WHEN stop_time_mjd > start_time_mjd 
                THEN (stop_time_mjd - start_time_mjd) * 86400.0
            ELSE 0
        END
    ) PERSISTED,
    CONSTRAINT periods_range_chk
        CHECK (start_time_mjd < stop_time_mjd),
    CONSTRAINT periods_unique
        UNIQUE (start_time_mjd, stop_time_mjd)
);
-- Reusable time ranges expressed in Modified Julian Date.
-- Surrogate key for the period.
-- Start of the time span in Modified Julian Date.
-- End of the time span in Modified Julian Date.
-- Stored length of the period in seconds.

-- =====================
-- Atomic observing constraints
-- =====================
CREATE TABLE dbo.altitude_constraints (
    altitude_constraints_id BIGINT IDENTITY(1,1) PRIMARY KEY,
    min_alt_deg             FLOAT NOT NULL DEFAULT 0,
    max_alt_deg             FLOAT NOT NULL DEFAULT 90,
    CONSTRAINT altitude_constraints_range_chk
        CHECK (min_alt_deg <= max_alt_deg),
    CONSTRAINT altitude_constraints_unique
        UNIQUE (min_alt_deg, max_alt_deg)
);
-- Minimum and maximum allowed target altitude for reuse across schedules.
-- Surrogate key for the altitude constraint.
-- Minimum elevation angle above the horizon, in degrees.
-- Maximum elevation angle above the horizon, in degrees.

CREATE TABLE dbo.azimuth_constraints (
    azimuth_constraints_id BIGINT IDENTITY(1,1) PRIMARY KEY,
    min_az_deg             FLOAT NOT NULL DEFAULT 0,
    max_az_deg             FLOAT NOT NULL DEFAULT 360,
    CONSTRAINT azimuth_constraints_range_chk
        CHECK (min_az_deg <= max_az_deg),
    CONSTRAINT azimuth_constraints_unique
        UNIQUE (min_az_deg, max_az_deg)
);
-- Allowed azimuth range to avoid obstructions or mechanical limits.
-- Surrogate key for the azimuth constraint.
-- Minimum azimuth angle, in degrees.
-- Maximum azimuth angle, in degrees.

-- =====================
-- Reusable composite constraints
-- =====================
CREATE TABLE dbo.constraints (
    constraints_id          BIGINT IDENTITY(1,1) PRIMARY KEY,
    time_constraints_id     BIGINT NULL,
    altitude_constraints_id BIGINT NULL,
    azimuth_constraints_id  BIGINT NULL,
    CONSTRAINT at_least_one_constraint CHECK (
        time_constraints_id IS NOT NULL
        OR altitude_constraints_id IS NOT NULL
        OR azimuth_constraints_id IS NOT NULL
    ),
    CONSTRAINT constraints_unique_combo
        UNIQUE (time_constraints_id, altitude_constraints_id, azimuth_constraints_id),
    CONSTRAINT FK_constraints_periods
        FOREIGN KEY (time_constraints_id)
        REFERENCES dbo.periods(period_id)
        ON DELETE SET NULL,
    CONSTRAINT FK_constraints_altitude
        FOREIGN KEY (altitude_constraints_id)
        REFERENCES dbo.altitude_constraints(altitude_constraints_id)
        ON DELETE SET NULL,
    CONSTRAINT FK_constraints_azimuth
        FOREIGN KEY (azimuth_constraints_id)
        REFERENCES dbo.azimuth_constraints(azimuth_constraints_id)
        ON DELETE SET NULL
);
-- Composite constraint objects linking time, altitude, and azimuth requirements.
-- Surrogate key for the composite constraint.
-- Reference to the periods table when a time window applies.
-- Reference to altitude constraint parameters.
-- Reference to azimuth constraint parameters.

-- =====================
-- Scheduling blocks combining targets, constraints, and requested durations
-- =====================
CREATE TABLE dbo.scheduling_blocks (
    scheduling_block_id    BIGINT IDENTITY(1,1) PRIMARY KEY,
    target_id              BIGINT NOT NULL 
        REFERENCES dbo.targets(target_id),
    constraints_id         BIGINT NULL
        REFERENCES dbo.constraints(constraints_id),
    priority               NUMERIC(4,1) NOT NULL,
    min_observation_sec    INT NOT NULL,
    requested_duration_sec INT NOT NULL,
    CONSTRAINT valid_min_obs_req_dur CHECK (
        min_observation_sec >= 0
        AND requested_duration_sec >= 0
        AND min_observation_sec <= requested_duration_sec
    )
);
-- Atomic observing requests for a single target with constraints and durations.
-- Surrogate key for each scheduling block.
-- Target to observe for this block.
-- Composite constraints that must be satisfied (optional).
-- Relative priority used during scheduling.
-- Minimum amount of time worth executing, in seconds.
-- Ideal integration time requested from the scheduler.

-- =====================
-- Relationship between schedules and scheduling blocks
-- =====================
CREATE TABLE dbo.schedule_scheduling_blocks (
    schedule_id          BIGINT NOT NULL,
    scheduling_block_id  BIGINT NOT NULL,
    scheduled_period_id  BIGINT NULL,
    CONSTRAINT PK_schedule_scheduling_blocks 
        PRIMARY KEY (schedule_id, scheduling_block_id),
    CONSTRAINT FK_ssb_schedules
        FOREIGN KEY (schedule_id)
        REFERENCES dbo.schedules(schedule_id)
        ON DELETE CASCADE,
    CONSTRAINT FK_ssb_scheduling_blocks
        FOREIGN KEY (scheduling_block_id)
        REFERENCES dbo.scheduling_blocks(scheduling_block_id)
        ON DELETE CASCADE,
    CONSTRAINT FK_ssb_periods
        FOREIGN KEY (scheduled_period_id)
        REFERENCES dbo.periods(period_id)
        ON DELETE SET NULL
);
-- Associates scheduling blocks with a schedule and optional scheduled time.
-- Schedule that owns the scheduling block.
-- Scheduling block inserted in the schedule.
-- Optional period chosen for execution.

-- =====================
-- Visibility and darkness periods per schedule
-- =====================
CREATE TABLE dbo.visibility_periods (
    schedule_id         BIGINT NOT NULL,
    scheduling_block_id BIGINT NOT NULL,
    period_id           BIGINT NOT NULL,
    CONSTRAINT PK_visibility_periods 
        PRIMARY KEY (schedule_id, scheduling_block_id, period_id),
    CONSTRAINT FK_visibility_periods_periods
        FOREIGN KEY (period_id)
        REFERENCES dbo.periods(period_id)
        ON DELETE CASCADE,
    CONSTRAINT FK_visibility_periods_ssb
        FOREIGN KEY (schedule_id, scheduling_block_id)
        REFERENCES dbo.schedule_scheduling_blocks (schedule_id, scheduling_block_id)
        ON DELETE CASCADE
);
-- Precomputed windows when a scheduling block is observable within a schedule.
-- Schedule for which the visibility was calculated.
-- Scheduling block covered by the visibility period.
-- Reference to the reusable period entry.

CREATE TABLE dbo.dark_periods (
    schedule_id BIGINT NOT NULL,
    period_id   BIGINT NOT NULL,
    CONSTRAINT PK_dark_periods 
        PRIMARY KEY (schedule_id, period_id),
    CONSTRAINT FK_dark_periods_schedules
        FOREIGN KEY (schedule_id)
        REFERENCES dbo.schedules(schedule_id)
        ON DELETE CASCADE,
    CONSTRAINT FK_dark_periods_periods
        FOREIGN KEY (period_id)
        REFERENCES dbo.periods(period_id)
        ON DELETE CASCADE
);
-- Dark or moonless intervals that improve observing conditions.
-- Schedule where the dark period applies.
-- Underlying time interval expressed as a period.

-- =====================
-- Indexes
-- =====================

-- Search SB by target
CREATE INDEX idx_scheduling_blocks_target
    ON dbo.scheduling_blocks (target_id);

-- Search SB by constraints
CREATE INDEX idx_scheduling_blocks_constraints
    ON dbo.scheduling_blocks (constraints_id);

-- Search by period start time
CREATE INDEX idx_periods_start_time
    ON dbo.periods (start_time_mjd);

-- Search visibility periods by SB
CREATE INDEX idx_visibility_periods_sb
    ON dbo.visibility_periods (scheduling_block_id);
