-- Telescope Scheduling Intelligence - relational schema for schedules & visibility windows
-- PostgreSQL-compatible; adjust data types/indexes if targeting another RDBMS.

CREATE TABLE ingest_runs (
    ingest_id           BIGSERIAL PRIMARY KEY,
    run_timestamp       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    schedule_source     TEXT NOT NULL,
    possible_source     TEXT,
    dark_periods_source TEXT,
    checksum            TEXT
);

CREATE TABLE targets (
    target_id                BIGINT PRIMARY KEY,
    name                     TEXT NOT NULL,
    ra_deg                   DOUBLE PRECISION NOT NULL,
    dec_deg                  DOUBLE PRECISION NOT NULL,
    ra_pm_masyr              DOUBLE PRECISION DEFAULT 0,
    dec_pm_masyr             DOUBLE PRECISION DEFAULT 0,
    equinox                  DOUBLE PRECISION DEFAULT 2000.0,
    extra_metadata           JSONB DEFAULT '{}'::JSONB
);

CREATE TABLE scheduling_blocks (
    scheduling_block_id     BIGINT PRIMARY KEY,
    target_id               BIGINT NOT NULL REFERENCES targets(target_id),
    ingest_id               BIGINT NOT NULL REFERENCES ingest_runs(ingest_id) ON DELETE CASCADE,
    priority                NUMERIC(4,1) NOT NULL,
    min_observation_sec     INTEGER NOT NULL,
    requested_duration_sec  INTEGER NOT NULL,
    status                  TEXT DEFAULT 'scheduled',
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE scheduled_periods (
    scheduled_period_id BIGSERIAL PRIMARY KEY,
    scheduling_block_id BIGINT NOT NULL REFERENCES scheduling_blocks(scheduling_block_id) ON DELETE CASCADE,
    start_time_mjd      DOUBLE PRECISION NOT NULL,
    stop_time_mjd       DOUBLE PRECISION NOT NULL,
    duration_sec        DOUBLE PRECISION GENERATED ALWAYS AS (GREATEST(stop_time_mjd - start_time_mjd, 0) * 86400.0) STORED,
    time_scale          TEXT NOT NULL DEFAULT 'UTC',
    time_format         TEXT NOT NULL DEFAULT 'MJD'
);

CREATE TABLE time_constraints (
    scheduling_block_id     BIGINT PRIMARY KEY REFERENCES scheduling_blocks(scheduling_block_id) ON DELETE CASCADE,
    fixed_start_time_mjd    DOUBLE PRECISION,
    fixed_stop_time_mjd     DOUBLE PRECISION,
    min_observation_sec     INTEGER,
    requested_duration_sec  INTEGER,
    notes                   TEXT
);

CREATE TABLE pointing_constraints (
    scheduling_block_id BIGINT PRIMARY KEY REFERENCES scheduling_blocks(scheduling_block_id) ON DELETE CASCADE,
    min_az_deg          DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_az_deg          DOUBLE PRECISION NOT NULL DEFAULT 360,
    min_el_deg          DOUBLE PRECISION NOT NULL DEFAULT 0,
    max_el_deg          DOUBLE PRECISION NOT NULL DEFAULT 90
);

CREATE TYPE visibility_window_type AS ENUM ('scheduled', 'possible', 'dark');

CREATE TABLE visibility_windows (
    window_id           BIGSERIAL PRIMARY KEY,
    scheduling_block_id BIGINT REFERENCES scheduling_blocks(scheduling_block_id) ON DELETE CASCADE,
    window_type         visibility_window_type NOT NULL,
    start_time_mjd      DOUBLE PRECISION NOT NULL,
    stop_time_mjd       DOUBLE PRECISION NOT NULL,
    duration_sec        DOUBLE PRECISION GENERATED ALWAYS AS (GREATEST(stop_time_mjd - start_time_mjd, 0) * 86400.0) STORED,
    source_file         TEXT,
    UNIQUE (scheduling_block_id, window_type, start_time_mjd, stop_time_mjd)
);

CREATE TABLE dark_periods (
    dark_period_id  BIGSERIAL PRIMARY KEY,
    site            TEXT NOT NULL,
    start_time_mjd  DOUBLE PRECISION NOT NULL,
    stop_time_mjd   DOUBLE PRECISION NOT NULL,
    moon_fraction   DOUBLE PRECISION,
    sun_alt_max_deg DOUBLE PRECISION,
    ingest_id       BIGINT REFERENCES ingest_runs(ingest_id) ON DELETE SET NULL
);

-- Aggregated stats mirroring the CSV used by the dashboard.
CREATE MATERIALIZED VIEW sb_visibility_stats AS
SELECT
    sb.scheduling_block_id,
    COUNT(vw.window_id) FILTER (WHERE vw.window_type = 'possible') AS num_visibility_periods,
    SUM(vw.duration_sec) FILTER (WHERE vw.window_type = 'possible') / 3600.0 AS total_visibility_hours,
    SUM(sp.duration_sec) / 3600.0 AS scheduled_hours
FROM scheduling_blocks sb
LEFT JOIN visibility_windows vw ON vw.scheduling_block_id = sb.scheduling_block_id
LEFT JOIN scheduled_periods sp ON sp.scheduling_block_id = sb.scheduling_block_id
GROUP BY sb.scheduling_block_id;

CREATE INDEX idx_scheduling_blocks_target ON scheduling_blocks(target_id);
CREATE INDEX idx_scheduled_periods_range ON scheduled_periods(start_time_mjd, stop_time_mjd);
CREATE INDEX idx_visibility_windows_type ON visibility_windows(window_type);
CREATE INDEX idx_dark_periods_range ON dark_periods(start_time_mjd, stop_time_mjd);
