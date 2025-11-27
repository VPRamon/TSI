-- =====================
-- Observing schedules metadata
-- =====================
CREATE TABLE schedules (
    schedule_id      BIGSERIAL PRIMARY KEY,
    upload_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    checksum         TEXT
);
COMMENT ON TABLE schedules IS 'Logical group of scheduling blocks that were uploaded together.';
COMMENT ON COLUMN schedules.schedule_id IS 'Surrogate key for each schedule upload.';
COMMENT ON COLUMN schedules.upload_timestamp IS 'UTC timestamp recorded automatically on insertion.';
COMMENT ON COLUMN schedules.checksum IS 'Optional hash to track duplicates or tampering.';

-- =====================
-- Astronomical targets referenced by scheduling blocks
-- =====================
CREATE TABLE targets (
    target_id       BIGSERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    ra_deg          DOUBLE PRECISION NOT NULL,
    dec_deg         DOUBLE PRECISION NOT NULL,
    ra_pm_masyr     DOUBLE PRECISION NOT NULL DEFAULT 0,
    dec_pm_masyr    DOUBLE PRECISION NOT NULL DEFAULT 0,
    equinox         DOUBLE PRECISION NOT NULL DEFAULT 2000.0,
    CONSTRAINT targets_unique_natural
        UNIQUE (ra_deg, dec_deg, ra_pm_masyr, dec_pm_masyr, equinox),
    CONSTRAINT valid_ra_dec CHECK (
        ra_deg >= 0 AND ra_deg < 360
        AND dec_deg >= -90 AND dec_deg <= 90
    )
);
COMMENT ON TABLE targets IS 'Resolved sky positions with optional proper motion values.';
COMMENT ON COLUMN targets.target_id IS 'Surrogate key for each target.';
COMMENT ON COLUMN targets.name IS 'Human readable target identifier.';
COMMENT ON COLUMN targets.ra_deg IS 'Right ascension at the provided equinox, expressed in degrees.';
COMMENT ON COLUMN targets.dec_deg IS 'Declination at the provided equinox, expressed in degrees.';
COMMENT ON COLUMN targets.ra_pm_masyr IS 'Proper motion along right ascension in milliarcseconds per year.';
COMMENT ON COLUMN targets.dec_pm_masyr IS 'Proper motion along declination in milliarcseconds per year.';
COMMENT ON COLUMN targets.equinox IS 'Reference equinox for the coordinates and proper motions.';

-- =====================
-- Time periods that can be reused across schedules
-- =====================
CREATE TABLE periods (
    period_id      BIGSERIAL PRIMARY KEY,
    start_time_mjd DOUBLE PRECISION NOT NULL,
    stop_time_mjd  DOUBLE PRECISION NOT NULL,
    duration_sec   DOUBLE PRECISION GENERATED ALWAYS AS (
        GREATEST(stop_time_mjd - start_time_mjd, 0) * 86400.0
    ) STORED,
    CONSTRAINT periods_range_chk
        CHECK (start_time_mjd < stop_time_mjd),
    CONSTRAINT periods_unique
        UNIQUE (start_time_mjd, stop_time_mjd)
);
COMMENT ON TABLE periods IS 'Reusable time ranges expressed in Modified Julian Date.';
COMMENT ON COLUMN periods.period_id IS 'Surrogate key for the period.';
COMMENT ON COLUMN periods.start_time_mjd IS 'Start of the time span in Modified Julian Date.';
COMMENT ON COLUMN periods.stop_time_mjd IS 'End of the time span in Modified Julian Date.';
COMMENT ON COLUMN periods.duration_sec IS 'Stored length of the period in seconds.';

-- =====================
-- Atomic observing constraints
-- =====================
CREATE TABLE altitude_constraints (
    altitude_constraints_id BIGSERIAL PRIMARY KEY,
    min_alt_deg             DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_alt_deg             DOUBLE PRECISION NOT NULL DEFAULT 90,
    CONSTRAINT altitude_constraints_range_chk
        CHECK (min_alt_deg <= max_alt_deg),
    CONSTRAINT altitude_constraints_unique
        UNIQUE (min_alt_deg, max_alt_deg)
);
COMMENT ON TABLE altitude_constraints IS 'Minimum and maximum allowed target altitude for reuse across schedules.';
COMMENT ON COLUMN altitude_constraints.altitude_constraints_id IS 'Surrogate key for the altitude constraint.';
COMMENT ON COLUMN altitude_constraints.min_alt_deg IS 'Minimum elevation angle above the horizon, in degrees.';
COMMENT ON COLUMN altitude_constraints.max_alt_deg IS 'Maximum elevation angle above the horizon, in degrees.';

CREATE TABLE azimuth_constraints (
    azimuth_constraints_id BIGSERIAL PRIMARY KEY,
    min_az_deg             DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_az_deg             DOUBLE PRECISION NOT NULL DEFAULT 360,
    CONSTRAINT azimuth_constraints_range_chk
        CHECK (min_az_deg <= max_az_deg),
    CONSTRAINT azimuth_constraints_unique
        UNIQUE (min_az_deg, max_az_deg)
);
COMMENT ON TABLE azimuth_constraints IS 'Allowed azimuth range to avoid obstructions or mechanical limits.';
COMMENT ON COLUMN azimuth_constraints.azimuth_constraints_id IS 'Surrogate key for the azimuth constraint.';
COMMENT ON COLUMN azimuth_constraints.min_az_deg IS 'Minimum azimuth angle, in degrees.';
COMMENT ON COLUMN azimuth_constraints.max_az_deg IS 'Maximum azimuth angle, in degrees.';

-- =====================
-- Reusable composite constraints
-- =====================
CREATE TABLE constraints (
    constraints_id          BIGSERIAL PRIMARY KEY,
    time_constraints_id     BIGINT REFERENCES periods(period_id) ON DELETE SET NULL,
    altitude_constraints_id BIGINT REFERENCES altitude_constraints(altitude_constraints_id) ON DELETE SET NULL,
    azimuth_constraints_id  BIGINT REFERENCES azimuth_constraints(azimuth_constraints_id)  ON DELETE SET NULL,
    CONSTRAINT at_least_one_constraint CHECK (
        time_constraints_id IS NOT NULL
        OR altitude_constraints_id IS NOT NULL
        OR azimuth_constraints_id IS NOT NULL
    ),
    CONSTRAINT constraints_unique_combo
        UNIQUE (time_constraints_id, altitude_constraints_id, azimuth_constraints_id)
);
COMMENT ON TABLE constraints IS 'Composite constraint objects linking time, altitude, and azimuth requirements.';
COMMENT ON COLUMN constraints.constraints_id IS 'Surrogate key for the composite constraint.';
COMMENT ON COLUMN constraints.time_constraints_id IS 'Reference to the periods table when a time window applies.';
COMMENT ON COLUMN constraints.altitude_constraints_id IS 'Reference to altitude constraint parameters.';
COMMENT ON COLUMN constraints.azimuth_constraints_id IS 'Reference to azimuth constraint parameters.';

-- =====================
-- Scheduling blocks combining targets, constraints, and requested durations
-- =====================
CREATE TABLE scheduling_blocks (
    scheduling_block_id    BIGSERIAL PRIMARY KEY,
    target_id              BIGINT NOT NULL REFERENCES targets(target_id),
    constraints_id         BIGINT REFERENCES constraints(constraints_id),
    priority               NUMERIC(4,1) NOT NULL,
    min_observation_sec    INTEGER NOT NULL,
    requested_duration_sec INTEGER NOT NULL,
    CONSTRAINT valid_min_obs_req_dur CHECK (
        min_observation_sec >= 0
        AND requested_duration_sec >= 0
        AND min_observation_sec <= requested_duration_sec
    )
);
COMMENT ON TABLE scheduling_blocks IS 'Atomic observing requests for a single target with constraints and durations.';
COMMENT ON COLUMN scheduling_blocks.scheduling_block_id IS 'Surrogate key for each scheduling block.';
COMMENT ON COLUMN scheduling_blocks.target_id IS 'Target to observe for this block.';
COMMENT ON COLUMN scheduling_blocks.constraints_id IS 'Composite constraints that must be satisfied (optional).';
COMMENT ON COLUMN scheduling_blocks.priority IS 'Relative priority used during scheduling.';
COMMENT ON COLUMN scheduling_blocks.min_observation_sec IS 'Minimum amount of time worth executing, in seconds.';
COMMENT ON COLUMN scheduling_blocks.requested_duration_sec IS 'Ideal integration time requested from the scheduler.';

-- =====================
-- Relationship between schedules and scheduling blocks
-- =====================
CREATE TABLE schedule_scheduling_blocks (
    schedule_id          BIGINT NOT NULL REFERENCES schedules(schedule_id) ON DELETE CASCADE,
    scheduling_block_id  BIGINT NOT NULL REFERENCES scheduling_blocks(scheduling_block_id) ON DELETE CASCADE,
    scheduled_period_id  BIGINT REFERENCES periods(period_id) ON DELETE SET NULL,
    PRIMARY KEY (schedule_id, scheduling_block_id)
);
COMMENT ON TABLE schedule_scheduling_blocks IS 'Associates scheduling blocks with a schedule and optional scheduled time.';
COMMENT ON COLUMN schedule_scheduling_blocks.schedule_id IS 'Schedule that owns the scheduling block.';
COMMENT ON COLUMN schedule_scheduling_blocks.scheduling_block_id IS 'Scheduling block inserted in the schedule.';
COMMENT ON COLUMN schedule_scheduling_blocks.scheduled_period_id IS 'Optional period chosen for execution.';

-- =====================
-- Visibility and darkness periods per schedule
-- =====================
CREATE TABLE visibility_periods (
    schedule_id         BIGINT NOT NULL,
    scheduling_block_id BIGINT NOT NULL,
    period_id           BIGINT NOT NULL REFERENCES periods(period_id) ON DELETE CASCADE,
    PRIMARY KEY (schedule_id, scheduling_block_id, period_id),
    FOREIGN KEY (schedule_id, scheduling_block_id)
        REFERENCES schedule_scheduling_blocks (schedule_id, scheduling_block_id)
        ON DELETE CASCADE
);
COMMENT ON TABLE visibility_periods IS 'Precomputed windows when a scheduling block is observable within a schedule.';
COMMENT ON COLUMN visibility_periods.schedule_id IS 'Schedule for which the visibility was calculated.';
COMMENT ON COLUMN visibility_periods.scheduling_block_id IS 'Scheduling block covered by the visibility period.';
COMMENT ON COLUMN visibility_periods.period_id IS 'Reference to the reusable period entry.';

CREATE TABLE dark_periods (
    schedule_id         BIGINT NOT NULL REFERENCES schedules(schedule_id) ON DELETE CASCADE,
    period_id           BIGINT NOT NULL REFERENCES periods(period_id) ON DELETE CASCADE,
    PRIMARY KEY (schedule_id, period_id)
);
COMMENT ON TABLE dark_periods IS 'Dark or moonless intervals that improve observing conditions.';
COMMENT ON COLUMN dark_periods.schedule_id IS 'Schedule where the dark period applies.';
COMMENT ON COLUMN dark_periods.period_id IS 'Underlying time interval expressed as a period.';


-- Search SB by target
CREATE INDEX idx_scheduling_blocks_target
    ON scheduling_blocks (target_id);

-- Search SB by constraints
CREATE INDEX idx_scheduling_blocks_constraints
    ON scheduling_blocks (constraints_id);

-- Search by period start time
CREATE INDEX idx_periods_start_time
    ON periods (start_time_mjd);

-- Search visibility periods by SB
CREATE INDEX idx_visibility_periods_sb
    ON visibility_periods (scheduling_block_id);
